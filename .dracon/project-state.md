# Project State

## Current Focus
Added automatic mount/unmount lifecycle hooks to widgets and integrated them into the application's widget management.

## Completed
- [x] Call `widget.on_mount()` when a widget is added via `add_widget`.
- [x] Call `widget.on_unmount()` when a widget is removed via `remove_widget`.
- [x] Declare `on_focus`, `on_blur`, `on_mount`, and `on_unmount` methods in the `Widget` trait with default empty implementations.
