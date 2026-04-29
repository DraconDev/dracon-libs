# Project State

## Current Focus
Replace segment length‑based width calculation with actual segment width in the breadcrumbs widget

## Completed
- [x] Switch from `segment.len()` to `segment.width()` when computing `seg_width` in breadcrumbs
- [x] Use the segment's rendered width to ensure accurate width limits and proper handling of wide characters
