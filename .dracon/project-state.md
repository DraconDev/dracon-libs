# Project State

## Current Focus
Add area propagation on resize and dispatch key/mouse events to widgets, and render sorted widget layers

## Completed
- [x] On terminal resize, compute full terminal area and set it for every widget via `set_area`.
- [x] Dispatch all non‑Ctrl‑C key events through `event_dispatcher` for per‑widget handling.
- [x] Dispatch mouse events through `event_dispatcher` for per‑widget handling.
- [x] Sort widgets by `z_index` before rendering and add each rendered plane to the compositor.
- [x] Integrate area‑setting logic into the resize handling path.
