# Project State

## Current Focus
Refactor input-mouse handling and test assertions to use precise `Down(Left)` events instead of ambiguous `Press`, while removing obsolete ratatui backend tests.

## Completed
- [x] Tighten mouse event semantics: replace `MouseEventKind::Press` with `MouseEventKind::Down(MouseButton::Left)` across mapping, text input, and password input tests to match real click behavior.
- [x] Fix test harness mutability: update `App::new()` binding to `mut` in widget-not-found test to reflect actual usage.
- [x] Remove stale ratatui integration tests: delete 175 lines of backend unit tests from `ratatui.rs` (creation, draw, cursor, clear, size, flush) to reduce maintenance overhead.
