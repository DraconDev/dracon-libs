//! Transcript processing protocol

use anyhow::Result;
use std::path::Path;

/// A segment of transcribed speech
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub confidence: f32,
}

/// Transcript processing capabilities
pub trait TranscriptProcessor: Send + Sync {
    /// Transcribe audio from a video file to text
    fn transcribe(&self, path: &Path) -> Result<Vec<TranscriptSegment>>;

    /// Export transcript as SRT subtitle format
    fn to_srt(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()>;

    /// Export transcript as YouTube chapters format
    fn to_chapters(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()>;

    /// Generate styled ASS subtitle file
    fn to_ass(&self, transcript: &[TranscriptSegment], output: &Path) -> Result<()>;
}
