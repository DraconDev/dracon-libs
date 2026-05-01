# Project State

## Current Focus
Add shared test utilities and helpers for dracon-terminal-engine integration tests

## Completed
- [x] Add `make_key`, `make_key_repeat`, and `make_key_with_modifiers` helpers to generate KeyEvent instances for testing input scenarios
- [x] Add `dummy_area`, `make_area`, and `rect` helpers to create Rect test areas with standard, custom dimension, or custom coordinate configurations
- [x] Add `assert_rgb` helper to verify Theme color fields match expected RGB values
- [x] Implement `TrackingWidget` mock struct implementing the Widget trait, tracking theme change invocation counts and current theme name for test validation
