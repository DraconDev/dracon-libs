# Project State

## Current Focus
Add a dirty state flag to Table and List widgets for selective redraw optimization.

## Completed
- Implemented `dirty = true;` in selection checks to trigger rebuilds when necessary.
- Updated scroll logic using `saturating_sub` and `min()` to refine offset adjustments safely.
