//! YouTube chapters export

use crate::protocol::transcript::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn export_youtube_chapters(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut chapters = String::new();
    chapters.push_str("00:00 Intro\n");

    let chapter_interval_secs = 180.0; // 3 minutes
    let mut chapter_start = 0.0;
    let mut chapter_texts: Vec<String> = Vec::new();

    for seg in transcript {
        if seg.start >= chapter_start + chapter_interval_secs {
            if !chapter_texts.is_empty() {
                let joined = chapter_texts.join(" ");
                let title = joined.trim();
                let title = if title.len() > 50 {
                    &title[..50]
                } else {
                    title
                };
                let title = title.replace('\n', " ").replace('\r', "");
                chapters.push_str(&format!(
                    "{} {}\n",
                    format_youtube_time(chapter_start),
                    title
                ));
            }
            chapter_start = seg.start;
            chapter_texts.clear();
        }

        let text = seg.text.trim();
        if !text.is_empty() && text != "[No speech detected]" {
            chapter_texts.push(text.to_string());
        }
    }

    // Last chapter
    if !chapter_texts.is_empty() {
        let joined = chapter_texts.join(" ");
        let title = joined.trim();
        let title = if title.len() > 50 {
            &title[..50]
        } else {
            title
        };
        let title = title.replace('\n', " ").replace('\r', "");
        chapters.push_str(&format!(
            "{} {}\n",
            format_youtube_time(chapter_start),
            title
        ));
    }

    fs::write(output_path, chapters).context("failed to write chapters file")?;
    Ok(())
}

fn format_youtube_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_youtube_chapters() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 30.0,
                text: "Welcome everyone".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 30.0,
                end: 60.0,
                text: "Today's topic".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 200.0,
                end: 230.0,
                text: "Advanced features".to_string(),
                confidence: 1.0,
            },
        ];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.contains("00:00 Intro"));
        assert!(content.contains("03:00") || content.contains("03:20"));

        Ok(())
    }
}
