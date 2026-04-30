# Project State

## Current Focus
Add comprehensive integration tests for widget event handling and simplify mouse handling in the Select widget.

## Completed
- [x] Simplified `Select::handle_mouse` to ignore the column argument and only handle left‑click on row 0.
- [x] Added `test_toggle_handle_key_enter` and `test_toggle_handle_mouse_click` to verify Toggle toggles on/off via key press and mouse click.
- [x] Added `test_radio_handle_key_enter` and `test_radio_handle_mouse_click` to verify Radio selection via key press and mouse click.
- [x] Updated `test_search_input_handle_key_enter` to use `Rc<RefCell<_>>` for tracking submitted input and assert on the borrowed value.
- [x] Modified `test_slider_handle_mouse_out_of_bounds_returns_false` to capture the consumption flag and assert it is `false`.
- [x] Refactored `test_split_pane_mouse_drag_updates_ratio` to use `SplitPane` and verify ratio changes during drag.
- [x] Added `// ========== Toggle Event Tests ==========` and `// ========== Radio Event Tests ==========` sections with corresponding tests.
- [x] Updated imports in test file to include `WidgetId`, `Rc`, and `RefCell` as needed.
