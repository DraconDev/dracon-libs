# Project State

## Current Focus
Adds configurable builder methods (title, fps, theme, tick_interval, on_tick), introduces a runnable event loop with stop capability, and provides a context (Ctx) with split utilities and rendering helpers.

## Completed
- [x] Added `title(&mut self, title: &str)` to set the terminal window title via OSC escape sequence.
- [x] Added `fps(&mut self, fps: u32)` to clamp and store a target frames‑per‑second value.
- [x] Added `theme(&mut self, theme: Theme)` to select a UI theme.
- [x] Added `tick_interval(&mut self, ms: u64)` to configure the tick callback interval.
- [x] Added `on_tick<F>(self, f: F)` to register a callback that receives the context and tick count.
- [x] Added `run<F>(self, f: F)` as the entry point that starts the event loop, handling input, ticks, and rendering.
- [x] Added `stop(&self)` to gracefully halt the running loop.
- [x] Modified `App` to store `title`, `frame_count`, `tick_interval`, `last_tick_time`, and related fields.
- [x] Introduced `Ctx` struct that gives access to the compositor, theme, and utility methods.
- [x] Implemented `Ctx::clear()` to clear the entire terminal.
- [x] Implemented `Ctx::fps()` to compute and return current frames‑per‑second.
- [x] Implemented `Ctx::split_h()` and `Ctx::split_v()` for horizontal and vertical screen splitting via closures.
- [x] Added `split_h` and `split_v` closure APIs for layout management.
- [x] Updated the `Default` implementation for `App`.
