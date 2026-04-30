# Project State

## Current Focus
Refactor tick callbacks to accept only `Ctx`, removing `&mut App` and updating dependencies

## Completed
- [x] Refactor tick callback signature in `run` to accept only `&mut Ctx`
- [x] Update Cargo.lock to newer dependency versions
