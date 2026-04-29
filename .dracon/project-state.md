# Project State

## Current Focus
Expose a configurable tick hook with interval control and internal tick counting for the terminal engine.

## Completed
- [x] Refactored `on_tick` registration to use `RefCell` and return the modified `App` instance.
- [x] Added `tick_interval` method to configure tick frequency in milliseconds.
- [x] Implemented internal tick timing (`last_tick_time`, `tick_interval`) and tick counting (`tick_count`).
- [x] Integrated tick callback execution into the main loop, invoking it when the interval has elapsed.
