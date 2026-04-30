# Project State

## Current Focus
Enhance text editor demo to use actual terminal size and prevent cursor coordinates from exceeding widget bounds.

## Completed
- [x] Update demo to query terminal dimensions via `tty::get_window_size` and set widget area accordingly
- [x] Clamp cursor screen coordinates to the widget’s visible area in `TextEditorAdapter::cursor_position` to avoid reporting positions outside the widget bounds
