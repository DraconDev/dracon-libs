# Project State

## Current Focus
Add area handling and rendering metadata (area, set_area, z_index, needs_render) to the Widget trait.

## Completed
- [x] Added `area(&self) -> Rect` method
- [x] Added `set_area(&mut self, Rect)` method
- [x] Added `z_index(&self) -> u16` method
- [x] Added `needs_render(&self) -> bool` method
