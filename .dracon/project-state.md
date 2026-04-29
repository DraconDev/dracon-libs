# Project State

## Current Focus
Add convenience methods for viewport handling and scrolling in List and Table widgets.

## Completed
- [x] impl List<T>: added `viewport()` returning current visible segment, and `scroll_to(index)` to programmatically move selection and adjust offset.
- [x] impl Table<T>: added `get_selected()` to access the currently selected row data, `len()` for row count, `viewport()` for visible slice, and `scroll_to(index)` for selection navigation.
