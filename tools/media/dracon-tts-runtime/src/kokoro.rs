use crate::contracts::{Gender, TextToSpeech, VoiceInfo, VoiceProvider};
use anyhow::{Context, Result};
use ort::session::Session;
use rodio::{OutputStream, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

static TTS_COUNTER: AtomicU64 = AtomicU64::new(0);
const KOKORO_SAMPLE_RATE: u32 = 24000;

/// Default Kokoro voice id.
pub const DEFAULT_VOICE: &str = "af_bella";
/// Default Kokoro male voice id.
pub const DEFAULT_MALE_VOICE: &str = "bm_lewis";

/// Built-in Kokoro voice descriptions.
pub const VOICE_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    ("af", "Default", "female"),
    ("af_alloy", "Alloy", "female"),
    ("af_aoede", "Aoede", "female"),
    ("af_bella", "Bella", "female"),
    ("af_heart", "Heart", "female"),
    ("af_jessica", "Jessica", "female"),
    ("af_kore", "Kore", "female"),
    ("af_nicole", "Nicole", "female"),
    ("af_nova", "Nova", "female"),
    ("af_river", "River", "female"),
    ("af_sarah", "Sarah", "female"),
    ("af_sky", "Sky", "female"),
    ("am_adam", "Adam", "male"),
    ("am_echo", "Echo", "male"),
    ("am_eric", "Eric", "male"),
    ("am_fenrir", "Fenrir", "male"),
    ("am_liam", "Liam", "male"),
    ("am_michael", "Michael", "male"),
    ("am_onyx", "Onyx", "male"),
    ("am_puck", "Puck", "male"),
    ("am_santa", "Santa", "male"),
    ("bf_alice", "Alice (British)", "female"),
    ("bf_emma", "Emma (British)", "female"),
    ("bf_isabella", "Isabella (British)", "female"),
    ("bf_lily", "Lily (British)", "female"),
    ("bm_daniel", "Daniel (British)", "male"),
    ("bm_fable", "Fable (British)", "male"),
    ("bm_george", "George (British)", "male"),
    ("bm_lewis", "Lewis (British)", "male"),
    ("ef_dora", "Dora (Emirates)", "female"),
    ("em_alex", "Alex (Emirates)", "male"),
    ("em_santa", "Santa (Emirates)", "male"),
    ("ff_siwis", "Siwis (French)", "female"),
    ("hf_alpha", "Alpha (Hindi)", "female"),
    ("hf_beta", "Beta (Hindi)", "female"),
    ("hm_omega", "Omega (Hindi)", "male"),
    ("hm_psi", "Psi (Hindi)", "male"),
    ("if_sara", "Sara (Indian)", "female"),
    ("im_nicola", "Nicola (Indian)", "male"),
    ("jf_alpha", "Alpha (Japanese)", "female"),
    ("jf_gongitsune", "Gongitsune (Japanese)", "female"),
    ("jf_nezumi", "Nezumi (Japanese)", "female"),
    ("jf_tebukuro", "Tebukuro (Japanese)", "female"),
    ("jm_kumo", "Kumo (Japanese)", "male"),
    ("pf_dora", "Dora (Portuguese)", "female"),
    ("pm_alex", "Alex (Portuguese)", "male"),
    ("pm_santa", "Santa (Portuguese)", "male"),
    ("zf_xiaobei", "Xiaobei (Chinese)", "female"),
    ("zf_xiaoni", "Xiaoni (Chinese)", "female"),
    ("zf_xiaoxiao", "Xiaoxiao (Chinese)", "female"),
    ("zf_xiaoyi", "Xiaoyi (Chinese)", "female"),
    ("zm_yunjian", "Yunjian (Chinese)", "male"),
    ("zm_yunxi", "Yunxi (Chinese)", "male"),
    ("zm_yunxia", "Yunxia (Chinese)", "male"),
    ("zm_yunyang", "Yunyang (Chinese)", "male"),
];

/// Resolves a voice name or alias (e.g., "af_heart", "female", "male") to its internal name.
/// Returns the default voice if no match is found.
pub fn resolve_voice(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();

    for (internal, friendly, _gender) in VOICE_DESCRIPTIONS {
        if name_lower == internal.to_lowercase() || name_lower == friendly.to_lowercase() {
            return internal;
        }
    }

    if name_lower == "male" || name_lower == "m" {
        return DEFAULT_MALE_VOICE;
    }
    if name_lower == "female" || name_lower == "f" {
        return DEFAULT_VOICE;
    }

    DEFAULT_VOICE
}

