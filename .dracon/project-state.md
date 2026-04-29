# Project State

## Current Focus
refactor(editor): remove unused cursor position tracking code in TextEditor widget

## Completed
- [x] removed unused `_cursor_screen_pos` variable that was never assigned or used
- [x] removed unused `_real_line_idx` variable call in line iteration loop
- [x] removed dead code block for cursor detection in wrapped lines (commented as "hard")
- [x] removed unused `_is_last_segment` variable and related incomplete cursor logic in terminal engine editor widget
