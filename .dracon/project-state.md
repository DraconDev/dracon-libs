# Project State

## Current Focus
Integrated tick‑driven updates, widget rendering, and event handling into a unified run loop that enables user callbacks to receive both context and app state.

## Completed
- [x] Modified `on_tick` to accept `&mut App` in addition to `&mut Ctx`.
- [x] Updated `run` to accept `F: FnMut(&mut Ctx, &mut App)` for user code access to the app.
- [x] Added tick interval tracking, counter, and callback invocation each tick.
- [x] Implemented sorting of widgets by `z_index` and rendering each plane to the compositor.
- [x] Created and passed a `Ctx` containing compositor, theme, frame count, and timing references.
- [x] Integrated user‑provided closure `f` to be called each frame after tick handling.
- [x] Added frame‑rate pacing with sleep based on elapsed time.
- [x] Relocated stdin event reading and handling into the main loop for continuous processing.
- [x] Added control‑C handling to gracefully stop the application.
- [x] Restructured the execution flow to combine tick updates, widget rendering, and event dispatch into a single cohesive loop.
