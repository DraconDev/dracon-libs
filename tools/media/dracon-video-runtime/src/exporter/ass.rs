//! ASS subtitle export for styled captions

use crate::protocol::transcript::TranscriptSegment;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn export_ass(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut ass = String::new();

    ass.push_str("[Script Info]\n");
    ass.push_str("Title: Generated Captions\n");
    ass.push_str("ScriptType: v4.00+\n");
    ass.push_str("Collisions: Normal\n");
    ass.push_str("PlayDepth: 0\n\n");

    ass.push_str("[V4+ Styles]\n");
    ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    ass.push_str("Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,2,2,2,10,10,30,1\n\n");

    ass.push_str("[Events]\n");
    ass.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for seg in transcript {
        let text = seg.text.trim();
        if text.is_empty() || text == "[No speech detected]" {
            continue;
        }
        let start = format_ass_time(seg.start);
        let end = format_ass_time(seg.end);
        let escaped = text.replace(['\\', '\n'], "\\N").replace('\r', "");
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start, end, escaped
        ));
    }

    fs::write(output_path, ass)?;
    Ok(())
}

fn format_ass_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let centisecs = ((seconds % 1.0) * 100.0) as u32;
    format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs, centisecs)
}
