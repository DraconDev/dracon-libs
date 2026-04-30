# Project State

## Current Focus
Introduce focus management into the application context and enable widgets to report cursor positions.

## Completed
- [x] Extend `Ctx` with mutable references to `FocusManager` and add methods `set_focus` and `focused` for managing widget focus.
- [x] Pass `focus_manager` and `terminal` references into `Ctx` construction in the main loop.
- [x] Add `focus_manager` field to `Ctx` struct definition.
- [x] Add `cursor_position` method to the `Widget` trait, allowing widgets to optionally report their cursor location.
