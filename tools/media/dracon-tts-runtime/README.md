# dracon-tts-runtime

Text-to-speech with Kitten (espeak-ng) and Kokoro (ONNX) backends.

## Usage

```rust
use dracon_tts_runtime::{KittenTTS, KokoroTts};

// Lightweight TTS
let kitten = KittenTTS::new(
    "assets/models/kitten_tts_nano_v0_8.onnx",
    "assets/models/voices_v0_8.npz",
)
.await?;
kitten.speak("Hello world").await;

// Neural TTS (higher quality)
let kokoro = KokoroTts::new(model_path, voices_dir).await?;
kokoro.speak("Hello world").await;
```

## Feature Flags

- `kitten` — espeak-ng-based TTS
- `kokoro` — ONNX neural TTS (requires `ort` ONNX runtime)

The default feature set enables both `kitten` and `kokoro`.

## API Notes

Concrete Kitten/Kokoro backends expose async `speak()` methods. The shared `TextToSpeech` trait exposes synchronous `speak()`/`stop()` methods for `TtsEngine` dispatch.

## Key Types

- [`KittenTTS`] — espeak-ng backend
- [`KokoroTts`] — ONNX Kokoro backend
- [`TtsEngine`] — enum for runtime engine selection
- `resolve_voice(name)` — resolves voice aliases ("male", "female", etc.)
- [`TtsResult<T>`][crate::contracts::TtsResult] — `anyhow::Result<T>` type alias

## License

AGPL-3.0-only
