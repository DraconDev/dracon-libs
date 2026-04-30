# Project State

## Current Focus
Implement dirty‑state tracking in `TextEditorAdapter` so that key and mouse events that modify the editor mark the widget as dirty for selective redraw.

## Completed
- [x] Update `handle_key` to set `self.dirty = true` when the editor reports a state‑changing key event.
- [x] Update `handle_mouse` to set `self.dirty = true` when the editor reports a state‑changing mouse event.
