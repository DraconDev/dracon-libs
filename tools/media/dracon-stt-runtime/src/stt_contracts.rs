use serde::{Deserialize, Serialize};

pub type SttResult<T> = anyhow::Result<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub language: Option<String>,
    pub duration_secs: f32,
}

impl TranscriptionResult {
    pub fn new(text: String) -> Self {
        Self {
            text,
            confidence: 0.0,
            language: None,
            duration_secs: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedSegment {
    pub start_secs: f64,
    pub end_secs: f64,
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EngineCapabilities {
    pub supports_timestamps: bool,
    pub supports_streaming: bool,
    pub supports_language_detection: bool,
}

pub trait SpeechToText: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample_rate(&self) -> u32;
    fn is_ready(&self) -> bool;
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities::default()
    }
    fn transcribe(&self, audio: &[f32], sample_rate: u32)
        -> SttResult<Option<TranscriptionResult>>;
}

pub trait TimestampedTranscription: SpeechToText {
    fn transcribe_with_timestamps(
        &self,
        _audio: &[f32],
        _sample_rate: u32,
    ) -> SttResult<Option<Vec<TimestampedSegment>>> {
        Ok(None)
    }
}
