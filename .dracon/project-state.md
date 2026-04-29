# Project State

## Current Focus
Add private `source_row_from_visual` helper to TextEditor to reverse-map wrapped visual rows to source line index, segment offset, and total line segments

## Completed
- [x] Implement `source_row_from_visual` method for TextEditor that calculates per-line wrapping segments based on line width and viewport width, returning source line index, visual offset within the line's wrapped segments, and total segments for a given visual row
