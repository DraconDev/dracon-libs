# Project State

## Current Focus
Refactor event handling and simplify tick context creation in the terminal application.

## Completed
- [x] Removed the redundant `dispatch_key` call for non‑Control key events in the `Event::Key` branch.
- [x] Simplified mouse event processing by capturing the target widget ID and invoking `widget.handle_mouse` directly.
- [x] Added creation of a `Ctx` instance for the tick callback and passed it to `on_tick` without redundant context creation.
