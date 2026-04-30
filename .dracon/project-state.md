# Project State

## Current Focus
Add dirty-state tracking to Breadcrumbs, DebugOverlay, Label, ProgressBar, Spinner, and StatusBar widgets for selective rendering.

## Completed
- [x] Add `dirty` field to Breadcrumbs and implement `needs_render`, `mark_dirty`, `clear_dirty` methods
- [x] Add `dirty` field to DebugOverlay and initialize it as dirty
- [x] Add `dirty` field to Label and implement dirty handling in `set_area`, `needs_render`, `mark_dirty`, `clear_dirty`
- [x] Add `dirty` field to ProgressBar and initialize it as dirty
- [x] Add `dirty` field to Spinner and initialize it as dirty
- [x] Add `dirty` field to StatusBar and implement dirty handling similar to other widgets
