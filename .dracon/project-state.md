# Project State

## Current Focus
Refactor test infrastructure to eliminate unsafe terminal mocking by replacing `std::mem::zeroed()` with a safe stdout-backed terminal in the TUI framework tests.

## Completed
- [x] Remove unsafe `dummy_terminal()` function that used `std::mem::zeroed()` to create a zeroed terminal
- [x] Add `make_ctx()` helper that constructs a proper test context with a real stdout-backed terminal
- [x] Update 10+ test functions in app.rs to use safe `make_ctx(...).terminal` instead of unsafe `&mut dummy_terminal()`
- [x] Update Cargo.lock (dependency version update)
