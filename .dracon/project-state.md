#Project State

## Current Focus
ONE LINE: Add comprehensive unit tests for dracon‑terminal‑engine Button widgets covering construction, rendering, theme handling, dirty state, and event handling.

## Completed
- [x] Add test for creating a new FrameworkButton and verifying label, no callback, needs_render true.
- [x] Add test verifying button ID assignment when constructed with an explicit WidgetId.
- [x] Add test confirming theme name is set via with_theme method.
- [x] Add test checking default area dimensions for a new button.
- [x] Add test ensuring empty label falls back to "Button" with brackets and correct cell positions.
- [x] Add test verifying rendered plane dimensions match the provided area.
- [x] Add test confirming brackets surround the label and cells are positioned correctly.
- [x] Add test verifying individual label characters are placed correctly in the rendered plane.
- [x] Add test ensuring long labels are truncated to fit the area width.
- [x] Add test verifying theme colors are applied to the label cell when a theme is set.
- [x] Add test confirming dirty state is cleared and no longer needs render.
- [x] Add test verifying marking dirty re‑enables render after clearing.
- [x] Add test verifying setting a new area marks the widget dirty.
- [x] Add test confirming Enter key press triggers the click callback.
- [x] Add test confirming non‑Enter key handling returns false and does not trigger callback.
- [x] Add test verifying mouse click within the button area triggers the click callback.
- [x] Add test confirming mouse click outside the button area returns false.
- [x] Add test verifying right‑click on the button does not trigger the callback.
- [x] Add test confirming multiple mouse clicks increment the callback count correctly.
