//! FFmpeg-based video processing runtime

use crate::protocol::audio::AudioProcessor;
use crate::protocol::video::{SilenceMode, SilenceSegment, TrimSegment, VideoProcessor};
use crate::runtime::loudnorm::enhance_audio_impl;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// FFmpeg-based video processor implementation
pub struct FfmpegVideoProcessor;

impl FfmpegVideoProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FfmpegVideoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoProcessor for FfmpegVideoProcessor {
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        min_duration: f32,
    ) -> Result<Vec<SilenceSegment>> {
        let output = Command::new("ffmpeg")
            .args([
                "-i",
                path.to_str().context("invalid path")?,
                "-af",
                &format!("silencedetect=noise={}dB:d={}", threshold_db, min_duration),
                "-f",
                "null",
                "-",
            ])
            .output()
            .context("failed to execute ffmpeg")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(parse_ffmpeg_silence(&stderr))
    }

    fn calculate_keep_segments(
        &self,
        silences: &[SilenceSegment],
        video_duration: f32,
        padding: f32,
        mode: SilenceMode,
        speedup_factor: f32,
        min_silence_for_speedup: f32,
    ) -> Vec<TrimSegment> {
        calculate_keep_segments(
            silences,
            video_duration,
            padding,
            mode,
            speedup_factor,
            min_silence_for_speedup,
        )
    }

    fn trim_segments(
        &self,
        input: &Path,
        output: &Path,
        segments: &[TrimSegment],
        mut progress: Option<&mut dyn FnMut(f32)>,
    ) -> Result<()> {
        if segments.is_empty() {
            anyhow::bail!("No segments to process");
        }

        let segments: Vec<_> = segments
            .iter()
            .map(|s| ProcessedSegment {
                start: s.start,
                end: s.end,
                speed: s.speed,
            })
            .collect();

        if segments.len() <= TRIM_SEGMENTS_PER_CHUNK {
            run_trim_filter_job(input, output, &segments)?;
            if let Some(ref mut p) = progress {
                p(1.0);
            }
            return Ok(());
        }

        let chunk_dir = create_trim_chunk_dir(output)?;
        let chunk_count = segments.len().div_ceil(TRIM_SEGMENTS_PER_CHUNK);
        let mut chunk_files = Vec::with_capacity(chunk_count);

        for (idx, chunk) in segments.chunks(TRIM_SEGMENTS_PER_CHUNK).enumerate() {
            let chunk_path = chunk_dir.join(format!("chunk_{idx:04}.mp4"));
            run_trim_filter_job(input, &chunk_path, chunk)?;
            chunk_files.push(chunk_path);
            if let Some(ref mut p) = progress {
                p((idx + 1) as f32 / (chunk_count + 1) as f32);
            }
        }

        concat_chunk_files(&chunk_files, output)?;
        if let Some(ref mut p) = progress {
            p(1.0);
        }

        let _ = std::fs::remove_dir_all(&chunk_dir);
        Ok(())
    }

    fn stabilize(&self, input: &Path, output: &Path) -> Result<()> {
        let input_str = input.to_str().context("invalid input path")?;
        let output_str = output.to_str().context("invalid output path")?;
        let trf_file = "/tmp/transforms.trf";

        // Pass 1: Detect motion
        let status1 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabdetect=stepsize=6:shakiness=5:accuracy=15:result={}",
                    trf_file
                ),
                "-f",
                "null",
                "-",
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 1)")?;

        if !status1.success() {
            anyhow::bail!("ffmpeg stabilize pass 1 failed with status: {}", status1);
        }

        // Pass 2: Apply stabilization
        let status2 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabtransform=input={}:smoothing=10:optzoom=1:interpol=bicubic",
                    trf_file
                ),
                "-c:a",
                "copy",
                "-y",
                output_str,
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 2)")?;

        let _ = std::fs::remove_file(trf_file);

        if !status2.success() {
            anyhow::bail!("ffmpeg stabilize pass 2 failed with status: {}", status2);
        }

        Ok(())
    }

    fn reframe(&self, input: &Path, output: &Path) -> Result<PathBuf> {
        info!("Auto-reframe: Analyzing video for face tracking...");

        // Try ML-powered reframe, fall back to center crop
        let filter = match crate::runtime::ml::AutoReframeProcessor::new() {
            Ok(processor) => match processor.analyze_video(input, 1.0) {
                Ok(crop_regions) => {
                    let (w, h) = crate::runtime::ml::FrameExtractor::get_video_dimensions(input)
                        .unwrap_or((1920, 1080));
                    processor.generate_crop_filter(&crop_regions, w, h)
                }
                Err(e) => {
                    warn!(error = %e, "Face detection failed, using center crop");
                    "crop=ih*9/16:ih,scale=1080:1920".to_string()
                }
            },
            Err(e) => {
                warn!(error = %e, "Could not load face detection model, using center crop");
                "crop=ih*9/16:ih,scale=1080:1920".to_string()
            }
        };

        info!(filter = %filter, "Applying crop filter");

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                &filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg reframe failed with status: {}", status);
        }

        Ok(output.to_path_buf())
    }

    fn blur_background(&self, input: &Path, output: &Path) -> Result<()> {
        info!("Background blur: Processing video...");

        let use_ml = std::env::var("DRACON_ML_BLUR")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if use_ml {
            info!("Using ML-powered background blur (experimental)...");
        }

        // Use ffmpeg's boxblur
        let filter = "boxblur=20:5";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg blur_background failed with status: {}", status);
        }

        Ok(())
    }

    fn color_correct(&self, input: &Path, output: &Path) -> Result<()> {
        let filter = "eq=contrast=1.1:brightness=0.05:saturation=1.1,unsharp=5:5:0.5:5:5:0.0";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg color_correct failed with status: {}", status);
        }

        Ok(())
    }

    fn enhance_audio(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()> {
        enhance_audio_impl(input, output, target_lufs)
    }

    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()> {
        let filter = "afftdn=nf=-25:tn=1";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-af",
                filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg reduce_noise failed with status: {}", status);
        }

        Ok(())
    }

    fn concatenate(
        &self,
        intro: Option<&Path>,
        main: &Path,
        outro: Option<&Path>,
        output: &Path,
    ) -> Result<()> {
        concatenate_videos(intro, main, outro, output)
    }

    fn get_duration(&self, path: &Path) -> Result<f32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path.to_str().context("invalid path")?,
            ])
            .output()
            .context("failed to execute ffprobe")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            warn!(stderr = %stderr, "ffprobe warning");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let duration: f32 = stdout.trim().parse().context("failed to parse duration")?;

        Ok(duration)
    }
}

