# Project State

## Current Focus
Remove legacy font asset and tidy module re‑exports in the terminal engine crate.

## Completed
- [x] Delete unused `font.ttf` asset from `dracon-terminal-engine` (legacy icon assets removed).
- [x] Clean up `lib.rs` by eliminating the hidden `contracts` re‑export and simplifying the `input` re‑export block.
- [x] Update `Cargo.lock` to reflect the lockfile changes after asset removal.
