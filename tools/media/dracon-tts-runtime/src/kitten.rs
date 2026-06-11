use crate::contracts::{Gender, TextToSpeech, VoiceInfo, VoiceProvider};
use anyhow::Context;
use anyhow::Result;
use ort::session::Session;
use rodio::{OutputStream, Sink, Source};
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType};
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

static KITTEN_COUNTER: AtomicU64 = AtomicU64::new(0);
const KITTEN_SAMPLE_RATE: usize = 24000;
const OUTPUT_SAMPLE_RATE: usize = 24000; // Most audio devices use 48kHz

/// Default Kitten voice id.

/// Runtime state for the Kitten backend.
#[derive(Debug, Clone, PartialEq)]
pub enum KittenState {
    /// Backend is idle.
    Idle,
    /// Backend is currently speaking.
    Speaking,
    /// Backend encountered an error.
    Error(String),
}

/// Kitten text-to-speech backend.
pub struct KittenTTS {
    sink: Arc<Sink>,
    state: Arc<AsyncMutex<KittenState>>,
    speaking: Arc<AtomicBool>,
    session: Option<Arc<Mutex<Session>>>,
    voices: HashMap<String, Vec<f32>>,
    current_voice: Arc<Mutex<String>>,
}

/// Default Kitten voice id.
pub const DEFAULT_VOICE: &str = "expr-voice-3-m"; // Bruno
/// Default female voice id.
pub const DEFAULT_FEMALE_VOICE: &str = "expr-voice-5-f"; // Kiki
/// Default model key.
pub const DEFAULT_MODEL: &str = "nano";

/// Kitten model file configuration.
#[derive(Debug, Clone, Copy)]
pub struct KittenModelConfig {
    /// Stable model key.
    pub key: &'static str,
    /// Human-readable model label.
    pub label: &'static str,
    /// ONNX model path.
    pub model_path: &'static str,
    /// Voice data path.
    pub voices_path: &'static str,
    /// Optional alternate voice data path.
    pub fallback_voices_path: Option<&'static str>,
    /// Human-readable note about this model.
    pub note: &'static str,
}

/// Built-in Kitten model descriptions.
pub const MODEL_DESCRIPTIONS: &[KittenModelConfig] = &[
    KittenModelConfig {
        key: "nano",
        label: "Nano",
        model_path: "assets/models/kitten_tts_nano_v0_8.onnx",
        voices_path: "assets/models/voices_v0_8.npz",
        fallback_voices_path: Some("assets/models/kitten_voices.npz"),
        note: "Default. Best balance of quality and latency.",
    },
    KittenModelConfig {
        key: "micro",
        label: "Micro",
        model_path: "assets/models/kitten_micro.onnx",
        voices_path: "assets/models/kitten_voices.npz",
        fallback_voices_path: Some("assets/models/voices_v0_8.npz"),
        note: "Fastest inference; lower voice quality.",
    },
    KittenModelConfig {
        key: "int8",
        label: "Nano INT8 (experimental)",
        model_path: "assets/models/kitten-nano-int8.onnx",
        voices_path: "assets/models/kitten_voices.npz",
        fallback_voices_path: Some("assets/models/voices_v0_8.npz"),
        note: "Experimental; may sound unstable on some systems.",
    },
];

/// Built-in Kitten voice descriptions.
pub const VOICE_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    // (internal_name, friendly_name, gender) - v0.8 aliases
    ("expr-voice-2-m", "Jasper", "male"),
    ("expr-voice-2-f", "Bella", "female"),
    ("expr-voice-3-m", "Bruno", "male"),
    ("expr-voice-3-f", "Luna", "female"),
    ("expr-voice-4-m", "Hugo", "male"),
    ("expr-voice-4-f", "Rosie", "female"),
    ("expr-voice-5-m", "Leo", "male"),
    ("expr-voice-5-f", "Kiki", "female"),
];

/// Resolve a Kitten model key from an alias.
pub fn resolve_model(name: &str) -> &'static str {
    let name_lower = name.trim().to_ascii_lowercase();
    match name_lower.as_str() {
        "" | "nano" => "nano",
        "mini" => "nano", // Mini is currently an alias to Nano in this repository.
        "micro" => "micro",
        "int8" | "nano-int8" | "nano_int8" => {
            eprintln!("WARNING: int8 model is unstable and causes static. Falling back to nano.");
            "nano"
        }
        _ => DEFAULT_MODEL,
    }
}

