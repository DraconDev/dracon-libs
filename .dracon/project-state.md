# Project State

## Current Focus
Refactor async input handling for async reader to improve performance and eliminate polling.

## Completed
- [x] Refatched async input handling to eliminate polling.
- [x] Fixed pattern to use `std::io::Read`'s `read()` on stdin inside a `block_in_place` to speed up the async input processing.
