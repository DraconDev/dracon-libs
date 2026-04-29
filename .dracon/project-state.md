# Project State

## Current Focus
Add a full‑featured application entry point and basic UI widgets while refactoring compositor integration.

## Completed
- [x] Introduced `App` struct with terminal handle, compositor, input parser, event loop, and control methods (`run`, `stop`, `title`, `fps`, `theme`)
- [x] Added `Hud` widget for displaying heads‑up‑display status information
- [x] Added `Modal` widget for overlay dialog handling
- [x] Added `Split` widget providing pane‑splitting layout capabilities
- [x] Refactored `Hitzone` to remove `ratatui` `Buffer` and `Rect` imports, using internal compositor types instead
- [x] Updated the crate root `lib.rs` to re‑export the new framework modules and adjust the public API accordingly
