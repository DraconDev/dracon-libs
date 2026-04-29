# Project State

## Current Focus
Introduce extensive TextEditor enhancements including undo/redo, filtering, syntax highlighting cache, and richer event handling.

## Completed
- [x] Added numerous struct fields: `cursor_row`, `cursor_col`, `scroll_row`, `scroll_col`, `style`, `cursor_style`, `modified`, `show_line_numbers`, `history`, `redo_stack`, `filter_query`, `filtered_indices`, `read_only`, `selection_start`, `selection_end`, `is_selecting`, `is_dragging_selection`, `language`, `wrap`, `highlighted_cache`, `first_invalid_line`.
- [x] Implemented `new()` as alias for `default()`.
- [x] Added `with_content(&str)` to create editors pre‑populated with user‑provided text.
- [x] Added `get_content()` to retrieve the full editor content as a newline‑joined string.
- [x] Added `replace_all(&mut self, find, replace)` to globally replace all occurrences.
- [x] Added `replace_next(&mut self, find, replace)` to replace the next match after the cursor.
- [x] Added `invalidate_from(row)` to mark lines needing re‑highlighting.
- [x] Added `gutter_width()` to compute line‑number gutter width.
- [x] Added `handle_event(&mut self, event, area)` method to process input events.
- [x] Modified `get_byte_index_from_visual` to safely handle out‑of‑bounds rows.
