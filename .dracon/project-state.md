# Project State

## Current Focus
Implement dirty‑state tracking for the `List` widget to enable selective re‑rendering.

## Completed
- [x] Added a `dirty: bool` field to `List` to record when its visual state has changed.
- [x] Initialized `dirty` to `true` in constructors so new lists render on first draw.
- [x] Implemented `needs_render` and `mark_dirty` methods required by the `Widget` trait.
- [x] Updated event handling (keyboard navigation, mouse scroll, selection) to set `dirty = true` after any state change that affects rendering.
