#![warn(missing_docs)]

//! Dracon TTS Runtime — text-to-speech with Kitten and Kokoro backends.
//!
//! ## Engines
//!
//! - [`KittenTTS`] — espeak-ng-based TTS (lightweight, no GPU required)
//! - [`KokoroTts`] — ONNX-based neural TTS (higher quality, GPU preferred)
//! - [`TtsEngine`] — enum dispatching to either engine
//!
//! ## Feature Flags
//!
//! - `kitten` — enable Kitten TTS (default)
//! - `kokoro` — enable Kokoro TTS (requires ort ONNX runtime)
//!
//! ## Example
//!
//! ```ignore
//! use dracon_tts_runtime::{TtsEngine, KokoroTts};
//! let tts = KokoroTts::new(model_path, voices_dir). await?;
//! tts.speak("Hello world").await;
//! ```

pub mod contracts;
pub mod kitten;
pub mod kokoro;

pub use contracts::{
    DynTtsEngine, Gender, SynthesisRequest, SynthesisResult, TextToSpeech, TtsConfig,
    TtsEngineBase, TtsResult, VoiceInfo, VoiceProvider,
};

pub use kitten::KittenTTS;
pub use kokoro::KokoroTts;

#[derive(Clone)]
pub enum TtsEngine {
    Kitten(std::sync::Arc<KittenTTS>),
    Kokoro(std::sync::Arc<KokoroTts>),
}

impl TtsEngine {
    pub fn speak(&self, text: &str) {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::speak(&**k, text),
            TtsEngine::Kokoro(k) => TextToSpeech::speak(&**k, text),
        }
    }

    pub fn stop(&self) {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::stop(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::stop(&**k),
        }
    }

    pub fn is_speaking(&self) -> bool {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::is_speaking(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::is_speaking(&**k),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::name(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::name(&**k),
        }
    }

    pub async fn speak_nowait(&self, text: &str) {
        match self {
            TtsEngine::Kitten(k) => k.speak_nowait(text).await,
            TtsEngine::Kokoro(k) => k.speak_nowait(text).await,
        }
    }

    pub async fn wait_until_done(&self) {
        match self {
            TtsEngine::Kitten(k) => k.wait_until_done().await,
            TtsEngine::Kokoro(k) => k.wait_until_done().await,
        }
    }
}
