# Project State

## Current Focus
Introduce convenience split methods and fix minor index handling in modal rendering.

## Completed
- [x] Add `Ctx::split_h` and `Ctx::split_v` to allow horizontal/vertical pane splitting with a closure on each pane.
- [x] Implement `SplitPane::from_rect` to create a split based on a `Rect` orientation automatically.
- [x] Correct modal drawing index calculations by casting to `usize` to avoid overflow warnings.
- [x] Update widget imports to include `Color` and `Styles` for consistent compositor usage.
- [x] Adjust `TabBar` imports to match the updated compositor API.