/// Return model configuration for a model key or alias.
pub fn model_info(name: &str) -> KittenModelConfig {
    let model = resolve_model(name);
    for info in MODEL_DESCRIPTIONS {
        if info.key == model {
            return *info;
        }
    }
    MODEL_DESCRIPTIONS[0]
}

/// Return existing model and voice asset paths for a model key or alias.
pub fn model_paths(name: &str) -> (&'static str, &'static str) {
    let info = model_info(name);
    if std::path::Path::new(info.voices_path).exists() {
        return (info.model_path, info.voices_path);
    }
    if let Some(fallback_path) = info.fallback_voices_path {
        if std::path::Path::new(fallback_path).exists() {
            return (info.model_path, fallback_path);
        }
    }
    (info.model_path, info.voices_path)
}

/// Resolve a voice id from an internal name, friendly name, or gender alias.
pub fn resolve_voice(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();

    for (internal, friendly, _gender) in VOICE_DESCRIPTIONS {
        if name_lower == internal.to_lowercase() || name_lower == friendly.to_lowercase() {
            return internal;
        }
    }

    if name_lower == "male" || name_lower == "m" {
        return DEFAULT_VOICE;
    }
    if name_lower == "female" || name_lower == "f" {
        return DEFAULT_FEMALE_VOICE;
    }

    DEFAULT_VOICE
}

/// Return voice metadata for a voice id.
pub fn voice_info(name: &str) -> (&'static str, &'static str, &'static str) {
    for info in VOICE_DESCRIPTIONS {
        if info.0 == name {
            return *info;
        }
    }
    ("unknown", "Unknown", "unknown")
}

impl KittenTTS {
    /// Create a Kitten backend with the default voice.
    pub async fn new(model_path: &str, voices_path: &str) -> Result<Self> {
        Self::new_with_voice(model_path, voices_path, DEFAULT_VOICE).await
    }

    /// Create a Kitten backend with an explicit voice.
    pub async fn new_with_voice(model_path: &str, voices_path: &str, voice: &str) -> Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().context("failed to initialize audio output")?;
        let sink = Arc::new(Sink::try_new(&handle).context("failed to create audio sink")?);
        sink.set_volume(1.0);
        sink.play();
        std::mem::forget(stream);

        let session = if std::path::Path::new(model_path).exists() {
            match Session::builder() {
                Ok(builder) => match builder
                    .with_intra_threads(4)
                    .and_then(|b| b.commit_from_file(model_path))
                {
                    Ok(s) => Some(Arc::new(Mutex::new(s))),
                    Err(e) => {
                        return Err(anyhow::anyhow!("failed to load Kitten ONNX model: {}", e));
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("failed to create session builder: {}", e));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Kitten model not found at: {}", model_path));
        };

        let voices = if std::path::Path::new(voices_path).exists() {
            Self::load_voices_npz(voices_path).context("failed to load Kitten voices")?
        } else {
            return Err(anyhow::anyhow!(
                "Kitten voices not found at: {}",
                voices_path
            ));
        };

        let current_voice = if voices.contains_key(voice) {
            voice.to_string()
        } else {
            DEFAULT_VOICE.to_string()
        };

        Ok(Self {
            sink,
            state: Arc::new(AsyncMutex::new(KittenState::Idle)),
            speaking: Arc::new(AtomicBool::new(false)),
            session,
            voices,
            current_voice: Arc::new(Mutex::new(current_voice)),
        })
    }

    /// Select a loaded voice by id.
    pub fn set_voice(&self, voice: &str) -> Result<bool> {
        if self.voices.contains_key(voice) {
            let mut current = self
                .current_voice
                .lock()
                .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))?;
            *current = voice.to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Return the currently selected voice id.
    pub fn get_voice(&self) -> Result<String> {
        self.current_voice
            .lock()
            .map_err(|e| anyhow::anyhow!("mutex poisoned: {}", e))
            .map(|guard| guard.clone())
    }

    /// Return the friendly description for a voice id.
    pub fn voice_description(voice: &str) -> &'static str {
        voice_info(voice).1
    }

    fn audio_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn trim_silence(mut samples: Vec<f32>) -> (Vec<f32>, usize) {
        if samples.is_empty() {
            return (samples, 0);
        }

        let threshold = 0.004;
        let first_sound = samples
            .iter()
            .position(|&s| s.abs() > threshold)
            .unwrap_or(0);
        let last_sound = samples
            .iter()
            .rposition(|&s| s.abs() > threshold)
            .unwrap_or(samples.len() - 1);

        let trim_start = first_sound.saturating_sub(80);
        // Add 2400 samples (100ms at 24kHz) of trailing padding for natural decay
        let trim_end = (last_sound + 2400).min(samples.len());
        if trim_start < trim_end && trim_end - trim_start > 1000 {
            samples = samples[trim_start..trim_end].to_vec();
        }

        (samples, trim_start)
    }

