# Project State

## Current Focus
Implement dirty region tracking to optimize screen redraws and expose related APIs in the framework context.

## Completed
- [x] Added `DirtyRegionTracker` usage in `App::set_theme`, resize handling, and event processing to mark all widgets as dirty when layout changes.
- [x] Updated widget rendering loop to skip rendering when a widget does not need a refresh, and to mark widgets dirty after rendering.
- [x] Exposed `DirtyRegion`, `DirtyRegionTracker` and related methods (`mark_dirty`, `mark_all_dirty`, `needs_full_refresh`) through `Ctx`.
- [x] Imported `dirty_regions` module and `FocusManager`, `Layout`, and animation types into the framework prelude for easier access.
