# Project State

## Current Focus
Add a full‑featured Table widget with sortable columns, selection callbacks, theme support, and interaction hooks.

## Completed
- [x] Implemented `Table<T>` struct with columns, rows, selection, sorting, offset, theme, and on_select/on_sort callbacks.
- [x] Added column header rendering with highlighted sort column and centered labels.
- [x] Integrated row rendering with selection highlighting and HitZone generation for event handling.
- [x] Exposed the new `Table` widget via `pub use table::Table` in the widgets module.
- [x] Updated the module registry to include the new `table` module and remove obsolete `modal` duplicate declarations.
