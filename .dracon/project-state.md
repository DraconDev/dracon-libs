# Project State

## Current Focus
Add detailed cell properties, plane compositing, and expanded input event contracts for the terminal engine.

## Completed
- [x] Expanded `Cell` struct with `char`, `fg`, `bg`, `style`, `transparent`, and documented `skip` fields.
- [x] Introduced `Plane` struct containing ID, Z‑index, position, size, cell grid, visibility, opacity, and filter, plus methods for positioning, styling, and manipulation.
- [x] Updated `Cargo.lock` to reflect new dependency versions (binary unchanged).
- [x] Added `KeyModifiers` bitflags enumerating Shift, Control, Alt, Super, Hyper, and Meta modifiers.
- [x] Added `MouseEvent` struct capturing event kind, column, row, and active modifiers.
- [x] Added `MouseEventKind` enum with variants for Down, Up, Drag, Moved, ScrollDown, ScrollUp, ScrollLeft, ScrollRight.
- [x] Added `MouseButton` enum defining Left and Right mouse buttons.
