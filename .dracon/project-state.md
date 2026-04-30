# Project State

## Current Focus
Simplify sorting logic, replace custom error handling with `io::Error::other`, and relax CI clippy warnings.

## Completed
- [x] CI: removed `-D warnings` flag from clippy run to allow non‑fatal warnings.
- [x] Compositor engine: replaced `sort_by` with `sort_by_key` for plane and layer sorting by `z_index`.
- [x] Scanline filter: switched from `y % 2 == 0` to `y.is_multiple_of(2)` for clearer intent.
- [x] Terminal cursor helpers: changed error conversions to `io::Error::other` for concise error handling.
- [x] Updated `set_cursor` method to use the new error helper while keeping escape sequence logic unchanged.
