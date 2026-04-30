# Project State

## Current Focus
Removed direct App reference and focused widget management functions from Ctx.

## Completed
- [x] Deleted `pub(crate) app:&'a App` field from `Ctx`.
- [x] Removed `focused_widget()` method.
- [x] Removed `set_focus(&mut self, id)` method.
- [x] Removed `widget_count(&self)` method.
- [x] Removed `widget(&self, id)` method.
- [x] Removed `widget_mut(&mut self, id)` method.
- [x] Removed generic `widget_ref<T>` method.
- [x] Removed generic `widget_mut_ref<T>` method.