impl AudioProcessor for FfmpegVideoProcessor {
    fn enhance(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()> {
        enhance_audio_impl(input, output, target_lufs)
    }

    fn denoise(&self, input: &Path, output: &Path) -> Result<()> {
        let filter = "afftdn=nf=-25:tn=1";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-af",
                filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg denoise failed with status: {}", status);
        }

        Ok(())
    }

    fn mix_with_music(
        &self,
        video: &Path,
        music: &Path,
        output: &Path,
        speech_segments: &[(f32, f32)],
        duck_volume: f32,
    ) -> Result<()> {
        mix_with_music_impl(video, music, output, speech_segments, duck_volume)
    }
}

// === Helper Functions ===

fn parse_ffmpeg_silence(output: &str) -> Vec<SilenceSegment> {
    let mut segments = Vec::new();
    let mut current_start: Option<f32> = None;

    for line in output.lines() {
        if line.contains("silence_start:") {
            if let Some(pos) = line.find("silence_start:") {
                let val_str = &line[pos + "silence_start:".len()..].trim();
                if let Ok(start) = val_str.parse::<f32>() {
                    current_start = Some(start);
                }
            }
        } else if line.contains("silence_end:") {
            if let Some(start) = current_start.take() {
                if let Some(pos) = line.find("silence_end:") {
                    let part = &line[pos + "silence_end:".len()..];
                    let val_str = if let Some(pipe_pos) = part.find('|') {
                        &part[..pipe_pos]
                    } else {
                        part
                    };
                    if let Ok(end) = val_str.trim().parse::<f32>() {
                        segments.push(SilenceSegment { start, end });
                    }
                }
            }
        }
    }
    segments
}

#[derive(Debug, Clone)]
pub struct ProcessedSegment {
    pub start: f32,
    pub end: f32,
    pub speed: f32,
}

const TRIM_SEGMENTS_PER_CHUNK: usize = 48;

pub fn calculate_keep_segments(
    silence_segments: &[SilenceSegment],
    total_duration: f32,
    padding: f32,
    mode: SilenceMode,
    speedup_factor: f32,
    min_silence_for_speedup: f32,
) -> Vec<TrimSegment> {
    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        let keep_end = (silence.start + padding).min(total_duration);
        if keep_end > current_pos {
            processed.push(TrimSegment {
                start: current_pos,
                end: keep_end,
                speed: 1.0,
            });
        }

        match mode {
            SilenceMode::Cut => {
                current_pos = (silence.end - padding).max(0.0);
            }
            SilenceMode::Speedup => {
                let silence_start = (silence.start + padding).max(0.0);
                let silence_end = (silence.end - padding).min(total_duration);
                let silence_duration = silence_end - silence_start;

                if silence_duration >= min_silence_for_speedup && silence_end > silence_start {
                    processed.push(TrimSegment {
                        start: silence_start,
                        end: silence_end,
                        speed: speedup_factor,
                    });
                }
                current_pos = silence_end;
            }
        }
    }

    if current_pos < total_duration {
        processed.push(TrimSegment {
            start: current_pos,
            end: total_duration,
            speed: 1.0,
        });
    }

    processed
}

