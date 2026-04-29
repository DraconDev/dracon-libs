# Project State

## Current Focus
Add SearchInput with theming and submit callback, and Tree widget with expand/collapse and selection callback.

## Completed
- [x] Introduced SearchInput struct with query handling and theming support
- [x] Added on_submit callback registration for Enter key
- [x] Implemented clear() and query() helper methods
- [x] Refactored layout code by removing unused height variable
- [x] Defined TreeNode with label, expanded flag, and children vector
- [x] Added add_child method to dynamically attach child nodes
- [x] Created Tree widget with root vector and theme configuration
- [x] Implemented on_select callback for node selection events
