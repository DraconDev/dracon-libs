# Project State

## Current Focus
Introduce a new drag‑and‑drop system and hit‑zone framework for terminal UI components.

## Completed
- [x] Added `DragPhase`, `DragItem`, `DragGhost`, `DropTarget`, and `DragManager` types with full lifecycle methods.
- [x] Implemented ghost creation (`new`, `with_size`) and rendering via `Plane`.
- [x] Provided drag control methods: `start_drag`, `move_ghost`, `end_drag`, `cancel`, `clear`, and target registration.
- [x] Introduced `ClickKind` and `DragState` enums for click and drag event classification.
- [x] Added `HitZone` struct and supporting `ScopedZone`/`ScopedZoneRegistry` for geometry‑only hit testing.