fn create_trim_chunk_dir(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("trim");
    let chunk_dir = parent.join(format!(".dracon-video-{}-{}", stem, std::process::id()));

    if chunk_dir.exists() {
        let _ = std::fs::remove_dir_all(&chunk_dir);
    }
    std::fs::create_dir_all(&chunk_dir)?;
    Ok(chunk_dir)
}

pub fn run_trim_filter_job(
    input: &Path,
    output: &Path,
    segments: &[ProcessedSegment],
) -> Result<()> {
    let (v_filter, a_filter) = generate_trim_filters(segments);

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-filter_complex",
            &format!("{}{}", v_filter, a_filter),
            "-map",
            "[outv]",
            "-map",
            "[outa]",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg trim failed with status: {}", status);
    }

    Ok(())
}

fn concat_chunk_files(chunk_files: &[PathBuf], output: &Path) -> Result<()> {
    if chunk_files.is_empty() {
        anyhow::bail!("No chunk files to concatenate");
    }

    if chunk_files.len() == 1 {
        std::fs::rename(&chunk_files[0], output)?;
        return Ok(());
    }

    let concat_list = output.with_extension("concat.txt");
    let concat_contents = chunk_files
        .iter()
        .map(|path| {
            format!(
                "file '{}'\n",
                path.display().to_string().replace('\'', "'\\''")
            )
        })
        .collect::<String>();
    std::fs::write(&concat_list, concat_contents)?;

    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_list.to_str().context("invalid concat list path")?,
            "-c",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg concat")?;

    let _ = std::fs::remove_file(&concat_list);
    for chunk_file in chunk_files {
        let _ = std::fs::remove_file(chunk_file);
    }

    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
    }

    Ok(())
}

fn generate_trim_filters(segments: &[ProcessedSegment]) -> (String, String) {
    let mut v_filter = String::new();
    let mut a_filter = String::new();
    let mut v_concat = String::new();
    let mut a_concat = String::new();

    for (i, seg) in segments.iter().enumerate() {
        let setpts = if seg.speed != 1.0 {
            format!("setpts={}*PTS", 1.0 / seg.speed)
        } else {
            "setpts=PTS-STARTPTS".to_string()
        };

        let atempo = if seg.speed != 1.0 {
            format!("atempo={}", seg.speed)
        } else {
            "asetpts=PTS-STARTPTS".to_string()
        };

        v_filter.push_str(&format!(
            "[0:v]trim=start={}:end={}, {}[v{}];",
            seg.start, seg.end, setpts, i
        ));
        a_filter.push_str(&format!(
            "[0:a]atrim=start={}:end={}, {}[a{}];",
            seg.start, seg.end, atempo, i
        ));
        v_concat.push_str(&format!("[v{}]", i));
        a_concat.push_str(&format!("[a{}]", i));
    }

    v_filter.push_str(&format!(
        "{}concat=n={}:v=1:a=0[outv];",
        v_concat,
        segments.len()
    ));
    a_filter.push_str(&format!(
        "{}concat=n={}:v=0:a=1[outa]",
        a_concat,
        segments.len()
    ));

    (v_filter, a_filter)
}

pub fn concatenate_videos(
    intro: Option<&Path>,
    main: &Path,
    outro: Option<&Path>,
    output: &Path,
) -> Result<()> {
    use std::io::Write;

    let mut files: Vec<&Path> = Vec::new();

    if let Some(ref i) = intro {
        files.push(i);
    }
    files.push(main);
    if let Some(ref o) = outro {
        files.push(o);
    }

    if files.len() == 1 {
        std::fs::copy(files[0], output)?;
        return Ok(());
    }

    let list_file = output.with_extension("concat.txt");
    {
        let mut f = std::fs::File::create(&list_file)?;
        for file in &files {
            writeln!(
                f,
                "file '{}'",
                file.display().to_string().replace('\'', "'\\''")
            )?;
        }
    }

    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            list_file.to_str().context("invalid concat list path")?,
            "-c",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg concat")?;

    let _ = std::fs::remove_file(&list_file);

    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
    }

    Ok(())
}

fn mix_with_music_impl(
    video: &Path,
    music: &Path,
    output: &Path,
    speech_segments: &[(f32, f32)],
    duck_volume: f32,
) -> Result<()> {
    let duck_filter = generate_duck_filter(speech_segments, duck_volume);

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            video.to_str().context("invalid video path")?,
            "-i",
            music.to_str().context("invalid music path")?,
            "-filter_complex",
            &duck_filter,
            "-map",
            "0:v",
            "-map",
            "[outa]",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg mix_with_music failed with status: {}", status);
    }

    Ok(())
}

fn generate_duck_filter(segments: &[(f32, f32)], duck_volume: f32) -> String {
    let mut volume_expr = "1.0".to_string();

    for &(start, end) in segments {
        volume_expr = format!(
            "if(between(t,{},{}),{},{})",
            start, end, duck_volume, volume_expr
        );
    }

    format!(
        "[1:a]volume=volume='{}'[ducked];[0:a][ducked]amix=inputs=2:duration=first[outa]",
        volume_expr
    )
}
