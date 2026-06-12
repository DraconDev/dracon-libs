//! Dracon Video Runtime
//!
//! Protocol traits and exporter utilities for video workflows.
//! Runtime implementations such as FFmpeg processors, Whisper transcription, and auto-reframe are not included in this crate yet.
//!
//! ## Example
//!
//! ```rust,ignore
//! use dracon_video_runtime::exporter::srt::export_srt;
//! use dracon_video_runtime::protocol::transcript::TranscriptSegment;
//! use std::path::Path;
//!
//! let segments = vec![TranscriptSegment {
//!     start_secs: 0.0,
//!     end_secs: 1.0,
//!     text: "Example subtitle".to_string(),
//!     confidence: 1.0,
//! }];
//! export_srt(&segments, Path::new("output.srt"))?;
//! ```

// === Public API ===

// Protocol layer (traits)
pub mod protocol;

// Runtime implementations are not present in this crate yet; protocol and exporter modules compile independently.

// Exporter utilities
pub mod exporter;

// Re-exports for convenience
pub use protocol::{AudioProcessor, TranscriptProcessor, VideoProcessor};
pub use protocol::{SilenceSegment, TranscriptSegment, TrimSegment};

#[cfg(test)]
mod tests {
    use crate::TranscriptSegment;

    #[test]
    fn test_crate_exists() {
        let segments = vec![TranscriptSegment {
            start: 0.0,
            end: 1.0,
            text: "Example subtitle".to_string(),
            confidence: 1.0,
        }];
        assert_eq!(segments.len(), 1);
    }
}
