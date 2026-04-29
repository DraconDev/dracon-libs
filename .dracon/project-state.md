# Project State

## Current Focus
Introduce a scoped hit‑zone system for UI elements and clean up unused async polling code.

## Completed
- [x] Remove the unused `poll_input_async` function and its async‑feature gating in the TTY backend.
- [x] Add `ScopedZone` struct with containment logic for rectangular UI regions.
- [x] Add `ScopedZoneRegistry` for managing collections of scoped zones, including registration, clearing, and dispatch based on cursor position.
- [x] Export the new scoped zone types (`ScopedZone`, `ScopedZoneRegistry`) from the framework prelude.
