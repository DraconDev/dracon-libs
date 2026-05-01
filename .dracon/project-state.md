# Project State

## Current Focus
Refactored the `List` and `Select` widgets to improve type safety and code organization by removing redundant type aliases and simplifying the callback definitions.

## Completed
- [x] Removed redundant `SelectCallback` type alias in `list.rs` and simplified to direct `Option<Box<dyn FnMut(&T)>>` usage
- [x] Removed redundant `ChangeCallback` type alias in `select.rs` and simplified to direct `Option<Box<dyn FnMut(&str)>>` usage
```
