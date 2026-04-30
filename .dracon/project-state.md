# Project State

## Current Focus
Refactor theme propagation tests to use per-widget call counting instead of a global mutex registry.

## Completed
- [x] Replace global `THEME_CALL_REGISTRY` mutex with per-widget `Rc<Cell<usize>>` for tracking theme change calls
- [x] Simplify `TrackingWidget` by removing `index` field and `Drop` implementation that managed global registry
- [x] Add `call_count()` method to `TrackingWidget` for direct access to theme change invocation count
- [x] Update tests to verify widgets receive theme changes individually rather than through filtered global counts
- [x] Add test verifying widget persistence after theme change
- [x] Add test verifying widget removal works correctly after theme change
