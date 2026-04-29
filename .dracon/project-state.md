# Project State

## Current Focus
Add configurable builder methods to `SplitPane` widget for orientation inference, ratio, divider character, min size, and interactive resize handling.

## Completed
- [x] Added `Orientation` enum with `Horizontal` and `Vertical` variants.
- [x] Added `SplitPane::new` for default 50/50 split.
- [x] Added `SplitPane::from_rect` to infer orientation from rect dimensions.
- [x] Added `ratio(...)` method to set split ratio with clamping.
- [x] Added `with_divider(char)` method to configure divider character.
- [x] Added `with_min_size(u16)` method to set minimum pane size.
- [x] Added `split(&self, Rect)` -> `(Rect, Rect)` to compute pane rectangles.
- [x] Added `divider_rect(&self, Rect)` -> `Rect` to get divider rectangle.
- [x] Added `render_divider(&self, Rect)` -> `Plane` to render divider.
- [x] Added `handle_resize(...)` method for interactive mouse drag resizing.
