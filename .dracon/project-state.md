# Project State

## Current Focus
Upgrade to dracon-terminal-engine v26.0.1 and refresh README to match new API and documentation.

## Completed
- [x] Bumped `dracon-terminal-engine` version from git reference to `26.0.1` in `Cargo.toml`.
- [x] Updated README code example: replaced `App::new()?` with `.unwrap()`, added `.on_tick` closure, removed `split_h` in favor of `size` and `Rect`‑based rendering.
- [x] Changed framework version header from “Framework (v25)” to “Framework (v26)”.
- [x] Updated changelog reference from `v26.0.0` to `v26.0.1`.
- [x] Added documentation entries for new API items: `App::on_tick`, `App::tick_interval`, `Ctx::split_v`, `ScopedZone<T>`, `ScopedZoneRegistry<T>`, `DragManager<T>`.
- [x] Revised dependencies section to show version `26.0.1` and optional git tag usage.
- [x] Removed outdated `split_h` usage and related comments from the example.
