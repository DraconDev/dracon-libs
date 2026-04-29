# Project State

## Current Focus
Refactor terminal visual subsystem by removing legacy icon assets and marking compositor methods as dead-code allowed.

## Completed
- [x] Remove `visuals/assets.rs` containing legacy icon definitions (Folder, File, Rust, Json, Settings, Dracon icons)
- [x] Mark compositor methods (`new`, `tick`, `hit_test`, `draw_text`, `draw_rect`) with `#[allow(dead_code)]` for API compatibility
- [x] Fix potential overflow in compositor initialization by casting width/height to u32 before multiplication
- [x] Remove documentation comment from `guess_icon_mode()` in utils.rs
- [x] Update `visuals/mod.rs` to reflect removal of assets module
