//! Loudnorm audio filter implementation

use anyhow::{Context, Result};
use std::process::Command;

struct LoudnormStats {
    i: String,
    tp: String,
    lra: String,
    thresh: String,
    offset: String,
}

fn parse_loudnorm_stats(stderr: &str) -> Option<LoudnormStats> {
    let json_start = stderr.find('{')?;
    let json_str = &stderr[json_start..];
    let json_end = json_str.find('}')? + 1;
    let json_str = &json_str[..json_end];

    let get_val = |key: &str| -> Option<String> {
        let pattern = format!("\"{}\":", key);
        let idx = json_str.find(&pattern)?;
        let after = &json_str[idx + pattern.len()..];
        let after = after.trim();
        if let Some(stripped) = after.strip_prefix('"') {
            let end = stripped.find('"')?;
            Some(stripped[..=end].to_string())
        } else {
            let end = after
                .find(|c| [',', '\n', '}'].contains(&c))
                .unwrap_or(after.len());
            Some(after[..end].trim().to_string())
        }
    };

    Some(LoudnormStats {
        i: get_val("input_i")?,
        tp: get_val("input_tp")?,
        lra: get_val("input_lra")?,
        thresh: get_val("input_thresh")?,
        offset: get_val("target_offset")?,
    })
}

pub fn enhance_audio_impl(
    input: &std::path::Path,
    output: &std::path::Path,
    target_lufs: f32,
) -> Result<()> {
    let input_str = input.to_str().context("invalid input path")?;

    // Pass 1: Measure audio loudness
    let measure_filter = format!(
        "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={}:TP=-1.5:LRA=11:print_format=json",
        target_lufs
    );

    let measure_output = Command::new("ffmpeg")
        .args(["-i", input_str, "-af", &measure_filter, "-f", "null", "-"])
        .stderr(std::process::Stdio::piped())
        .output()
        .context("failed to run loudnorm measurement pass")?;

    let stderr = String::from_utf8_lossy(&measure_output.stderr);
    let stats = parse_loudnorm_stats(&stderr);

    // Pass 2: Apply measured normalization
    let filter = if let Some(s) = stats {
        format!(
            "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={}:TP=-1.5:LRA=11:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
            target_lufs, s.i, s.tp, s.lra, s.thresh, s.offset
        )
    } else {
        format!(
            "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={}:TP=-1.5:LRA=11",
            target_lufs
        )
    };

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input_str,
            "-af",
            &filter,
            "-c:v",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg enhance_audio failed with status: {}", status);
    }

    Ok(())
}
