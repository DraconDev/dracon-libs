# Project State

## Current Focus
feat(dragdrop): translate rendered ghost to specified screen coordinates

## Completed
- [x] Changed `render` method parameters from `(_x: u16, _y: u16)` to `(x: u16, y: u16)` to use the coordinates.
- [x] Added `plane.translate(x, y)` call to position the ghost at the provided `(x, y)` location.
