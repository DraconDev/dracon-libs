# Project State

## Current Focus
Move null terminal support into a dedicated test module and eliminate duplicated code.

## Completed
- [x] Add a `test_support` module providing `NullWriter` and a `Terminal<NullWriter>` constructor `new_null()` for headless testing.
- [x] Remove the former inline null terminal implementation from `terminal.rs` and its associated test code.
- [x] Consolidate imports by dropping unused `RawFd` and retaining only `AsFd` and `BorrowedFd`.
