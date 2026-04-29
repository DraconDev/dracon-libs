# Project State

## Current Focus
Introduce a selectable list widget with builder methods and a sortable, selectable table widget with header and row hit zones.

## Completed
- [x] Added builder methods to `List<T>`: `with_theme`, `with_item_height`, `with_width`, `on_select`, `selected_index`, `get_selected`, `len`, `viewport`, `scroll_to`, `scroll_state`, `set_visible_count`, `render`, `handle_mouse`, `handle_key`.
- [x] Implemented theming support using `Theme` for `List<T>`.
- [x] Added configurable item height and width for `List<T>`.
- [x] Added selection callback registration (`on_select`) and methods to retrieve selected index/item.
- [x] Provided viewport and scroll‑state queries plus programmatic scrolling (`scroll_to`).
- [x] Added `Column` and `TableRow<T>` structs to define table columns and rows.
- [x] Added `with_rows` builder to populate `Table<T>` with data rows.
- [x] Added sortable column registration via `on_sort` callback and selection callback (`on_select`) for `Table<T>`.
- [x] Added selection index getter and related methods for `Table<T>`.
- [x] Implemented rendering and mouse handling methods for both widgets.
- [x] Updated imports and added documentation comments for the new functionality.
