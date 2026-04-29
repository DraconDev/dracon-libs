# Project State

## Current Focus
Simplify the Table widget by removing its internal sorting configuration and related styling logic.

## Completed
- [x] Removed the `on_sort` method and its associated callback field from the `Table` implementation.
- [x] Simplified row mapping to directly create `TableRow { data }` without allocating empty cells.
- [x] Eliminated conditional styling that highlighted the sort column, now always applying `theme.fg` and `Styles::empty`.
- [x] Updated `Cargo.lock` to reflect the latest dependency version changes.
