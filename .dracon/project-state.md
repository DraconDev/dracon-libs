# Project State

## Current Focus
Add an EventDispatcher to route keyboard/mouse events via hit zones, supporting capture/bubble phases and focus‑aware tab navigation.

## Completed
- [x] Added new file `src/framework/event_dispatcher.rs` defining `EventDispatcher`, `DispatchEntry`, and associated structs.
- [x] Implemented logic for grouping hit zones into capture and bubble phases.
- [x] Provided `dispatch_mouse` and `dispatch_key` methods that route events to registered widgets.
- [x] Integrated `FocusManager` for tab‑navigation handling and focus routing.
- [x] Added `Default` trait impl and `with_focus` constructor for focus manager injection.
- [x] Added unit tests verifying mouse capture ordering, tab navigation, and zone registration.
- [x] Modified `src/framework/mod.rs` to import and expose the new dispatcher functionality.
