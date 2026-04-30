# Project State

## Current Focus
Implement graceful fallback for non‑TTY environments by adding a null‑mode Terminal and removing legacy test‑only constructors.

## Completed
- [x] Add `new_null_mode` method to `Terminal` that creates a zero‑initialized terminal for headless contexts
- [x] Modify `Drop` for `Terminal` to silently ignore errors when operating in null mode
- [x] Update `Terminal::new` to detect `ENOTTY` (error 25) and automatically switch to null mode
- [x] Remove the deprecated `new_null_terminal` function and `NullWriter` test helper
- [x] Simplify `App` construction by eliminating the `new_for_testing` helper; tests now use `App::new()` directly
- [x] Adjust theme‑propagation tests to use the standard `App` constructor and update assertions accordingly
- [x] Fix increment semantics in `TrackingWidget` to use `saturating_add` for safe overflow handling
---
*All listed changes are fully implemented and reflected in the current codebase.*
