# Project State

## Current Focus
Fix rendering index calculations in Modal to correctly address border and button cells, simplifying arithmetic and removing unnecessary casts.

## Completed
- [x] Adjusted border‑right cell index from `row * self.width + self.width as usize - 1` to `(row * self.width + self.width - 1)` cast once
- [x] Simplified background/foreground cell indexing for buttons, removing redundant `as usize` casts
- [x] Updated label placement index calculation to use explicit usize arithmetic
- [x] Maintained bounds checks while streamlining the code
