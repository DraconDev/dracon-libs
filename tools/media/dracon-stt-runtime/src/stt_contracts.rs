use serde::{Deserialize, Serialize};

/// STT operation result type.
pub type SttResult<T> = anyhow::Result<T>;

/// Plain transcription result without timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Recognized text.
    pub text: String,
    /// Confidence score for the recognized text.
    pub confidence: f32,
    /// Detected language, when available.
    pub language: Option<String>,
    /// Audio duration represented by this result in seconds.
    pub duration_secs: f32,
}

impl TranscriptionResult {
    /// Create a transcription result with unknown confidence and duration.
    pub fn new(text: String) -> Self {
        Self {
            text,
            confidence: 0.0,
            language: None,
            duration_secs: 0.0,
        }
    }
}

/// One timestamped transcription segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedSegment {
    /// Segment start time in seconds.
    pub start_secs: f64,
    /// Segment end time in seconds.
    pub end_secs: f64,
    /// Recognized text for the segment.
    pub text: String,
    /// Confidence score for the segment.
    pub confidence: f32,
}

/// Capability flags describing a speech-to-text backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EngineCapabilities {
    /// Whether the backend can return timestamped segments.
    pub supports_timestamps: bool,
    /// Whether the backend supports streaming transcription.
    pub supports_streaming: bool,
    /// Whether the backend detects language automatically.
    pub supports_language_detection: bool,
}

/// Speech-to-text backend contract.
pub trait SpeechToText: Send + Sync {
    /// Return the backend name.
    fn name(&self) -> &'static str;
    /// Return the expected sample rate in Hz.
    fn sample_rate(&self) -> u32;
    /// Return whether the backend is ready.
    fn is_ready(&self) -> bool;
    /// Return backend capabilities.
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities::default()
    }
    /// Transcribe audio samples.
    fn transcribe(&self, audio: &[f32], sample_rate: u32)
        -> SttResult<Option<TranscriptionResult>>;
}

/// Speech-to-text backend contract with timestamped segment support.
pub trait TimestampedTranscription: SpeechToText {
    /// Transcribe audio samples into timestamped segments.
    fn transcribe_with_timestamps(
        &self,
        _audio: &[f32],
        _sample_rate: u32,
    ) -> SttResult<Option<Vec<TimestampedSegment>>> {
        Ok(None)
    }
}
