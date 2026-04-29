# Project State

## Current Focus
feat(tick): expose a hook registration API via `on_tick` and internal tick‑time tracking

## Completed
- [x] Added `RefCell` import and `on_tick` field to `App` to store the tick callback
- [x] Added `last_tick_time` field to record tick timestamps
- [x] Implemented `on_tick` method that sets the callback and returns `Self` for chaining
- [x] Updated `App::new` initialization to set `last_tick_time` and initialize `on_tick` as `None`
