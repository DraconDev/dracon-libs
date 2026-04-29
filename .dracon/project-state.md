# Project State

## Current Focus
Use Unicode width to calculate display‑limited string lengths in drag‑and‑drop, breadcrumbs, HUD, and list widgets.

## Completed
- [x] Upgrade dracon-terminal-engine to version 26.0.1 (Cargo.lock binary updated)
- [x] Replace `label.len()` with `label.width()` in DragGhost width calculation
- [x] Replace `segment.len()` with `segment.width()` in Breadcrumbs width limiting
- [x] Replace `text.len()` with `text.width()` in HUD text length limiting
- [x] Replace `label.len()` with `label.width()` in HUD label length limiting
- [x] Replace `text.len()` with `text.width()` in List label length limiting
- [x] Add `unicode_width::UnicodeWidthStr` import to affected modules
