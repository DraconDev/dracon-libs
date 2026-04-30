# Project State

## Current Focus
Enable dirty‑state tracking for Checkbox, Radio, and Slider widgets by marking them dirty on state changes.

## Completed
- [x] Added `self.mark_dirty()` calls to `check`, `uncheck`, and `toggle` in Checkbox.
- [x] Added `self.mark_dirty()` calls to `select` and `deselect` in Radio.
- [x] Added `self.mark_dirty()` call to `set_value` in Slider.
