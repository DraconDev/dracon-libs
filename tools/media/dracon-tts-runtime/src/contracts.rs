use anyhow;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub gender: Gender,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl VoiceInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>, gender: Gender) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            gender,
            language: None,
            description: None,
        }
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

pub type TtsResult<T> = anyhow::Result<T>;

pub trait TextToSpeech: Send + Sync {
    fn speak(&self, text: &str) -> TtsResult<()>;

    fn stop(&self) -> TtsResult<()>;

    fn is_speaking(&self) -> bool;

    fn name(&self) -> &'static str;

    fn sample_rate(&self) -> u32 {
        24000
    }
}

pub trait VoiceProvider: Send + Sync {
    fn list_voices(&self) -> Vec<VoiceInfo>;

    fn set_voice(&self, voice: &str) -> TtsResult<bool>;

    fn current_voice(&self) -> TtsResult<VoiceInfo>;

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

pub trait TtsEngineBase: TextToSpeech + VoiceProvider {}

#[derive(Clone)]
pub struct DynTtsEngine {
    inner: Arc<dyn TextToSpeech>,
    speak_nowait_fn: Option<Arc<dyn Fn(&str) + Send + 'static>>,
    wait_until_done_fn: Option<Arc<dyn Fn() + Send + 'static>>,
}

impl DynTtsEngine {
    pub fn new<T: TextToSpeech + 'static>(engine: Arc<T>) -> Self {
        Self {
            inner: engine,
            speak_nowait_fn: None,
            wait_until_done_fn: None,
        }
    }

    pub fn with_speak_nowait<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + 'static,
    {
        self.speak_nowait_fn = Some(Arc::new(f));
        self
    }

    pub fn with_wait_until_done<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.wait_until_done_fn = Some(Arc::new(f));
        self
    }

    pub fn speak(&self, text: &str) -> TtsResult<()> {
        self.inner.speak(text)
    }

    pub fn stop(&self) -> TtsResult<()> {
        self.inner.stop()
    }

    pub fn is_speaking(&self) -> bool {
        self.inner.is_speaking()
    }

    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsConfig {
    pub engine: String,
    pub model_path: String,
    pub voices_path: String,
    pub default_voice: String,
    pub sample_rate: u32,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthesisRequest {
    pub text: String,
    pub voice_id: Option<String>,
    pub speed: Option<f32>,
    pub pitch: Option<f32>,
}

impl SynthesisRequest {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice_id: None,
            speed: None,
            pitch: None,
        }
    }

    pub fn with_voice(mut self, voice_id: impl Into<String>) -> Self {
        self.voice_id = Some(voice_id.into());
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }
}

#[derive(Clone, Debug)]
pub struct SynthesisResult {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_ms: u64,
    pub voice_used: String,
}

pub mod prelude {
    pub use crate::contracts::{
        DynTtsEngine, Gender, SynthesisRequest, SynthesisResult, TextToSpeech, TtsConfig,
        TtsEngineBase, TtsResult, VoiceInfo, VoiceProvider,
    };
}
