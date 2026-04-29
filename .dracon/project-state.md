# Project State

## Current Focus
Remove unnecessary mutability and dereference iterator items in layout calculation to fix borrow warnings and improve code clarity

## Completed
- [x] Removed `mut` from `remaining` variable, making it immutable
- [x] Added dereference (`*p`) to use the actual percentage value in percentage‑based sizing
- [x] Added dereference (`*n`) to use the actual ratio numerator in ratio‑based sizing
- [x] Updated Cargo.lock with refreshed dependency versions
