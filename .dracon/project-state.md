# Project State

## Current Focus
Adds range, theme, and change‑callback support to the Slider widget while simplifying mouse handling.

## Completed
- [x] Added `with_range` method to configure minimum and maximum values
- [x] Added `with_theme` method to attach a theme
- [x] Added `on_change` method for registering a value‑change callback
- [x] Added `set_value` and `value` methods for mutable and read access to the slider value
- [x] Added `min` and `max` fields to store the slider’s range
- [x] Modified `handle_mouse` to drop the unused `row` parameter
