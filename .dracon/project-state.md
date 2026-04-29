# Project State

## Current Focus
Updating AI_GUIDE to reflect the revised `ctx.fps()` return type and the new callback‑based mouse handling model for `HitZone`.

## Completed
- [x] Changed `ctx.fps()` comment from `u32` to `u64` to match the measured FPS type.
- [x] Replaced deprecated mouse‑event methods (`click_count`, `is_right_click`, `is_drag`, `drag_delta`) with the new `handle_mouse` API.
- [x] Updated usage example to employ the builder‑pattern callbacks (`.on_click`, `.on_right_click`, `.on_drag_start`) and `dispatch_mouse` for zone dispatching.
