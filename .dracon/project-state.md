# Project State

## Current Focus
Adjust widget focus registration, simplify resize handling, and refine mouse click targeting with z-index ordering and local coordinates.

## Completed
- [x] Register widget focusability (`widget.focusable()`) instead of always `true` in the focus manager.
- [x] Remove the redundant area recomputation and `set_area` loop that previously ran on every resize event.
- [x] Reorder widget hit‑testing by sorting on `z_index()` and select the topmost matching widget.
- [x] Use localized click coordinates (`local_col`, `local_row`) when dispatching mouse events to the target widget.
- [x] Update the application’s focus state when a widget is identified as the click target.
