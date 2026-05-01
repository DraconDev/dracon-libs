# Project State

## Current Focus
ONE LINE: Implement widget ID management and expand comprehensive UI/UX test coverage for the terminal engine.

## Completed
- [x] Fixed missing `WidgetRegistry.next_id` by initializing it to 1 in `WidgetRegistry::new()`.
- [x] Simplified release workflow to GitHub Releases only (removed crates.io publishing step).
- [x] Adjusted CI: removed the `minimal-versions` job and the `-D warnings` flag from the clippy step.
- [x] Added 10 new integration test modules (≈ 300 tests) covering buttons, labels, panels, context menus, filters, hit zones, drag‑and‑drop, utilities, password input, key parsing, text input, terminal behavior, visuals, and layout helpers.
- [x] Introduced shared test helpers in `tests/common/mod.rs` for key generation, area creation, and mock widgets.
- [x] Updated test count from 272 to 609, reflecting the expanded test suite.
