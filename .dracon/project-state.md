# Project State

## Current Focus
Add animation tweening utilities and expose debugging widgets (overlay, profiler, inspector) via module re-exports.

## Completed
- [x] Added `tools/tui/dracon-terminal-engine/src/framework/animation.rs` implementing `Animation` and `AnimationManager` for value interpolation.
- [x] Added `tools/tui/dracon-terminal-engine/src/framework/widgets/profiler.rs` defining `Profiler` and `Metric`.
- [x] Modified `tools/tui/dracon-terminal-engine/src/framework/widgets/mod.rs` to `pub use` the new debug overlay, event logger, profiler, and widget inspector modules.
- [x] Updated `tools/tui/dracon-terminal-engine/src/framework/widgets/widget_inspector.rs` with inspector-specific code.
