# Project State

## Current Focus
Introduce a suite of new utility helpers for terminal detection, string handling, file operations, clipboard interaction, and formatting.

## Completed
- [x] Added `guess_icon_mode` to infer icon rendering mode from terminal environment variables.
- [x] Implemented `squarify` to strip control characters from strings.
- [x] Added `command_exists` to verify the presence of a command in `$PATH`.
- [x] Created `spawn_detached` for launching background processes without I/O handling.
- [x] Implemented `format_size` for human‑readable byte size formatting.
- [x] Added `format_time` for compact timestamp rendering.
- [x] Added `format_datetime_smart` for context‑aware date/time display.
- [x] Implemented `format_permissions` to convert Unix mode bits to rwx strings.
- [x] Added `is_binary_content` to detect binary data via null‑byte scanning.
- [x] Implemented `copy_recursive` for deep directory/file copying.
- [x] Implemented `move_recursive` with cross‑device fallback (copy‑then‑delete).
- [x] Added `delete_word_backwards` for word‑wise text editing.
- [x] Implemented `set_clipboard_text` supporting OSC 52, wl‑copy, xclip, and pbcopy.
- [x] Implemented `get_clipboard_text` supporting wl‑paste, xclip, and pbpaste.
