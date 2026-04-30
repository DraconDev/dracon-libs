# Project State

## Current Focus
Add `get_ratio` accessor and interactive mouse-drag resizing, refactor area handling.

## Completed
- [x] Added `pub fn get_ratio(&self) -> f32` returning the stored split ratio.
- [x] Added `pub fn handle_resize(&mut self, kind, col, row, area) -> bool` to update ratio on drag.
- [x] Refactored `handle_mouse` to use `self.area.get()` as `current_area`, compute width/height from it, and clamp ratio.
- [x] Removed old `area()` method and legacy `handle_resize` implementation.
- [x] Updated orientation logic to use `current_area` dimensions for accurate sizing.
