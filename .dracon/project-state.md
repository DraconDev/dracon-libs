# Project State

## Current Focus
Centralized text editor configuration and improved widget integration for dynamic resizing and accurate cursor handling.

## Completed
- [x] Moved text editor options (line numbers, indent guides, status bar) to direct configuration on the `TextEditor` instance, simplifying setup and enabling unified state management.
- [x] Implemented `set_area` method in `TextEditorAdapter` to allow framework-managed automatic resizing, ensuring widgets adapt to terminal window changes.
- [x] Fixed cursor position calculation in `TextEditorAdapter` to account for vertical/horizontal scrolling, maintaining accurate visibility under scroll.
- [x] Updated mouse event handling in `TextEditorAdapter` to use absolute screen coordinates instead of local widget coordinates for precise editor interaction.
