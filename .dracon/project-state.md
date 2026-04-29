# Project State

## Current Focus
Refactored layout calculation to handle percentage and ratio constraints via a computed intermediate value and updated test assertions accordingly.

## Completed
- [x] Refactored `Layout::layout` to compute size using an intermediate `computed` variable for both percentage and ratio constraints, ensuring proper clamping with `min(*max)`.
- [x] Updated test `test_max_constraint` to replace `Constraint::Percentage(100)` and `Constraint::Max(30)` with `Constraint::Fixed(50)` and `Constraint::Max(20)`, adjusting assertions to verify the new widths.
