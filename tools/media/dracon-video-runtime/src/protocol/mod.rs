//! Dracon Video Runtime - Protocol Layer
//!
//! This module defines the traits that implementations must satisfy.
//! Applications depend on these traits, not concrete implementations.

pub mod audio;
pub mod transcript;
pub mod video;

pub use audio::AudioProcessor;
pub use transcript::{TranscriptProcessor, TranscriptSegment};
pub use video::{SilenceMode, SilenceSegment, TrimSegment, VideoProcessor};