/// Returns (internal_name, friendly_name, gender) for a voice, or ("unknown", "Unknown", "unknown").
pub fn voice_info(name: &str) -> (&'static str, &'static str, &'static str) {
    for info in VOICE_DESCRIPTIONS {
        if info.0 == name {
            return *info;
        }
    }
    ("unknown", "Unknown", "unknown")
}

/// Kokoro text-to-speech backend.
#[non_exhaustive]
pub struct KokoroTts {
    sink: Arc<Sink>,
    /// Atomic flag indicating whether playback is active.
    pub speaking: Arc<AtomicBool>,
    active_playbacks: Arc<AtomicUsize>,
    queue_end_at: Arc<Mutex<Option<Instant>>>,
    session: Option<Arc<Mutex<Session>>>,
    voices: HashMap<String, Vec<f32>>,
    current_voice: Arc<Mutex<String>>,
    output_channels: u16,
    queue_debug: bool,
    chunk_dump_dir: Option<PathBuf>,
}

impl KokoroTts {
    /// Create a Kokoro backend with the default voice.
    pub async fn new(model_path: &str, voices_dir: &str) -> Result<Self> {
        Self::new_with_voice(model_path, voices_dir, DEFAULT_VOICE).await
    }

    /// Create a Kokoro backend with an explicit voice.
    pub async fn new_with_voice(
        model_path: &str,
        voices_dir: &str,
        voice: &str,
    ) -> anyhow::Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().context("failed to initialize audio output")?;
        let sink = Arc::new(Sink::try_new(&handle).context("failed to create audio sink")?);
        let sink_volume = std::env::var("REMI_TTS_SINK_VOLUME")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0)
            .clamp(0.0, 2.0);
        sink.set_volume(sink_volume);
        sink.play();
        std::mem::forget(stream);

        let output_channels = if std::env::var_os("REMI_TTS_FORCE_STEREO").is_some() {
            2
        } else {
            1
        };

        let queue_debug = std::env::var_os("REMI_TTS_QUEUE_DEBUG").is_some();

        let chunk_dump_dir = std::env::var("REMI_TTS_DUMP_CHUNKS_DIR")
            .ok()
            .map(PathBuf::from);

        let session = if std::path::Path::new(model_path).exists() {
            match Session::builder() {
                Ok(builder) => match builder.with_intra_threads(4) {
                    Ok(mut builder) => match builder.commit_from_file(model_path) {
                        Ok(s) => Some(Arc::new(Mutex::new(s))),
                        Err(e) => {
                            return Err(anyhow::anyhow!("failed to load Kokoro ONNX model: {}", e));
                        }
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "failed to configure Kokoro session builder: {}",
                            e
                        ));
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("failed to create session builder: {}", e));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Kokoro model not found at: {}", model_path));
        };

        let voices = Self::load_voices_from_dir(voices_dir);

        let resolved_voice = resolve_voice(voice);
        let current_voice = if voices.contains_key(resolved_voice) {
            resolved_voice.to_string()
        } else {
            voices
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| DEFAULT_VOICE.to_string())
        };

        Ok(Self {
            sink,
            speaking: Arc::new(AtomicBool::new(false)),
            active_playbacks: Arc::new(AtomicUsize::new(0)),
            queue_end_at: Arc::new(Mutex::new(None)),
            session,
            voices,
            current_voice: Arc::new(Mutex::new(current_voice)),
            output_channels,
            queue_debug,
            chunk_dump_dir,
        })
    }

    fn load_voices_from_dir(dir: &str) -> HashMap<String, Vec<f32>> {
        let mut voices = HashMap::new();

        let dirs_to_check = vec![dir.to_string(), format!("{}/voices", dir)];

        for dir in dirs_to_check {
            if !std::path::Path::new(&dir).exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "bin") {
                        if let Some(name) = path.file_stem() {
                            let voice_name = name.to_string_lossy().to_string();
                            let prefix = voice_name.split('_').next().unwrap_or("");
                            if [
                                "af", "am", "bf", "bm", "ef", "em", "ff", "hf", "hm", "if", "im",
                                "jf", "pf", "pm", "zf", "zm",
                            ]
                            .contains(&prefix)
                            {
                                use std::collections::hash_map::Entry;

                                if let Entry::Vacant(entry) = voices.entry(voice_name) {
                                    match Self::load_voice(&path.to_string_lossy()) {
                                        Ok(v) => {
                                            entry.insert(v);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Failed to load voice {}: {}",
                                                path.display(),
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        voices
    }

    fn load_voice(path: &str) -> Result<Vec<f32>, std::io::Error> {
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;
        if !file_size.is_multiple_of(std::mem::size_of::<f32>()) {
            return Err(std::io::Error::other(format!(
                "voice file {} size {} is not aligned to 4 bytes",
                path, file_size
            )));
        }
        let num_floats = file_size / std::mem::size_of::<f32>();
        let mut data = vec![0.0f32; num_floats];
        // SAFETY: `data` is initialized with `num_floats` f32 values, and
        // `file_size` was checked to be exactly `num_floats * size_of::<f32>()`.
        // The mutable byte slice is used only for this call and is dropped before `data` is returned.
        unsafe {
            std::io::Read::read_exact(
                &mut file,
                std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, file_size),
            )?;
        }
        Ok(data)
    }

    /// Select a loaded voice by id.
    pub fn set_voice(&self, voice: &str) -> anyhow::Result<bool> {
        let resolved = resolve_voice(voice);
        if self.voices.contains_key(resolved) {
            let mut current = self
                .current_voice
                .lock()
                .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))?;
            *current = resolved.to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Return the currently selected voice id.
    pub fn get_voice(&self) -> anyhow::Result<String> {
        self.current_voice
            .lock()
            .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))
            .map(|guard| guard.clone())
    }

    fn begin_playback(&self) {
        if self.active_playbacks.fetch_add(1, Ordering::SeqCst) == 0 {
            self.speaking.store(true, Ordering::SeqCst);
        }
    }

    fn finish_playback_state(active_playbacks: &AtomicUsize, speaking: &AtomicBool) {
        loop {
            let current = active_playbacks.load(Ordering::SeqCst);
            if current == 0 {
                speaking.store(false, Ordering::SeqCst);
                return;
            }
            if active_playbacks
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                if current == 1 {
                    speaking.store(false, Ordering::SeqCst);
                }
                return;
            }
        }
    }

    fn end_playback(&self) {
        Self::finish_playback_state(self.active_playbacks.as_ref(), self.speaking.as_ref());
    }

    fn audio_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn audio_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    fn trim_silence(mut samples: Vec<f32>) -> (Vec<f32>, usize) {
        if samples.is_empty() {
            return (samples, 0);
        }

        let threshold = 0.005;
        let first_sound = samples
            .iter()
            .position(|&s| s.abs() > threshold)
            .unwrap_or(0);
        let last_sound = samples
            .iter()
            .rposition(|&s| s.abs() > threshold)
            .unwrap_or(samples.len() - 1);

        let trim_start = first_sound.saturating_sub(50);
        let trim_end = (last_sound + 50).min(samples.len());

        if trim_start < trim_end && trim_end - trim_start > 1000 {
            samples = samples[trim_start..trim_end].to_vec();
        }

        (samples, trim_start)
    }

    fn expand_channels(samples: Vec<f32>, channels: u16) -> Vec<f32> {
        if channels <= 1 {
            return samples;
        }

        let mut expanded = Vec::with_capacity(samples.len() * channels as usize);
        for sample in samples {
            for _ in 0..channels {
                expanded.push(sample);
            }
        }
        expanded
    }

    fn track_queue_append(&self, call_id: u64, frame_count: usize) {
        if !self.queue_debug {
            return;
        }

        let now = Instant::now();
        let chunk_duration =
            Duration::from_secs_f64(frame_count as f64 / KOKORO_SAMPLE_RATE as f64);

        let mut queue_end = match self.queue_end_at.lock() {
            Ok(guard) => guard,
            Err(_) => {
                eprintln!("[Kokoro-{call_id}][queue] mutex poisoned");
                return;
            }
        };
        let previous_end = *queue_end;

        let lead_before_ms = previous_end
            .map(|end| end.saturating_duration_since(now).as_millis() as u64)
            .unwrap_or(0);
        let gap_ms = previous_end
            .map(|end| now.saturating_duration_since(end).as_millis() as u64)
            .unwrap_or(0);

        let scheduled_start = previous_end
            .map(|end| if end > now { end } else { now })
            .unwrap_or(now);
        let new_end = scheduled_start + chunk_duration;
        *queue_end = Some(new_end);

        let lead_after_ms = new_end.saturating_duration_since(now).as_millis() as u64;
        let underrun_note = if gap_ms > 0 { " UNDERRUN" } else { "" };
        println!(
            "[Kokoro-{}][queue] lead_before={}ms gap={}ms chunk={}ms lead_after={}ms{}",
            call_id,
            lead_before_ms,
            gap_ms,
            chunk_duration.as_millis(),
            lead_after_ms,
            underrun_note
        );
    }

    fn maybe_dump_chunk_wav(&self, call_id: u64, samples: &[f32]) {
        let Some(dir) = &self.chunk_dump_dir else {
            return;
        };

        let path = dir.join(format!("kokoro_chunk_{:04}.wav", call_id));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: KOKORO_SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = match hound::WavWriter::create(&path, spec) {
            Ok(w) => w,
            Err(e) => {
                eprintln!(
                    "[Kokoro-{}] Failed to create chunk dump {}: {}",
                    call_id,
                    path.display(),
                    e
                );
                return;
            }
        };

        for sample in samples {
            let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            if let Err(e) = writer.write_sample(pcm) {
                eprintln!(
                    "[Kokoro-{}] Failed writing chunk dump {}: {}",
                    call_id,
                    path.display(),
                    e
                );
                return;
            }
        }

        if let Err(e) = writer.finalize() {
            eprintln!(
                "[Kokoro-{}] Failed finalizing chunk dump {}: {}",
                call_id,
                path.display(),
                e
            );
        }
    }

    fn process_samples_for_playback(
        &self,
        call_id: u64,
        samples: Vec<f32>,
    ) -> Option<(Vec<f32>, usize)> {
        if samples.is_empty() {
            return None;
        }

        let raw_peak = Self::audio_peak(&samples);
        let raw_rms = Self::audio_rms(&samples);
        println!(
            "[Kokoro-{}] Raw peak={:.3} rms={:.3}, {} samples",
            call_id,
            raw_peak,
            raw_rms,
            samples.len()
        );

        let (processed, trim_start) = Self::trim_silence(samples);
        if processed.is_empty() {
            return None;
        }

        if std::env::var_os("REMI_TTS_LOUDNESS_DEBUG").is_some() {
            let chunk_ms = (processed.len() as f32 / 24000.0) * 1000.0;
            let post_peak = Self::audio_peak(&processed);
            let post_rms = Self::audio_rms(&processed);
            println!(
                "[Kokoro-{}][raw] chunk_ms={:.1} post_peak={:.3} post_rms={:.3}",
                call_id, chunk_ms, post_peak, post_rms
            );
        }

        Some((processed, trim_start))
    }

    fn text_to_phonemes(text: &str) -> anyhow::Result<Vec<i64>> {
        let mut child = std::process::Command::new("espeak-ng")
            .args(["--ipa", "-q", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("failed to spawn espeak-ng for phoneme extraction")?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(text.as_bytes())
                .context("failed to write text to espeak-ng stdin")?;
        } else {
            anyhow::bail!("failed to open espeak-ng stdin");
        }

        let output = child
            .wait_with_output()
            .context("failed to wait for espeak-ng")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            anyhow::bail!(
                "espeak-ng failed with exit code {:?}: {stderr}",
                output.status.code()
            );
        }
        let phonemes =
            String::from_utf8(output.stdout).context("espeak-ng returned non-UTF8 phonemes")?;

        let mut tokens = vec![0i64];

        for c in phonemes.chars() {
            let id = Self::phoneme_to_token(c);
            tokens.push(id);
        }

        tokens.push(0);
        Ok(tokens)
    }

    fn phoneme_to_token(c: char) -> i64 {
        match c {
            ';' => 1,
            ':' => 2,
            ',' => 3,
            '.' => 4,
            '!' => 5,
            '?' => 6,
            '—' => 9,
            '…' => 10,
            '"' => 11,
            '(' => 12,
            ')' => 13,
            ' ' => 16,
            'A' => 24,
            'I' => 25,
            'O' => 31,
            'Q' => 33,
            'S' => 35,
            'T' => 36,
            'W' => 39,
            'Y' => 41,
            'a' => 43,
            'b' => 44,
            'c' => 45,
            'd' => 46,
            'e' => 47,
            'f' => 48,
            'h' => 50,
            'i' => 51,
            'j' => 52,
            'k' => 53,
            'l' => 54,
            'm' => 55,
            'n' => 56,
            'o' => 57,
            'p' => 58,
            'q' => 59,
            'r' => 60,
            's' => 61,
            't' => 62,
            'u' => 63,
            'v' => 64,
            'w' => 65,
            'x' => 66,
            'y' => 67,
            'z' => 68,
            'ɑ' => 69,
            'ɐ' => 70,
            'ɒ' => 71,
            'æ' => 72,
            'ɔ' => 76,
            'ð' => 81,
            'ə' => 83,
            'ɚ' => 85,
            'ɛ' => 86,
            'ɜ' => 87,
            'ɡ' => 92,
            'ɪ' => 102,
            'ŋ' => 112,
            'θ' => 119,
            'ɹ' => 123,
            'ʃ' => 131,
            'ʊ' => 135,
            'ʌ' => 138,
            'ʒ' => 147,
            'ˈ' => 156,
            'ˌ' => 157,
            'ː' => 158,
            'ʰ' => 162,
            'ʲ' => 164,
            _ => 0,
        }
    }

    /// Synthesize and play text using the current voice at inference speed.
    pub async fn speak_impl(&self, text: &str) {
        self.speak_impl_with_speed(text, 1.0).await;
    }

    /// Synthesize and play text with a model-inference speed factor.
    /// Unlike post-processing atempo, this affects the model's prosody directly.
    pub async fn speak_impl_with_speed(&self, text: &str, speed: f32) {
        let voice = self
            .get_voice()
            .unwrap_or_else(|_| DEFAULT_VOICE.to_string());
        let call_id = TTS_COUNTER.fetch_add(1, Ordering::SeqCst);
        let total_start = std::time::Instant::now();
        println!(
            "\n[Kokoro-{}] speak: \"{}\" (voice: {})",
            call_id, text, voice
        );

        if self.session.is_none() || self.voices.is_empty() {
            eprintln!("[Kokoro-{}] Not initialized", call_id);
            return;
        }

        self.begin_playback();

        let Some(session) = self.session.as_ref() else {
            return;
        };
        let session = session.clone();
        let sink = self.sink.clone();
        let voice_data = self
            .voices
            .get(&voice)
            .cloned()
            .unwrap_or_else(|| self.voices.values().next().cloned().unwrap_or_default());
        let text = text.to_string();

        let result = async {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<f32>> {
                let token_start = std::time::Instant::now();
                let tokens = Self::text_to_phonemes(&text)?;
                let token_time = token_start.elapsed();
                println!(
                    "[Kokoro-{}] Tokens: {} ({:.1}ms)",
                    call_id,
                    tokens.len(),
                    token_time.as_secs_f64() * 1000.0
                );

                if tokens.len() < 2 {
                    return Ok(Vec::new());
                }

                let input_arr = ndarray::Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;

                let style_dim = 256;
                // Voice data is a 2D array laid out as
                // [style_for_0_phonemes, style_for_1_phoneme, ...] each
                // style_dim floats wide. Pick the style for this batch's
                // phoneme count (matches the reference kokoro-onnx impl).
                let max_idx = voice_data.len() / style_dim;
                let idx = (tokens.len()).min(max_idx.saturating_sub(1));
                let style: Vec<f32> = if voice_data.len() >= style_dim {
                    voice_data[idx * style_dim..(idx + 1) * style_dim].to_vec()
                } else {
                    vec![0.0; style_dim]
                };
                let style_arr = ndarray::Array2::from_shape_vec((1, style_dim), style)?;

                let speed_arr = ndarray::Array1::from_vec(vec![speed]);

                let inputs = ort::inputs![
                    "tokens" => ort::value::Value::from_array(input_arr)?,
                    "style" => ort::value::Value::from_array(style_arr)?,
                    "speed" => ort::value::Value::from_array(speed_arr)?,
                ];

                let infer_start = std::time::Instant::now();
                let mut sess = session
                    .lock()
                    .map_err(|_| anyhow::anyhow!("mutex poisoned"))?;
                let outputs = sess.run(inputs)?;
                let infer_time = infer_start.elapsed();

                let result = if let Some(output) = outputs.values().next() {
                    let tensor = output.try_extract_tensor::<f32>()?;
                    tensor.1.to_vec()
                } else {
                    Vec::new()
                };
                println!(
                    "[Kokoro-{}] ONNX inference: {:.1}ms, {} samples",
                    call_id,
                    infer_time.as_secs_f64() * 1000.0,
                    result.len()
                );
                Ok(result)
            })
            .await?
        }
        .await;

        let total_time = total_start.elapsed();
        match result {
            Ok(samples) => {
                if let Some((samples, trim_start)) =
                    self.process_samples_for_playback(call_id, samples)
                {
                    let sample_count = samples.len();
                    self.track_queue_append(call_id, sample_count);
                    self.maybe_dump_chunk_wav(call_id, &samples);
                    let output_samples = Self::expand_channels(samples, self.output_channels);
                    let source = rodio::buffer::SamplesBuffer::new(
                        self.output_channels,
                        KOKORO_SAMPLE_RATE,
                        output_samples,
                    );
                    sink.append(source.convert_samples::<f32>());
                    println!(
                        "[Kokoro-{}] Total: {:.1}ms, Playing {} samples (trimmed {} leading, raw audio)",
                        call_id,
                        total_time.as_secs_f64() * 1000.0,
                        sample_count,
                        trim_start
                    );
                    sink.sleep_until_end();
                }
            }
            Err(e) => {
                eprintln!("[Kokoro-{}] Task error: {:?}", call_id, e);
            }
        }
        self.end_playback();
    }

    /// Queue speech without waiting for playback to finish.
    pub async fn speak_nowait(&self, text: &str) {
        self.speak_nowait_with_speed(text, 1.0).await;
    }

    /// Queue speech with a model-inference speed factor.
    pub async fn speak_nowait_with_speed(&self, text: &str, speed: f32) {
        let voice = self
            .get_voice()
            .unwrap_or_else(|_| DEFAULT_VOICE.to_string());
        let call_id = TTS_COUNTER.fetch_add(1, Ordering::SeqCst);
        println!(
            "\n[Kokoro-{}] speak_nowait: \"{}\" (voice: {}, speed: {})",
            call_id, text, voice, speed
        );

        if self.session.is_none() || self.voices.is_empty() {
            eprintln!("[Kokoro-{}] Not initialized", call_id);
            return;
        }

        self.begin_playback();

        let Some(session) = self.session.as_ref() else {
            return;
        };
        let session = session.clone();
        let sink = self.sink.clone();
        let voice_data = self
            .voices
            .get(&voice)
            .cloned()
            .unwrap_or_else(|| self.voices.values().next().cloned().unwrap_or_default());
        let text = text.to_string();

        let result = async {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<f32>> {
                let tokens = Self::text_to_phonemes(&text)?;
                if tokens.len() < 2 {
                    return Ok(Vec::new());
                }

                let tokens_len = tokens.len();
                let input_arr = ndarray::Array2::from_shape_vec((1, tokens_len), tokens)?;

                let style_dim = 256;
                // Voice data is a 2D array laid out as
                // [style_for_0_phonemes, style_for_1_phoneme, ...] each
                // style_dim floats wide. Pick the style for this batch's
                // phoneme count (matches the reference kokoro-onnx impl).
                let max_idx = voice_data.len() / style_dim;
                let idx = (tokens_len).min(max_idx.saturating_sub(1));
                let style: Vec<f32> = if voice_data.len() >= style_dim {
                    voice_data[idx * style_dim..(idx + 1) * style_dim].to_vec()
                } else {
                    vec![0.0; style_dim]
                };
                let style_arr = ndarray::Array2::from_shape_vec((1, style_dim), style)?;
                let speed_arr = ndarray::Array1::from_vec(vec![speed]);

                let inputs = ort::inputs![
                    "tokens" => ort::value::Value::from_array(input_arr)?,
                    "style" => ort::value::Value::from_array(style_arr)?,
                    "speed" => ort::value::Value::from_array(speed_arr)?,
                ];

                let mut sess = session
                    .lock()
                    .map_err(|_| anyhow::anyhow!("mutex poisoned"))?;
                let outputs = sess.run(inputs)?;

                if let Some(output) = outputs.values().next() {
                    if let Ok(tensor) = output.try_extract_tensor::<f32>() {
                        return Ok(tensor.1.to_vec());
                    }
                }
                Ok(Vec::new())
            })
            .await?
        }
        .await;

        match result {
            Ok(samples) => {
                if let Some((samples, trim_start)) =
                    self.process_samples_for_playback(call_id, samples)
                {
                    let sample_count = samples.len();
                    self.track_queue_append(call_id, sample_count);
                    self.maybe_dump_chunk_wav(call_id, &samples);
                    let output_samples = Self::expand_channels(samples, self.output_channels);
                    let source = rodio::buffer::SamplesBuffer::new(
                        self.output_channels,
                        KOKORO_SAMPLE_RATE,
                        output_samples,
                    );
                    sink.append(source.convert_samples::<f32>());
                    println!(
                        "[Kokoro-{}] Playing {} samples (trimmed {} leading, raw audio)",
                        call_id, sample_count, trim_start
                    );
                }
            }
            Err(e) => {
                eprintln!("[Kokoro-{}] Task error: {:?}", call_id, e);
            }
        }

        let sink_for_wait = sink.clone();
        let active = self.active_playbacks.clone();
        let speaking = self.speaking.clone();
        drop(tokio::task::spawn_blocking(move || {
            sink_for_wait.sleep_until_end();
            Self::finish_playback_state(active.as_ref(), speaking.as_ref());
        }));
    }

    /// Block until active playback finishes.
    pub async fn wait_until_done(&self) {
        loop {
            if self.active_playbacks.load(Ordering::SeqCst) == 0 && self.sink.empty() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

impl TextToSpeech for KokoroTts {
    fn speak(&self, text: &str) -> anyhow::Result<()> {
        futures::executor::block_on(self.speak_impl(text));
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.speaking.store(false, Ordering::SeqCst);
        self.active_playbacks.store(0, Ordering::SeqCst);
        *self
            .queue_end_at
            .lock()
            .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))? = None;
        self.sink.stop();
        self.sink.clear();
        self.sink.play();
        Ok(())
    }

    fn is_speaking(&self) -> bool {
        self.speaking.load(Ordering::SeqCst)
            || self.active_playbacks.load(Ordering::SeqCst) > 0
            || !self.sink.empty()
    }

    fn name(&self) -> &'static str {
        "Kokoro"
    }

    fn sample_rate(&self) -> u32 {
        KOKORO_SAMPLE_RATE
    }
}

