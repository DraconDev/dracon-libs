# Project State

## Current Focus
Add EventDispatcher with hit‑zone routing and focus management

## Completed
- [x] Added `EventDispatcher` struct with `groups` and `entries` fields
- [x] Implemented `new()` constructor for creation without focus management
- [x] Implemented `with_focus(fm: Mutex<FocusManager>)` constructor for focus‑aware creation
- [x] Added `add_zone(zone, capture)` method to register hit zones
- [x] Added `build_groups()` method to construct capture and bubble groups
- [x] Added `dispatch_mouse(kind)` method to route mouse events
- [x] Added `dispatch_key<F>(key, handler)` method for keyboard event handling with generic handler
- [x] Updated `Cargo.lock` to reflect new dependency versions (binary file change)
