//! Audio processing protocol

use anyhow::Result;
use std::path::Path;

/// Audio processing capabilities
pub trait AudioProcessor: Send + Sync {
    /// Enhance audio: normalize loudness + gentle EQ
    fn enhance(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()>;

    /// Reduce background noise
    fn denoise(&self, input: &Path, output: &Path) -> Result<()>;

    /// Mix background music with auto-ducking based on speech segments
    fn mix_with_music(
        &self,
        video: &Path,
        music: &Path,
        output: &Path,
        speech_segments: &[(f32, f32)],
        duck_volume: f32,
    ) -> Result<()>;
}
