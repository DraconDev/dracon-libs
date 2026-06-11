#![warn(missing_docs)]

//! Dracon STT Runtime — speech-to-text with Parakeet and Whisper backends.
//!
//! ## Engines
//!
//! - [`ParakeetStt`] — Parakeet-CTC model (default, CPU-friendly)
//! - [`WhisperStt`] — Whisper model via Candle (enable with `whisper` feature)
//! - [`VadStateMachine`] — voice activity detection state machine
//!
//! ## Feature Flags
//!
//! - `parakeet` — enable Parakeet-STT (default)
//! - `whisper` — enable Whisper backend (requires candle-core, candle-nn, candle-transformers)

/// Speech-to-text module using the Parakeet engine.
pub mod parakeet;
/// Shared STT contracts and result types.
pub mod stt_contracts;
/// Voice activity detection state machine.
pub mod vad_state;

#[cfg(feature = "whisper")]
/// Whisper speech-to-text backend.
pub mod whisper;

pub use parakeet::ParakeetStt;
pub use vad_state::{VadState, VadStateMachine, VadTransition};
#[cfg(feature = "whisper")]
pub use whisper::WhisperStt;

pub use stt_contracts::{
    EngineCapabilities, SpeechToText, SttResult, TimestampedSegment, TimestampedTranscription,
    TranscriptionResult,
};

use std::sync::Arc;

/// Runtime speech-to-text engine wrapper.
#[derive(Clone)]
pub enum SttEngine {
    /// Parakeet-backed engine.
    Parakeet(Arc<ParakeetStt>),
    /// Whisper-backed engine when the `whisper` feature is enabled.
    #[cfg(feature = "whisper")]
    Whisper(Arc<WhisperStt>),
}

impl SttEngine {
    /// Transcribe audio synchronously.
    pub fn transcribe(
        &self,
        audio: &[f32],
        sample_rate: u32,
    ) -> SttResult<Option<TranscriptionResult>> {
        match self {
            SttEngine::Parakeet(p) => SpeechToText::transcribe(p.as_ref(), audio, sample_rate),
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(w) => SpeechToText::transcribe(w.as_ref(), audio, sample_rate),
        }
    }

    /// Transcribe audio asynchronously and return text when available.
    pub async fn transcribe_async(&self, audio: Vec<f32>) -> Option<String> {
        match self {
            SttEngine::Parakeet(p) => p.transcribe_audio(audio).await,
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(_) => {
                tracing::warn!("Whisper does not support async transcription yet");
                None
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SttEngine::Parakeet(p) => SpeechToText::name(p.as_ref()),
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(w) => SpeechToText::name(w.as_ref()),
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            SttEngine::Parakeet(p) => SpeechToText::is_ready(p.as_ref()),
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(w) => SpeechToText::is_ready(w.as_ref()),
        }
    }

    pub fn capabilities(&self) -> EngineCapabilities {
        match self {
            SttEngine::Parakeet(p) => SpeechToText::capabilities(p.as_ref()),
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(w) => SpeechToText::capabilities(w.as_ref()),
        }
    }

    pub fn supports_timestamps(&self) -> bool {
        self.capabilities().supports_timestamps
    }

    pub fn transcribe_with_timestamps(
        &self,
        _audio: &[f32],
        _sample_rate: u32,
    ) -> SttResult<Option<Vec<TimestampedSegment>>> {
        match self {
            SttEngine::Parakeet(_) => Ok(None),
            #[cfg(feature = "whisper")]
            SttEngine::Whisper(w) => TimestampedTranscription::transcribe_with_timestamps(
                w.as_ref(),
                _audio,
                _sample_rate,
            ),
        }
    }
}