impl VoiceProvider for KokoroTts {
    fn list_voices(&self) -> Vec<VoiceInfo> {
        VOICE_DESCRIPTIONS
            .iter()
            .filter(|(id, _, _)| self.voices.contains_key(*id))
            .map(|(id, name, gender)| VoiceInfo {
                id: id.to_string(),
                name: name.to_string(),
                gender: if *gender == "male" {
                    Gender::Male
                } else {
                    Gender::Female
                },
                language: None,
                description: None,
            })
            .collect()
    }

    fn set_voice(&self, voice: &str) -> anyhow::Result<bool> {
        let resolved = resolve_voice(voice);
        if self.voices.contains_key(resolved) {
            let mut current = self
                .current_voice
                .lock()
                .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))?;
            *current = resolved.to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn current_voice(&self) -> anyhow::Result<VoiceInfo> {
        let voice_id = self
            .current_voice
            .lock()
            .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))?
            .clone();
        for (id, name, gender) in VOICE_DESCRIPTIONS {
            if *id == voice_id {
                return Ok(VoiceInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    gender: if *gender == "male" {
                        Gender::Male
                    } else {
                        Gender::Female
                    },
                    language: None,
                    description: None,
                });
            }
        }
        Ok(VoiceInfo::new(DEFAULT_VOICE, "Heart", Gender::Female))
    }
}

#[cfg(test)]
mod tests {
    use super::KokoroTts;

    #[test]
    fn trim_silence_keeps_audio_body() {
        let mut samples = vec![0.0; 300];
        samples.extend(vec![0.02; 2000]);
        samples.extend(vec![0.0; 200]);

        let (trimmed, trim_start) = KokoroTts::trim_silence(samples);
        assert!(trim_start > 0);
        assert!(trimmed.len() > 1500);
    }
}
