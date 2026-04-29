# Project State

## Current Focus
Remove redundant input-event mapping layer and unify event types in the terminal engine.

## Completed
- [x] Deprecate `from_runtime_event` and `to_runtime_event` as identity functions; eliminate 50 lines of bidirectional mapping code.
- [x] Simplify `to_ui_event` and downstream input handling by relying on a single `Event` type.
