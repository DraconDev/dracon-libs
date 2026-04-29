# Project State

## Current Focus
Introduce an interactive scroll state with full programmatic navigation, a scrollable container widget, and a typed context‑menu builder that adds clearer actions and theming.

## Completed
- [x] Added `ScrollState` with offset, content height, viewport height and methods (`max_offset`, `page_size`, `scroll_up`, `scroll_down`, `scroll_to`, `scroll_page_up/down`, `scroll_to_top/bottom`).
- [x] Added `ScrollContainer` exposing builder methods (`with_content_height`, `with_viewport_height`, `with_scrollbar`) and key/mouse handling, plus scrollbar rendering.
- [x] Created a typed `ContextAction` enum and populated it with explicit variants (Open, Edit, Delete, Rename, Copy, Cut, Paste, Separator).
- [x] Refactored `ContextMenu` with a builder pattern (`new`, `with_theme`) and clarified its purpose in the documentation.
- [x] Updated related widgets (modal handling) to integrate the new scroll interaction APIs.
---
