# dracon-tts-runtime

Text-to-speech with Kitten (espeak-ng) and Kokoro (ONNX) backends.

## Usage

```rust
use dracon_tts_runtime::{KittenTTS, KokoroTts};

// Lightweight TTS
let kitten = KittenTTS::new().await?;
kitten.speak("Hello world").await?;

// Neural TTS (higher quality)
let kokoro = KokoroTts::new(model_path, voices_dir).await?;
kokoro.speak("Hello world").await?;
```

## Feature Flags

- `kitten` — espeak-ng-based TTS (default, no GPU required)
- `kokoro` — ONNX neural TTS (requires `ort` ONNX runtime)

## Key Types

- [`KittenTTS`] — espeak-ng backend
- [`KokoroTts`] — ONNX Kokoro backend
- [`TtsEngine`] — enum for runtime engine selection
- `resolve_voice(name)` — resolves voice aliases ("male", "female", etc.)

## License

MIT OR Apache-2.0
