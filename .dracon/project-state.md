# Project State

## Current Focus
feat(editor): add option to toggle indent guide rendering in the text editor widget

## Completed
- [x] Introduced `show_indent_guides` flag in `TextEditor` to control visibility of vertical indentation lines
- [x] Added public method `with_indent_guides` to configure the indent guide setting
- [x] Updated default initialization to set `show_indent_guides` to `false`
- [x] Updated struct documentation to explain the new flag and method usage