    fn apply_compression(samples: &mut [f32], threshold: f32, ratio: f32) {
        for sample in samples {
            let abs = sample.abs();
            if abs > threshold {
                let sign = sample.signum();
                *sample = sign * (threshold + (abs - threshold) / ratio);
            }
        }
    }

    fn apply_soft_limiter(samples: &mut [f32], ceiling: f32) {
        if ceiling <= 0.0 {
            return;
        }
        for sample in samples {
            let x = *sample / ceiling;
            *sample = ceiling * x.tanh();
        }
    }

    fn apply_fade(samples: &mut [f32], fade_samples: usize) {
        let len = samples.len();
        for i in 0..fade_samples.min(len) {
            let factor = i as f32 / fade_samples as f32;
            samples[i] *= factor;
            samples[len - 1 - i] *= factor;
        }
    }

    /// Resample audio from KITTEN_SAMPLE_RATE (24000 Hz) to OUTPUT_SAMPLE_RATE (48000 Hz)
    /// This fixes the "machine noise" issue when audio devices expect 48kHz
    fn resample_to_output_rate(samples: &[f32]) -> Vec<f32> {
        if KITTEN_SAMPLE_RATE == OUTPUT_SAMPLE_RATE {
            return samples.to_vec();
        }

        let resample_ratio = OUTPUT_SAMPLE_RATE as f64 / KITTEN_SAMPLE_RATE as f64;

        // Use simple linear interpolation for 2x upsampling (24000 -> 48000)
        // This is efficient and high quality for integer ratios
        if resample_ratio == 2.0 {
            let mut resampled = Vec::with_capacity(samples.len() * 2);
            for i in 0..samples.len() {
                let current = samples[i];
                let next = samples.get(i + 1).copied().unwrap_or(current);
                // Linear interpolation: insert sample halfway between
                resampled.push(current);
                resampled.push((current + next) * 0.5);
            }
            resampled
        } else {
            // For non-integer ratios, use rubato
            let params = SincInterpolationParameters {
                sinc_len: 256,
                f_cutoff: 0.95,
                interpolation: SincInterpolationType::Linear,
                oversampling_factor: 256,
                window: rubato::WindowFunction::BlackmanHarris2,
            };

            let mut resampler =
                match SincFixedIn::<f32>::new(resample_ratio, 2.0, params, samples.len(), 1) {
                    Ok(resampler) => resampler,
                    Err(_) => return samples.to_vec(),
                };

            let waves_in = vec![samples.to_vec()];
            match resampler.process(&waves_in, None) {
                Ok(waves_out) => waves_out.into_iter().next().unwrap_or_default(),
                Err(_) => samples.to_vec(),
            }
        }
    }

