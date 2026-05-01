# Project State

## Current Focus
Refactor the test infrastructure to replace the unsafe zeroed `Vec<u8>` terminal mock with a safe, file‑backed dummy terminal. This eliminates unsound `unsafe` usage and provides a deterministic stdout for unit tests.

## Completed
- [x] Replaced `make_ctx`‑based unsafe terminal mock with `dummy_terminal()` that opens a `/tmp` file and creates a safe `Terminal<File>`.
- [x] Updated all relevant test cases to inject `&mut dummy_terminal()` as the terminal argument instead of the previous unsafe construction.
- [x] Removed the static mutable `DUMMY_STDOUT` and the surrounding `unsafe { std::mem::transmute_copy(&std::io::stdout()) }` logic.
- [x] Added `dummy_terminal()` and `dummy_terminal_ref()` helper functions to centrally manage safe dummy terminal creation in tests.
- [x] Cleaned up dead imports and annotations left from the previous test setup (e.g., removed `io::Stdout`, unused `ParserConfig`, etc.).
