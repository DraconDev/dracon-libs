# Project State

## Current Focus
Expose the application context to widget-level operations via the `Ctx` struct

## Completed
- [x] Added `app: &self` parameter to the tick closure in `App`
- [x] Added `app: &'a App` field to `Ctx` struct
- [x] Added `focused_widget` method to `Ctx` to retrieve focused widget ID
- [x] Added `set_focus` method to `Ctx` for programmatically setting focus
- [x] Added `widget_count` method to `Ctx` to get total widget count
- [x] Added `widget` method to `Ctx` for immutable widget access
- [x] Added `widget_mut` method to `Ctx` for mutable widget access
- [x] Added `widget_ref` generic method to downcast widget reference
- [x] Added `widget_mut_ref` generic method to downcast mutable widget reference