    fn kitten_speed() -> f32 {
        std::env::var("REMI_KITTEN_SPEED")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0)
            .clamp(0.80, 1.20)
    }

    fn style_index(text: &str, voice_data_len: usize) -> usize {
        let style_count = (voice_data_len / 256).max(1);
        let max_index = style_count - 1;

        let default_idx = text.chars().count().clamp(80, 240).min(max_index);
        std::env::var("REMI_KITTEN_STYLE_INDEX")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|idx| idx.min(max_index))
            .unwrap_or(default_idx)
    }

    fn process_samples_for_playback(call_id: u64, samples: Vec<f32>) -> Option<(Vec<f32>, usize)> {
        if samples.is_empty() {
            return None;
        }

        let (mut samples, trim_start) = Self::trim_silence(samples);
        if samples.is_empty() {
            return None;
        }

        let no_dsp = std::env::var_os("REMI_KITTEN_NO_DSP").is_some();
        if !no_dsp {
            // Gentle polish for cleaner, fuller voice without heavy pumping.
            Self::apply_compression(&mut samples, 0.42, 1.5);
            Self::apply_soft_limiter(&mut samples, 0.78);

            let target_peak = std::env::var("REMI_KITTEN_TARGET_PEAK")
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.70)
                .clamp(0.40, 0.90);

            let peak = Self::audio_peak(&samples);
            if peak > 0.0001 {
                let gain = (target_peak / peak).clamp(0.7, 4.0);
                for sample in &mut samples {
                    *sample *= gain;
                }
            }
            Self::apply_soft_limiter(&mut samples, 0.78);
        }

        // ~8ms fade reduces clicks and keeps attacks natural.
        let fade_samples = (KITTEN_SAMPLE_RATE as f32 * 0.008) as usize;
        Self::apply_fade(&mut samples, fade_samples.max(6));

        if std::env::var_os("REMI_KITTEN_DEBUG").is_some() {
            let peak = Self::audio_peak(&samples);
            println!(
                "[Kitten-{}][post] samples={} trim_start={} peak={:.3} no_dsp={}",
                call_id,
                samples.len(),
                trim_start,
                peak,
                no_dsp
            );
        }

        Some((samples, trim_start))
    }

    fn load_voices_npz(path: &str) -> Result<HashMap<String, Vec<f32>>> {
        use std::fs::File;
        use zip::ZipArchive;

        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut voices = HashMap::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".npy") {
                let voice_name = name.trim_end_matches(".npy").to_string();
                let data = Self::parse_npy(file)?;
                voices.insert(voice_name, data);
            }
        }

        Ok(voices)
    }

    fn parse_npy<R: Read>(mut reader: R) -> Result<Vec<f32>> {
        let mut header = [0u8; 10];
        reader.read_exact(&mut header)?;

        if &header[0..6] != b"\x93NUMPY" {
            anyhow::bail!("Invalid NPY file");
        }

        let version = header[6];
        let header_len = if version == 1 {
            u16::from_le_bytes([header[8], header[9]]) as usize
        } else {
            let mut extra = [0u8; 2];
            reader.read_exact(&mut extra)?;
            u16::from_le_bytes([extra[0], extra[1]]) as usize
        };

        let mut header_bytes = vec![0u8; header_len];
        reader.read_exact(&mut header_bytes)?;

        let mut data = Vec::new();
        let mut buffer = [0u8; 4];
        while reader.read_exact(&mut buffer).is_ok() {
            data.push(f32::from_le_bytes(buffer));
        }

        Ok(data)
    }

    fn text_to_tokens(text: &str) -> Vec<i64> {
        let mut child = std::process::Command::new("espeak-ng")
            .args(["--ipa", "-q", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .ok();

        let phonemes = if let Some(ref mut c) = child {
            if let Some(mut stdin) = c.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            c.stdout
                .as_mut()
                .map(|o| {
                    let mut buf = String::new();
                    o.read_to_string(&mut buf).unwrap_or(0);
                    buf
                })
                .unwrap_or_else(|| text.to_string())
        } else {
            text.to_string()
        };

        let cleaned = Self::clean_phonemes(&phonemes.chars().collect::<String>());
        let mut tokens = vec![0i64];

        for c in cleaned.chars() {
            let id = Self::char_to_token(c);
            tokens.push(id);
        }

        tokens.push(0);
        tokens
    }

    fn clean_phonemes(phonemes: &str) -> String {
        phonemes
            .replace('\n', " ")
            .replace('\r', " ")
            .chars()
            .filter(|c| {
                matches!(c,
                    '$' | ';' | ':' | ',' | '.' | '!' | '?' | '¡' | '¿' | '—' | '…' | '"' |
                    '«' | '»' | '“' | '”' | ' ' |
                    'A'..='Z' | 'a'..='z' |
                    'ɑ' | 'ɐ' | 'ɒ' | 'æ' | 'ɓ' | 'ʙ' | 'β' | 'ɔ' | 'ɕ' | 'ç' |
                    'ɗ' | 'ɖ' | 'ð' | 'ʤ' | 'ə' | 'ɘ' | 'ɚ' | 'ɛ' | 'ɜ' | 'ɝ' |
                    'ɞ' | 'ɟ' | 'ʄ' | 'ɡ' | 'ɠ' | 'ɢ' | 'ʛ' | 'ɦ' | 'ɧ' | 'ħ' |
                    'ɥ' | 'ʜ' | 'ɨ' | 'ɪ' | 'ʝ' | 'ɭ' | 'ɬ' | 'ɫ' | 'ɮ' | 'ʟ' |
                    'ɱ' | 'ɯ' | 'ɰ' | 'ŋ' | 'ɳ' | 'ɲ' | 'ɴ' | 'ø' | 'ɵ' | 'ɸ' |
                    'θ' | 'œ' | 'ɶ' | 'ʘ' | 'ɹ' | 'ɺ' | 'ɾ' | 'ɻ' | 'ʀ' | 'ʁ' |
                    'ɽ' | 'ʂ' | 'ʃ' | 'ʈ' | 'ʧ' | 'ʉ' | 'ʊ' | 'ʋ' | 'ⱱ' | 'ʌ' |
                    'ɣ' | 'ɤ' | 'ʍ' | 'χ' | 'ʎ' | 'ʏ' | 'ʑ' | 'ʐ' | 'ʒ' | 'ʔ' |
                    'ʡ' | 'ʕ' | 'ʢ' | 'ǀ' | 'ǁ' | 'ǂ' | 'ǃ' |
                    'ˈ' | 'ˌ' | 'ː' | 'ˑ' | 'ʼ' | 'ʴ' | 'ʰ' | 'ʱ' | 'ʲ' | 'ʷ' |
                    'ˠ' | 'ˤ' | '˞' | '↓' | '↑' | '→' | '↗' | '↘'
                )
            })
            .collect()
    }

    fn char_to_token(c: char) -> i64 {
        match c {
            '$' => 0,
            ';' => 1,
            ':' => 2,
            ',' => 3,
            '.' => 4,
            '!' => 5,
            '?' => 6,
            '¡' => 7,
            '¿' => 8,
            '—' => 9,
            '…' => 10,
            '"' => 11,
            '«' => 12,
            '»' => 13,
            '“' => 14,
            '”' => 15,
            ' ' => 16,
            'A'..='Z' => 17 + (c as i64 - 'A' as i64),
            'a'..='z' => 43 + (c as i64 - 'a' as i64),
            'ɑ' => 69,
            'ɐ' => 70,
            'ɒ' => 71,
            'æ' => 72,
            'ɓ' => 73,
            'ʙ' => 74,
            'β' => 75,
            'ɔ' => 76,
            'ɕ' => 77,
            'ç' => 78,
            'ɗ' => 79,
            'ɖ' => 80,
            'ð' => 81,
            'ʤ' => 82,
            'ə' => 83,
            'ɘ' => 84,
            'ɚ' => 85,
            'ɛ' => 86,
            'ɜ' => 87,
            'ɝ' => 88,
            'ɞ' => 89,
            'ɟ' => 90,
            'ʄ' => 91,
            'ɡ' => 92,
            'ɠ' => 93,
            'ɢ' => 94,
            'ʛ' => 95,
            'ɦ' => 96,
            'ɧ' => 97,
            'ħ' => 98,
            'ɥ' => 99,
            'ʜ' => 100,
            'ɨ' => 101,
            'ɪ' => 102,
            'ʝ' => 103,
            'ɭ' => 104,
            'ɬ' => 105,
            'ɫ' => 106,
            'ɮ' => 107,
            'ʟ' => 108,
            'ɱ' => 109,
            'ɯ' => 110,
            'ɰ' => 111,
            'ŋ' => 112,
            'ɳ' => 113,
            'ɲ' => 114,
            'ɴ' => 115,
            'ø' => 116,
            'ɵ' => 117,
            'ɸ' => 118,
            'θ' => 119,
            'œ' => 120,
            'ɶ' => 121,
            'ʘ' => 122,
            'ɹ' => 123,
            'ɺ' => 124,
            'ɾ' => 125,
            'ɻ' => 126,
            'ʀ' => 127,
            'ʁ' => 128,
            'ɽ' => 129,
            'ʂ' => 130,
            'ʃ' => 131,
            'ʈ' => 132,
            'ʧ' => 133,
            'ʉ' => 134,
            'ʊ' => 135,
            'ʋ' => 136,
            'ⱱ' => 137,
            'ʌ' => 138,
            'ɣ' => 139,
            'ɤ' => 140,
            'ʍ' => 141,
            'χ' => 142,
            'ʎ' => 143,
            'ʏ' => 144,
            'ʑ' => 145,
            'ʐ' => 146,
            'ʒ' => 147,
            'ʔ' => 148,
            'ʡ' => 149,
            'ʕ' => 150,
            'ʢ' => 151,
            'ǀ' => 152,
            'ǁ' => 153,
            'ǂ' => 154,
            'ǃ' => 155,
            'ˈ' => 156,
            'ˌ' => 157,
            'ː' => 158,
            'ˑ' => 159,
            'ʼ' => 160,
            'ʴ' => 161,
            'ʰ' => 162,
            'ʱ' => 163,
            'ʲ' => 164,
            'ʷ' => 165,
            'ˠ' => 166,
            'ˤ' => 167,
            '˞' => 168,
            '↓' => 169,
            '↑' => 170,
            '→' => 171,
            '↗' => 172,
            '↘' => 173,
            _ => 0,
        }
    }

    /// Speak text using the current voice.
    pub async fn speak(&self, text: &str) {
        let voice = self
            .get_voice()
            .unwrap_or_else(|_| DEFAULT_VOICE.to_string());
        self.speak_with_voice(text, &voice).await
    }

    /// Queue speech with the current voice without waiting for playback.
    pub async fn speak_nowait(&self, text: &str) {
        let voice = self
            .get_voice()
            .unwrap_or_else(|_| DEFAULT_VOICE.to_string());
        self.speak_nowait_with_voice(text, &voice).await
    }

    /// Queue speech with a specific voice without waiting for playback.
    pub async fn speak_nowait_with_voice(&self, text: &str, voice: &str) {
        let call_id = KITTEN_COUNTER.fetch_add(1, Ordering::SeqCst);

        if self.session.is_none() || self.voices.is_empty() {
            return;
        }

        let Some(session) = self.session.as_ref() else {
            return;
        };
        let session = session.clone();
        let sink = self.sink.clone();

        let voice_data = self
            .voices
            .get(voice)
            .cloned()
            .unwrap_or_else(|| self.voices.values().next().cloned().unwrap_or_default());

        let text = text.to_string();

        let result = async {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<f32>> {
                let tokens: Vec<i64> = Self::text_to_tokens(&text);
                if tokens.len() < 2 {
                    return Ok(Vec::new());
                }

                let input_arr =
                    ndarray::Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;
                let style_idx = Self::style_index(&text, voice_data.len());
                let style: Vec<f32> = if voice_data.len() >= 256 {
                    let start = style_idx * 256;
                    let end = (start + 256).min(voice_data.len());
                    if end - start == 256 {
                        voice_data[start..end].to_vec()
                    } else {
                        vec![0.0; 256]
                    }
                } else {
                    vec![0.0; 256]
                };
                let style_arr = ndarray::Array2::from_shape_vec((1, 256), style)?;
                let speed = Self::kitten_speed();

                if std::env::var_os("REMI_KITTEN_DEBUG").is_some() {
                    println!(
                        "[Kitten-{}][synth] tokens={} style_idx={} speed={:.3}",
                        call_id,
                        tokens.len(),
                        style_idx,
                        speed
                    );
                }

                let inputs = ort::inputs![
                    "input_ids" => ort::value::Value::from_array(input_arr)?,
                    "style" => ort::value::Value::from_array(style_arr)?,
                    "speed" => ort::value::Value::from_array(ndarray::Array1::from_vec(vec![speed]))?,
                ];

                let mut sess = session.lock().map_err(|_| anyhow::anyhow!("mutex poisoned"))?;
                let outputs = sess.run(inputs)?;
                if let Some(output) = outputs.values().next() {
                    let tensor = output.try_extract_tensor::<f32>()?;
                    Ok(tensor.1.to_vec())
                } else {
                    Ok(Vec::new())
                }
            })
            .await?
        }
        .await;

        if let Ok(samples) = result {
            if let Some((samples, trim_start)) =
                Self::process_samples_for_playback(call_id, samples)
            {
                // Resample from 24000 Hz to 48000 Hz for better device compatibility
                let resampled = Self::resample_to_output_rate(&samples);
                let sample_count = resampled.len();
                let source =
                    rodio::buffer::SamplesBuffer::new(1, OUTPUT_SAMPLE_RATE as u32, resampled);
                sink.append(source.convert_samples::<f32>());
                if std::env::var_os("REMI_KITTEN_DEBUG").is_some() {
                    println!(
                        "[Kitten-{}] Playing {} samples ({} resampled, trimmed {} leading)",
                        call_id,
                        sample_count,
                        samples.len(),
                        trim_start
                    );
                }
            }
        }

        // Wait for audio to finish playing (prevents truncation)
        let _ = tokio::task::spawn_blocking(move || {
            sink.sleep_until_end();
        })
        .await;
        // Small delay to ensure audio hardware buffer is fully flushed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    /// Block until the current audio buffer finishes playing.
    pub async fn wait_until_done(&self) {
        self.sink.sleep_until_end();
    }

    /// Speak text with a specific voice and wait for playback.
    pub async fn speak_with_voice(&self, text: &str, voice: &str) {
        let call_id = KITTEN_COUNTER.fetch_add(1, Ordering::SeqCst);
        self.speaking.store(true, Ordering::SeqCst);
        let total_start = std::time::Instant::now();
        println!(
            "\n[Kitten-{}] speak: \"{}\" (voice: {})",
            call_id, text, voice
        );

        if self.session.is_none() || self.voices.is_empty() {
            eprintln!("[Kitten-{}] Not initialized", call_id);
            return;
        }

        let Some(session) = self.session.as_ref() else {
            return;
        };
        let session = session.clone();
        let sink = self.sink.clone();

        let voice_data = self
            .voices
            .get(voice)
            .cloned()
            .unwrap_or_else(|| self.voices.values().next().cloned().unwrap_or_default());

        let text = text.to_string();

        let result = async {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<f32>> {
                let token_start = std::time::Instant::now();
                let tokens: Vec<i64> = Self::text_to_tokens(&text);
                let token_time = token_start.elapsed();
                println!(
                    "[Kitten-{}] Tokens: {} ({:.1}ms)",
                    call_id,
                    tokens.len(),
                    token_time.as_secs_f64() * 1000.0
                );

                if tokens.len() < 2 {
                    return Ok(Vec::new());
                }

                let input_arr =
                    ndarray::Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;
                let style_idx = Self::style_index(&text, voice_data.len());
                let style: Vec<f32> = if voice_data.len() >= 256 {
                    let start = style_idx * 256;
                    let end = (start + 256).min(voice_data.len());
                    if end - start == 256 {
                        voice_data[start..end].to_vec()
                    } else {
                        vec![0.0; 256]
                    }
                } else {
                    vec![0.0; 256]
                };
                let style_arr = ndarray::Array2::from_shape_vec((1, 256), style)?;
                let speed = Self::kitten_speed();

                if std::env::var_os("REMI_KITTEN_DEBUG").is_some() {
                    println!(
                        "[Kitten-{}][synth] tokens={} style_idx={} speed={:.3}",
                        call_id,
                        tokens.len(),
                        style_idx,
                        speed
                    );
                }

                let inputs = ort::inputs![
                    "input_ids" => ort::value::Value::from_array(input_arr)?,
                    "style" => ort::value::Value::from_array(style_arr)?,
                    "speed" => ort::value::Value::from_array(ndarray::Array1::from_vec(vec![speed]))?,
                ];

                let infer_start = std::time::Instant::now();
                let mut sess = session.lock().map_err(|_| anyhow::anyhow!("mutex poisoned"))?;
                let outputs = sess.run(inputs)?;
                let infer_time = infer_start.elapsed();
                let result = if let Some(output) = outputs.values().next() {
                    let tensor = output.try_extract_tensor::<f32>()?;
                    tensor.1.to_vec()
                } else {
                    Vec::new()
                };
                println!("[Kitten-{}] ONNX inference: {:.1}ms, {} samples", call_id, infer_time.as_secs_f64() * 1000.0, result.len());
                Ok(result)
            })
            .await?
        }
        .await;

        let total_time = total_start.elapsed();
        match result {
            Ok(samples) => {
                if !samples.is_empty() {
                    println!(
                        "[Kitten-{}] Total: {:.1}ms, Playing {} samples...",
                        call_id,
                        total_time.as_secs_f64() * 1000.0,
                        samples.len()
                    );

                    if let Some((samples, trim_start)) =
                        Self::process_samples_for_playback(call_id, samples)
                    {
                        // Resample from 24000 Hz to 48000 Hz for better device compatibility
                        let resampled = Self::resample_to_output_rate(&samples);
                        let sample_count = resampled.len();
                        let source = rodio::buffer::SamplesBuffer::new(
                            1,
                            OUTPUT_SAMPLE_RATE as u32,
                            resampled,
                        );
                        sink.append(source.convert_samples::<f32>());
                        println!(
                            "[Kitten-{}] Playing {} samples ({} resampled, trimmed {} leading)",
                            call_id,
                            sample_count,
                            samples.len(),
                            trim_start
                        );
                        sink.sleep_until_end();
                    }
                }
            }
            Err(e) => {
                eprintln!("[Kitten-{}] Task error: {:?}", call_id, e);
            }
        }
        // Small delay to ensure audio hardware buffer is fully flushed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        self.speaking.store(false, Ordering::SeqCst);
    }

    /// Return loaded voice ids.
    pub fn available_voices(&self) -> &[String] {
        &self.voice_names
    }

    /// Return the current backend state.
    pub async fn get_state(&self) -> KittenState {
        self.state.lock().await.clone()
    }

    /// Synthesize speech samples without playing them.
    pub fn synthesize(&self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let voice = self.get_voice()?;
        let call_id = KITTEN_COUNTER.fetch_add(1, Ordering::SeqCst);

        if self.session.is_none() || self.voices.is_empty() {
            return Err(anyhow::anyhow!("Kitten not initialized"));
        }

        let Some(session) = self.session.as_ref() else {
            return Err(anyhow::anyhow!("Kitten not initialized"));
        };
        let session = session.clone();
        let voice_data = self
            .voices
            .get(&voice)
            .cloned()
            .unwrap_or_else(|| self.voices.values().next().cloned().unwrap_or_default());

        let tokens: Vec<i64> = Self::text_to_tokens(text);
        if tokens.len() < 2 {
            return Ok(Vec::new());
        }

        let input_arr = ndarray::Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;
        let style_idx = Self::style_index(text, voice_data.len());
        let style: Vec<f32> = if voice_data.len() >= 256 {
            let start = style_idx * 256;
            let end = (start + 256).min(voice_data.len());
            if end - start == 256 {
                voice_data[start..end].to_vec()
            } else {
                vec![0.0; 256]
            }
        } else {
            vec![0.0; 256]
        };
        let style_arr = ndarray::Array2::from_shape_vec((1, 256), style)?;
        let speed = Self::kitten_speed();

        let inputs = ort::inputs![
        "input_ids" => ort::value::Value::from_array(input_arr)?,
        "style" => ort::value::Value::from_array(style_arr)?,
        "speed" => ort::value::Value::from_array(ndarray::Array1::from_vec(vec![speed]))?,
        ];

        let mut sess = session
            .lock()
            .map_err(|_| anyhow::anyhow!("mutex poisoned"))?;
        let outputs = sess.run(inputs)?;

        let samples = if let Some(output) = outputs.values().next() {
            let tensor = output.try_extract_tensor::<f32>()?;
            tensor.1.to_vec()
        } else {
            Vec::new()
        };

        let processed = Self::process_samples_for_playback(call_id, samples)
            .map(|(s, _)| s)
            .unwrap_or_default();

        Ok(processed)
    }

    /// Save synthesized samples as a 16-bit mono WAV file.
    pub fn save_wav(&self, samples: &[f32], path: &str) -> Result<(), anyhow::Error> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: KITTEN_SAMPLE_RATE as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in samples {
            let amplitude = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(amplitude)?;
        }
        writer.finalize()?;
        Ok(())
    }
}

