# Project State

## Current Focus
Replace unwrap with assert! on FocusManager set_focus and adjust focus change tracking; simplify Widget.handle_mouse signature.

## Completed
- [x] Replace `.unwrap()` with `assert!(fm.set_focus(...))` in focus tests
- [x] Modify focus change closure to capture `changes_ref` and push via it
- [x] Update `Widget::handle_mouse` to ignore parameters and always return `false`
- [x] Update Cargo.lock dependency snapshot
