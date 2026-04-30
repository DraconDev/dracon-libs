# Project State

## Current Focus
Refactor terminal test utilities by moving null terminal implementation into a dedicated test module and exposing a headless `new_null_terminal()` function.

## Completed
- [x] Added `new_null_terminal()` function that creates a `Terminal<NullWriter>` for headless testing
- [x] Moved `NullWriter` struct and its `Write`/`AsFd` implementations into the test-specific module
- [x] Updated `Terminal<NullWriter>` constructor to delegate to `new_null_terminal()`
- [x] Modified `App::new_for_testing()` to instantiate the terminal via `new_null_terminal()`
- [x] Adjusted imports in `app.rs` to use the new `new_null_terminal` function
- [x] Updated `Cargo.lock` reflecting dependency revisions (binary change)
