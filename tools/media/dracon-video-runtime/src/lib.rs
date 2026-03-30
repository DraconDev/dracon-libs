//! Dracon Video Runtime
//!
//! A Rust-native video processing library with:
//! - FFmpeg-based video/audio processing
//! - Whisper-based speech transcription
//! - ML-powered face detection for auto-reframe
//!
//! ## Example
//!
//! ```rust,ignore
//! use dracon_video_runtime::{FfmpegVideoProcessor, WhisperTranscriptProcessor};
//! use dracon_video_runtime::protocol::{VideoProcessor, TranscriptProcessor};
//! use std::path::Path;
//!
//! // Process a video
//! let processor = FfmpegVideoProcessor::new();
//! let input_path = Path::new("input.mp4");
//! let silences = processor.detect_silence(input_path, -30.0, 0.5).unwrap();
//!
//! // Transcribe a video
//! let transcriber = WhisperTranscriptProcessor::new();
//! let transcript = transcriber.transcribe(input_path).unwrap();
//! ```

// === Public API ===

// Protocol layer (traits)
pub mod protocol;

// Runtime implementations
pub mod runtime;

// Exporter utilities
pub mod exporter;

// Re-exports for convenience
pub use protocol::{AudioProcessor, TranscriptProcessor, VideoProcessor};
pub use protocol::{SilenceSegment, TranscriptSegment, TrimSegment};
pub use runtime::{AutoReframeProcessor, FfmpegVideoProcessor, WhisperTranscriptProcessor};

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_exists() {
        // Basic smoke test
        assert!(true);
    }
}
