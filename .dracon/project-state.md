# Project State

## Current Focus Add dirty-state tracking to ContextMenu and PasswordInput to enable selective redraw optimization.

## Completed - [x] Added `dirty

bool` field to ContextMenu struct - [x] Initialized `dirty` to true in ContextMenu constructors and set it to true in `with_width`/`with_anchor` - [x] Modified PasswordInput to use `self.base.dirty` for rendering decisions and provided `needs_render`, `mark_dirty`, `clear_dirty` methods
