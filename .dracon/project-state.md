# Project State

## Current Focus
Add dirty‑state tracking to text input widgets and expose rendering hooks in `SearchInput`.

## Completed
- [x] Implement dirty flag in `BaseInput` with `dirty` field, and update state on text modifications, cursor movement, and area changes.
- [x] Add helper methods `set_area`, `mark_dirty`, and `clear_dirty` to `BaseInput`.
- [x] Update `SearchInput` to delegate area updates to `BaseInput::set_area` and provide `needs_render`, `mark_dirty`, and `clear_dirty` methods that interact with the new dirty flag.
- [x] Ensure `BaseInput::clear` now marks the widget as dirty for subsequent redraws.
