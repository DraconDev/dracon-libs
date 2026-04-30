# Project State

## Current Focus
Introduce a dirty‑state flag to the `Table` widget to enable selective redraws.

## Completed
- [x] Add `dirty: bool` field to `Table` and initialize it as `true` in constructors.
- [x] Implement `needs_render`, `mark_dirty`, and `clear_dirty` methods for dirty‑state tracking.
