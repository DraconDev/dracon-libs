# Project State

## Current Focus
Dynamically calculate tab width using the provided widget width instead of a fixed 80.

## Completed
- [x] Added `width: u16` parameter to `handle_mouse` method signature
- [x] Replaced `80u16` with `width` when computing `tab_width`
- [x] Used the computed `tab_width` to determine tab index based on column position
