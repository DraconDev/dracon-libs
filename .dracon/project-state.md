# Project State

## Current Focus
Refactor test infrastructure to replace the unsafe zeroed `Vec<u8` terminal mock with a safe dummy terminal backed by `/dev/null`, and eliminate unsafe static mutable terminal handling.

## Completed
- [x] Removed `dummy_terminal()` and `dummy_terminal_ref()` functions and their unsafe static mutable state.
- [x] Added `make_test_terminal()` that creates a safe `Terminal<File>` using `File::options().read(true).write(true).open("/dev/null")?` and `Terminal::new(file)`.
- [x] Updated all test functions in `src/framework/app.rs` to call `make_test_terminal()?` instead of `dummy_terminal()`.
- [x] Removed unused imports (`std::fs::File`, `std::io::{self, Write}`, `std::os::fd::{AsFd, FromRawFd, OpenOptions}`) from the test module.
- [x] Refactored test setup to use the safe dummy terminal, eliminating unsafe code and improving test reliability.
