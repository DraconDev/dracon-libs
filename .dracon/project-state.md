# Project State

## Current Focus
Simplify HitZone API by removing default generic parameters and splitting drag callbacks

## Completed
- [x] Removed default generic parameter from `HitZone`, making `T` required.
- [x] Simplified `on_click` and `on_right_click` to use `impl FnMut` without explicit where bound.
- [x] Replaced single `on_drag` with three dedicated methods: `on_drag_start`, `on_drag_move`, `on_drag_end`.
- [x] Removed default generic parameter from `HitZoneGroup`.
- [x] Dropped return of `zone.id` from `dispatch_mouse`, returning `None` instead.
