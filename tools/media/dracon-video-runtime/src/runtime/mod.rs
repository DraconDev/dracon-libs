//! Runtime implementations

pub mod ffmpeg;
pub mod loudnorm;
pub mod ml;
pub mod trim;
pub mod whisper;

pub use ffmpeg::FfmpegVideoProcessor;
pub use ml::AutoReframeProcessor;
pub use whisper::WhisperTranscriptProcessor;
