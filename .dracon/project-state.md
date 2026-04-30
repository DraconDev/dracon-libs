# Project State

## Current Focus
Adding widget identifier and area handling to Table and TabBar for unified widget integration.

## Completed
- [x] TabBar now stores `WidgetId` and `area` via `Cell<Rect>` and provides `new_with_id`.
- [x] Table implements `Widget` trait with `id()`, `area()`, `z_index()` and a simplified `render()` returning only `Plane`.
- [x] Table gains `set_area()` to update its bounding rectangle.
- [x] Table's `handle_key` method signature updated to accept only `KeyEvent` and return `bool`.
- [x] Partial `handle_mouse` method added to Table for basic mouse interaction, removing old complex logic.
- [x] Dependency updates reflected in Cargo.lock (binary unchanged).
