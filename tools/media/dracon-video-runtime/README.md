# Dracon Video Runtime

A Rust crate for video-processing protocols and subtitle/chapter exporters. Runtime implementations such as FFmpeg processors, Whisper transcription, and auto-reframe are not included in this crate yet.

## Architecture

This crate currently exposes protocol traits and exporter utilities:

- `VideoProcessor`, `AudioProcessor`, and `TranscriptProcessor` protocol traits
- silence/trim/transcript segment types
- SRT, ASS, and YouTube chapter exporters

Runtime implementations are intentionally not present yet. Planned implementations may include FFmpeg-backed video processing, Whisper transcription, and auto-reframe support after product scope is confirmed.

## Usage

```rust,ignore
use dracon_video_runtime::exporter::srt::export_srt;
use dracon_video_runtime::protocol::transcript::{TranscriptProcessor, TranscriptSegment};

// Runtime processors are not included in this crate yet.
// Implementations can adopt TranscriptProcessor and use export_srt(&segments, path)?
// once a concrete transcript is available.
```

## Features

| Feature | Description |
|---------|-------------|
| `ffmpeg` | Reserved for future FFmpeg-backed processing; not enabled by default |

## License

AGPL-3.0-only
