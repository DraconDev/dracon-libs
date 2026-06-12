//! Video processing protocol

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Represents a silence segment in the video
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegment {
    pub start: f32,
    pub end: f32,
}

/// Represents a processed segment (after silence removal)
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TrimSegment {
    pub start: f32,
    pub end: f32,
    pub speed: f32,
}

/// Face detection result
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FaceRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Video processing capabilities
pub trait VideoProcessor: Send + Sync {
    // === Silence Detection ===

    /// Detect silent segments in a video file
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        min_duration: f32,
    ) -> Result<Vec<SilenceSegment>>;

    // === Trimming ===

    /// Calculate which segments to keep based on silences
    fn calculate_keep_segments(
        &self,
        silences: &[SilenceSegment],
        video_duration: f32,
        padding: f32,
        mode: SilenceMode,
        speedup_factor: f32,
        min_silence_for_speedup: f32,
    ) -> Vec<TrimSegment>;

    /// Trim video to keep only the specified segments
    fn trim_segments(
        &self,
        input: &Path,
        output: &Path,
        segments: &[TrimSegment],
        progress: Option<&mut dyn FnMut(f32)>,
    ) -> Result<()>;

    // === Video Effects ===

    /// Stabilize video using vidstab two-pass
    fn stabilize(&self, input: &Path, output: &Path) -> Result<()>;

    /// Auto-reframe to vertical (9:16) using face detection
    fn reframe(&self, input: &Path, output: &Path) -> Result<PathBuf>;

    /// Apply background blur
    fn blur_background(&self, input: &Path, output: &Path) -> Result<()>;

    /// Apply color correction (contrast, brightness, saturation)
    fn color_correct(&self, input: &Path, output: &Path) -> Result<()>;

    // === Audio Effects ===

    /// Enhance audio with loudnorm + EQ
    fn enhance_audio(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()>;

    /// Reduce noise from audio
    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()>;

    // === Intro/Outro ===

    /// Concatenate intro + main + outro videos
    fn concatenate(
        &self,
        intro: Option<&Path>,
        main: &Path,
        outro: Option<&Path>,
        output: &Path,
    ) -> Result<()>;

    // === Utility ===

    /// Get video duration
    fn get_duration(&self, path: &Path) -> Result<f32>;
}

/// How to handle detected silences
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SilenceMode {
    /// Cut out silences completely
    #[default]
    Cut,
    /// Speed up silences instead of cutting
    Speedup,
}
