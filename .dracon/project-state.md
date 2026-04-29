# Project State

## Current Focus
Add Kitty keyboard protocol parser, integration bridge for external TUI libraries, and terminal sync utilities for tear‑free rendering.

## Completed
- [x] Implement `parse_kitty_keyboard` to parse key part slices into `Event`
- [x] Add integration module documentation and expose `ratatui` for external TUI usage
- [x] Implement `begin_sync` and `end_sync` functions for DECSET 2026 synchronized rendering
