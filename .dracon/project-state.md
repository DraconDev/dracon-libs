# Project State

## Current Focus
Added `dirty` state tracking to several widgets to enable selective redraw optimization.

## Completed
- [x] Added `dirty` boolean to `SplitPane` widget
- [x] Implemented `needs_render`, `mark_dirty`, and `clear_dirty` methods in `TextEditorAdapter` widget
- [x] Added `dirty` state to `Toast` widget and implemented related methods
- [x] Integrated `dirty` state tracking into `Tooltip` widget with corresponding methods for rendering control
