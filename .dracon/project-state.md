#Project State

## Current Focus
Introduce async error handling for TTS speak_nowait and add a chat UI example.

## Completed
- [x] Added `anyhow` import and changed `speak_nowait` return type to `TtsResult<()>` with proper error propagation.
- [x] Added new example `framework_chat.rs` implementing a chat UI with list, input, theme, and timestamp logic.
