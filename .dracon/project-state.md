# Project State

## Current Focus
Switch focus_manager to direct ownership and update MenuBar width calculations using `last_area_width`

## Completed
- [x] Replace `focus_manager` field from `Option<std::sync::Mutex<FocusManager>>` to `Option<FocusManager>`
- [x] Update `EventDispatcher` struct definition accordingly
- [x] Derive width from `self.last_area_width.get()` instead of a fixed `80`
- [x] Recalculate entry width as `(width / total_entries.max(1)).max(1)`
- [x] Recalculate entry index using `(col as usize / entry_width).min(total_entries.saturating_sub(1))` and clamp to a valid range
