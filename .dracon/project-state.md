# Project State

## Current Focus
Add compositing engine methods for rendering primitives and layout.

## Completed
- [x] Added `tick` method to advance compositor state per frame.
- [x] Added `hit_test` method to query the topmost visible plane at given coordinates.
- [x] Added `size` method to retrieve the compositor width and height.
- [x] Added `add_plane` method to insert a plane and automatically sort by z‑index.
- [x] Added `draw_text` method to render text with foreground, background and style.
- [x] Added `draw_rect` method to draw filled rectangles with character, colors and style.
- [x] Added `force_clear` method to clear the terminal and reset the frame buffer.
- [x] Added `draw_ratatui_line` method to render a ratatui `Line` at a position.
- [x] Added `resize` method to change compositor dimensions and reset the buffer.
- [x] Added `render` method to output terminal escape codes via a writer.
- [x] Added `map_color` utility to convert ratatui colors to compositor colors.
- [x] Modified `Stack` in layout to support adding child components via `add_child`.
