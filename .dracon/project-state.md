# Project State

## Current Focus
Add public visual sub‑modules for icon rendering, OSC sequences, and terminal sync, and introduce a dependency‑free Base64 encoder utility.

## Completed
- [x] Updated `UiEvent::Key` to keep the key identifier as a `Cow<'static, str>` and add inline documentation comment.
- [x] Declared public sub‑modules `icons`, `osc`, and `sync` inside `visuals` for icon rendering, OSC handling, and terminal sync mode.
- [x] Added `simple_base64_encode` function in `osc.rs` that encodes bytes to Base64 without external dependencies.