impl TextToSpeech for KittenTTS {
    fn speak(&self, text: &str) -> anyhow::Result<()> {
        let voice = self.get_voice()?;
        futures::executor::block_on(self.speak_with_voice(text, &voice));
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.speaking.store(false, Ordering::SeqCst);
        self.sink.stop();
        self.sink.clear();
        self.sink.play();
        Ok(())
    }

    fn is_speaking(&self) -> bool {
        self.speaking.load(Ordering::SeqCst)
    }

    fn name(&self) -> &'static str {
        "Kitten"
    }

    fn sample_rate(&self) -> u32 {
        KITTEN_SAMPLE_RATE as u32
    }
}

#[cfg(test)]
mod tests {
    use super::{model_info, resolve_model, KittenTTS};

    #[test]
    fn style_index_stays_in_bounds() {
        let idx = KittenTTS::style_index("hello", 256 * 32);
        assert!(idx < 32);
    }

    #[test]
    fn soft_limiter_caps_peak() {
        let mut samples = vec![0.2, 0.8, 1.6, -1.4];
        KittenTTS::apply_soft_limiter(&mut samples, 0.78);
        let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak <= 0.78 + 0.0001);
    }

    #[test]
    fn process_samples_trims_and_keeps_body() {
        let mut samples = vec![0.0; 300];
        samples.extend(vec![0.04; 2000]);
        samples.extend(vec![0.0; 300]);

        let (processed, trim_start) = KittenTTS::process_samples_for_playback(0, samples).unwrap();
        assert!(trim_start > 0);
        assert!(processed.len() > 1200);
    }

    #[test]
    fn kitten_model_aliases_resolve() {
        assert_eq!(resolve_model("nano"), "nano");
        assert_eq!(resolve_model("mini"), "nano");
        assert_eq!(resolve_model("micro"), "micro");
        // int8 falls back to nano due to instability
        assert_eq!(resolve_model("int8"), "nano");
        assert_eq!(resolve_model("unknown"), "nano");
    }

    #[test]
    fn kitten_model_info_matches_expected_assets() {
        let nano = model_info("nano");
        assert_eq!(nano.model_path, "assets/models/kitten_tts_nano_v0_8.onnx");
        assert_eq!(nano.voices_path, "assets/models/voices_v0_8.npz");

        let micro = model_info("micro");
        assert_eq!(micro.model_path, "assets/models/kitten_micro.onnx");
        assert_eq!(micro.voices_path, "assets/models/kitten_voices.npz");
    }
}

impl VoiceProvider for KittenTTS {
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
        Ok(VoiceInfo::new(DEFAULT_VOICE, "Bruno", Gender::Male))
    }
}
