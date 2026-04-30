# Project State

## Current Focus
feat(widgets): add dirty state tracking to Form, Hud, and BaseInput widgets for selective redraw optimization

## Completed
- [x] Add `dirty: bool` field to Form, Hud, and BaseInput (text_input_base) widgets, initialized to `true` on instantiation
- [x] Implement Widget trait methods `needs_render`, `mark_dirty`, and `clear_dirty` for Form and Hud widgets
- [x] Set Form widget `dirty` flag to `true` on state mutations: field value/error updates, area changes, focus shifts via keyboard/mouse, text input modifications (character input, backspace, full value clear)
- [x] Set Hud widget `dirty` flag to `true` when area is updated via `set_area`
