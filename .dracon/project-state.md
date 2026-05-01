# Project State

## Current Focus
feat(test): refine unit tests for drag‑and‑drop state matching and label dirty‑state assertions

## Completed
- [x] Update hitzone tests to use explicit pattern matching on `DragState` variants, improving clarity and ensuring correct field values
- [x] Replace direct `dirty` field checks with `needs_render()` method in label tests to better reflect rendering logic
- [x] Rename test to `test_label_set_area_marks_dirty` and adjust assertions to verify that setting an area marks the label for re‑render
- [x] Add assertions confirming `needs_render()` state after creating a label, clearing, and marking dirty
- [x] Regenerate `Cargo.lock` (binary update) to keep dependency lockfile in sync
