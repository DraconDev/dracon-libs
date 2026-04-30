# Project State

## Current Focus
Add theme propagation support to the UI framework and simplify example run closure.

## Completed
- [x] feat(app): introduce `set_theme` method that updates the global theme and notifies all widgets via `on_theme_change`.
- [x] feat(widget): add `on_theme_change` hook for widgets to react to theme updates.
- [x] refactor(example): remove unnecessary `move` keyword from `app.run` closure in `text_editor_demo`.
