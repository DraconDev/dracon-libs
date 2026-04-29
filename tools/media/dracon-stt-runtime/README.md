# dracon-stt-runtime

Speech-to-text with Parakeet and Whisper backends.

## Usage

```rust
use dracon_stt_runtime::{ParakeetStt, VadStateMachine};

// Voice activity detection
let mut vad = VadStateMachine::new(0.5);
vad.process_audio(audio_samples)?;

// STT
let stt = ParakeetStt::new(model_path).await?;
let result = stt.transcribe(audio_samples).await?;
```

## Feature Flags

- `parakeet` — Parakeet-CTC model (default, CPU-friendly)
- `whisper` — Whisper model via Candle (requires ML dependencies)

## Key Types

- [`ParakeetStt`] — Parakeet-CTC backend
- [`WhisperStt`] — Whisper backend (with `whisper` feature)
- [`VadStateMachine`] — voice activity detection

## License

MIT OR Apache-2.0
