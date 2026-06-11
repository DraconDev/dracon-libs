use anyhow;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Voice gender metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    /// Male voice.
    Male,
    /// Female voice.
    Female,
    /// Other or unspecified voice.
    Other,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
            Gender::Other => write!(f, "other"),
        }
    }
}

impl From<&str> for Gender {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "male" | "m" => Gender::Male,
            "female" | "f" => Gender::Female,
            _ => Gender::Other,
        }
    }
}

/// Metadata describing a TTS voice.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// Stable voice identifier.
    pub id: String,
    /// Human-readable voice name.
    pub name: String,
    /// Voice gender metadata.
    pub gender: Gender,
    /// Optional language tag.
    #[serde(default)]
    pub language: Option<String>,
    /// Optional voice description.
    #[serde(default)]
    pub description: Option<String>,
}

impl VoiceInfo {
    /// Create a voice with required identity fields.
    pub fn new(id: impl Into<String>, name: impl Into<String>, gender: Gender) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            gender,
            language: None,
            description: None,
        }
    }

    /// Attach a language tag to this voice.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Attach a human-readable description to this voice.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// TTS operation result type.
pub type TtsResult<T> = anyhow::Result<T>;

/// Synchronous text-to-speech backend contract.
pub trait TextToSpeech: Send + Sync {
    /// Speak `text` synchronously.
    fn speak(&self, text: &str) -> TtsResult<()>;

    /// Stop any active playback.
    fn stop(&self) -> TtsResult<()>;

    /// Return whether audio is currently playing.
    fn is_speaking(&self) -> bool;

    /// Return the backend name.
    fn name(&self) -> &'static str;

    /// Return the native sample rate in Hz.
    fn sample_rate(&self) -> u32 {
        24000
    }
}

/// Voice provider contract for listing and selecting voices.
pub trait VoiceProvider: Send + Sync {
    /// List all available voices.
    fn list_voices(&self) -> Vec<VoiceInfo>;

    /// Select a voice by id or name.
    fn set_voice(&self, voice: &str) -> TtsResult<bool>;

    /// Return the currently selected voice.
    fn current_voice(&self) -> TtsResult<VoiceInfo>;

    /// Resolve a voice by id, display name, or gender alias.
    fn resolve_voice(&self, name: &str) -> Option<VoiceInfo> {
        let name_lower = name.to_lowercase();

        for voice in self.list_voices() {
            if voice.id.to_lowercase() == name_lower {
                return Some(voice);
            }
            if voice.name.to_lowercase() == name_lower {
                return Some(voice);
            }
        }

        let gender = Gender::from(name);
        if gender != Gender::Other {
            return self.list_voices().into_iter().find(|v| v.gender == gender);
        }

        None
    }
}

/// Combined TTS engine marker trait.
pub trait TtsEngineBase: TextToSpeech + VoiceProvider {}

/// Type-erased text callback used by [`DynTtsEngine`].
type SpeakNowaitCallback = dyn Fn(&str) + Send + 'static;

/// Type-erased wait callback used by [`DynTtsEngine`].
type WaitUntilDoneCallback = dyn Fn() + Send + 'static;

/// Type-erased TTS engine wrapper with optional async helpers.
#[derive(Clone)]
pub struct DynTtsEngine {
    inner: Arc<dyn TextToSpeech>,
    speak_nowait_fn: Option<Arc<SpeakNowaitCallback>>,
    wait_until_done_fn: Option<Arc<WaitUntilDoneCallback>>,
}

impl DynTtsEngine {
    /// Wrap a concrete text-to-speech engine.
    pub fn new<T: TextToSpeech + 'static>(engine: Arc<T>) -> Self {
        Self {
            inner: engine,
            speak_nowait_fn: None,
            wait_until_done_fn: None,
        }
    }

    /// Add an asynchronous fire-and-forget speak helper.
    pub fn with_speak_nowait<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + 'static,
    {
        self.speak_nowait_fn = Some(Arc::new(f));
        self
    }

    /// Add an asynchronous wait helper.
    pub fn with_wait_until_done<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.wait_until_done_fn = Some(Arc::new(f));
        self
    }

    /// Speak synchronously through the wrapped engine.
    pub fn speak(&self, text: &str) -> TtsResult<()> {
        self.inner.speak(text)
    }

    /// Stop playback through the wrapped engine.
    pub fn stop(&self) -> TtsResult<()> {
        self.inner.stop()
    }

    /// Return whether the wrapped engine is speaking.
    pub fn is_speaking(&self) -> bool {
        self.inner.is_speaking()
    }

    /// Return the wrapped engine name.
    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// Return the wrapped engine sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    /// Speak asynchronously using the configured helper or a blocking task.
    pub async fn speak_nowait(&self, text: &str) -> TtsResult<()> {
        if let Some(f) = &self.speak_nowait_fn {
            f(text);
            Ok(())
        } else {
            let text = text.to_string();
            let inner = self.inner.clone();
            tokio::task::spawn_blocking(move || inner.speak(&text))
                .await
                .map_err(|e| anyhow::anyhow!("join error: {}", e))?
        }
    }

    /// Wait until playback is no longer active.
    pub async fn wait_until_done(&self) {
        if let Some(f) = &self.wait_until_done_fn {
            f();
        } else {
            while self.inner.is_speaking() {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }
    }
}

/// TTS runtime configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Backend name (`kitten` or `kokoro`).
    pub engine: String,
    /// ONNX model path.
    pub model_path: String,
    /// Voice data path or directory.
    pub voices_path: String,
    /// Default voice id.
    pub default_voice: String,
    /// Native sample rate in Hz.
    pub sample_rate: u32,
    /// Playback volume multiplier.
    pub volume: f32,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "kitten".to_string(),
            model_path: "assets/models/kitten_tts_nano_v0_8.onnx".to_string(),
            voices_path: "assets/models/voices_v0_8.npz".to_string(),
            default_voice: "expr-voice-3-m".to_string(),
            sample_rate: 24000,
            volume: 1.0,
        }
    }
}

/// Request for synthesizing speech.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthesisRequest {
    /// Text to synthesize.
    pub text: String,
    /// Optional voice id.
    pub voice_id: Option<String>,
    /// Optional speed multiplier.
    pub speed: Option<f32>,
    /// Optional pitch adjustment.
    pub pitch: Option<f32>,
}

impl SynthesisRequest {
    /// Create a synthesis request for `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice_id: None,
            speed: None,
            pitch: None,
        }
    }

    /// Set the voice id for synthesis.
    pub fn with_voice(mut self, voice_id: impl Into<String>) -> Self {
        self.voice_id = Some(voice_id.into());
        self
    }

    /// Set the speed multiplier.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }
}

/// Synthesized audio result.
#[derive(Clone, Debug)]
pub struct SynthesisResult {
    /// Audio samples.
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Voice id used for synthesis.
    pub voice_used: String,
}

/// Common TTS types for ergonomic imports.
pub mod prelude {
    pub use crate::contracts::{
        DynTtsEngine, Gender, SynthesisRequest, SynthesisResult, TextToSpeech, TtsConfig,
        TtsEngineBase, TtsResult, VoiceInfo, VoiceProvider,
    };
}
