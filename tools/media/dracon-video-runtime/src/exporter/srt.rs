//! SRT subtitle export

use crate::protocol::transcript::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn export_srt(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut srt = String::new();
    for (i, seg) in transcript.iter().enumerate() {
        srt.push_str(&format!("{}\n", i + 1));
        srt.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(seg.start),
            format_srt_time(seg.end)
        ));
        srt.push_str(&format!("{}\n\n", seg.text.trim()));
    }

    fs::write(output_path, srt).context("failed to write SRT file")?;
    Ok(())
}

fn format_srt_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millis = ((seconds % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_srt() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subtitles.srt");
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 5.0,
                text: "Hello world".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 5.0,
                end: 10.0,
                text: "This is a test".to_string(),
                confidence: 1.0,
            },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(output_srt)?;
        assert!(content.contains("1\n"));
        assert!(content.contains("Hello world"));
        assert!(content.contains("2\n"));
        assert!(content.contains("This is a test"));
        assert!(content.contains("00:00:00,000 --> 00:00:05,000"));

        Ok(())
    }
}
