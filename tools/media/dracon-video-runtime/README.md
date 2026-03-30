# Dracon Video Runtime

A Rust-native video processing runtime providing:

- **Video Processing** - Silence detection/removal, trimming, stabilization, color correction
- **Audio Processing** - Loudnorm normalization, noise reduction, music mixing with auto-ducking
- **Transcription** - Whisper-based speech-to-text
- **ML Features** - Face detection for auto-reframe, person segmentation for background blur

## Architecture

This crate is designed with a **protocol-first** architecture:

```
┌─────────────────────────────────────┐
│      Application Layer               │
│  (depends on traits, not impl)      │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│      Protocol Layer (traits)         │
│  VideoProcessor, AudioProcessor,     │
│  TranscriptProcessor                │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│      Runtime Implementations         │
│  - FfmpegVideoProcessor             │
│  - WhisperTranscriptProcessor       │
│  - AutoReframeProcessor            │
└─────────────────────────────────────┘
```

## Usage

```rust
use dracon_video_runtime::{FfmpegVideoProcessor, WhisperTranscriptProcessor};
use dracon_video_runtime::protocol::{VideoProcessor, TranscriptProcessor};

// Video processing
let processor = FfmpegVideoProcessor::new();
let silences = processor.detect_silence("input.mp4", -30.0, 0.5)?;

// Transcription
let transcriber = WhisperTranscriptProcessor::new();
let transcript = transcriber.transcribe("input.mp4")?;

// Export subtitles
transcriber.to_srt(&transcript, "output.srt")?;
```

## Features

| Feature | Description |
|---------|-------------|
| `ffmpeg` | Enable FFmpeg-based processing (default) |

## License

MIT
