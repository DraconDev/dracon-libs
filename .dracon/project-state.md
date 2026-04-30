# Project State

## Current Focus
Expose multi-cursor operations on TextEditor and decouple adapter from theme.

## Completed
- [x] Add public API for multi-cursor management (add, remove, clear, count, list) to support simultaneous editing points.
- [x] Remove unused `Theme` import from `TextEditorAdapter` to reduce coupling and clarify dependencies.
- [x] Mark unused binding `_is_opening` in auto-pairing logic to silence warnings while preserving bracket-matching behavior.
