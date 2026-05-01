# Project State
##Current Focus
Add comprehensive unit tests for dracon-terminal-engine Button widgets, refactor tests to use shared helpers, and expand coverage with new functionality checks.

## Completed
- [x] Update test imports to use `common::make_area`, `common::rect`, and shared utilities.
- [x] Replace direct mutable state in callbacks with `Rc<Cell>` and `move` closures for safe tracking.
- [x] Refactor event‑handling tests to use `common::make_key` and `common::rect`.
- [x] Add missing tests for empty label fallback, widget ID, focusability, Z‑index, cursor position, and ID setting.
- [x] Verify default ID, focusable flag, and cursor position behavior.
- [x] Update StandaloneButton rendering call to use `StandaloneButton::render`.
- [x] Add tests for multiple clicks, input handling, and focus/ID management.
