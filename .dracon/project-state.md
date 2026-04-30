# Project State

## Current Focus
One‑line description: streamline core framework updates by removing unused imports, simplifying mouse event handling, and updating dependencies

## Completed
- [x] Updated Cargo.lock to newer dependency versions
- [x] Removed unnecessary `mut` on `anim` in animation.rs test
- [x] Refactored mouse event target lookup in `app.rs` to use direct area checks instead of zone dispatcher iteration
- [x] Dropped unused `ratatui::layout::Rect` import from `dirty_regions.rs`
- [x] Trimmed import list in `event_logger.rs` by removing `Color`
- [x] Trimmed import list in `profiler.rs` by removing `Color`
