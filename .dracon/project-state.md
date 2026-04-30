# Project State## Current Focus
Add dirty state tracking to Select and Tree widgets for selective redraw optimization.

## Completed
- [x] Added `dirty` field and `needs_render`, `mark_dirty`, `clear_dirty` methods to `Select`.
- [x] Set `dirty = true` in `Select::set_area`, key event handlers, and `Tree::set_path`.
- [x] Implemented dirty flag propagation to enable selective redraw optimization.
