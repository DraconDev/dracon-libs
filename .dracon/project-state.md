# Project State

## Current Focus
Refactor tick handling to pass a richer application context to callbacks and restructure widget rendering by sorting and adding planes before compositing.

## Completed
- [x] Sorted widgets by `z_index` and added each rendered plane to the compositor before processing ticks.
- [x] Created a new `Ctx` with `compositor`, `theme`, `frame_count`, and `last_frame` and passed it to tick callbacks.
- [x] Removed direct increment of `tick_count` and redundant `last_tick_time` updates from tick handling.
- [x] Added a duplicated widget‑sorting/plane‑addition block to ensure widgets are rendered prior to the rendering function `f`.
- [x] Eliminated reliance on the `App` reference in tick callbacks, focusing interaction through widget‑level functions.
