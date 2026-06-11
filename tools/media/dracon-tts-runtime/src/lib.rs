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

/// TTS contracts and type aliases.
pub mod contracts;
/// Kitten TTS backend.
pub mod kitten;
/// Kokoro TTS backend.
pub mod kokoro;

use anyhow::Result;

pub use contracts::{
    DynTtsEngine, Gender, SynthesisRequest, SynthesisResult, TextToSpeech, TtsConfig,
    TtsEngineBase, TtsResult, VoiceInfo, VoiceProvider,
};

pub use kitten::KittenTTS;
pub use kokoro::KokoroTts;

/// Enum dispatching to supported TTS backends.
#[derive(Clone)]
pub enum TtsEngine {
    /// Kitten backend.
    Kitten(std::sync::Arc<KittenTTS>),
    /// Kokoro backend.
    Kokoro(std::sync::Arc<KokoroTts>),
}

impl TtsEngine {
    /// Speak text synchronously.
    pub fn speak(&self, text: &str) -> Result<()> {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::speak(&**k, text),
            TtsEngine::Kokoro(k) => TextToSpeech::speak(&**k, text),
        }
    }

    /// Stop active playback.
    pub fn stop(&self) -> Result<()> {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::stop(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::stop(&**k),
        }
    }

    /// Return whether audio is currently playing.
    pub fn is_speaking(&self) -> bool {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::is_speaking(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::is_speaking(&**k),
        }
    }

    /// Return the backend name.
    pub fn name(&self) -> &'static str {
        match self {
            TtsEngine::Kitten(k) => TextToSpeech::name(&**k),
            TtsEngine::Kokoro(k) => TextToSpeech::name(&**k),
        }
    }

    /// Queue speech without waiting for playback to finish.
    pub async fn speak_nowait(&self, text: &str) {
        match self {
            TtsEngine::Kitten(k) => k.speak_nowait(text).await,
            TtsEngine::Kokoro(k) => k.speak_nowait(text).await,
        }
    }

    /// Wait until active playback finishes.
    pub async fn wait_until_done(&self) {
        match self {
            TtsEngine::Kitten(k) => k.wait_until_done().await,
            TtsEngine::Kokoro(k) => k.wait_until_done().await,
        }
    }
}
