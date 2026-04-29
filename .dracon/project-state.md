# Project State

## Current Focus
Introduced UI resize handling and expanded input event enums to model terminal resize, key events, and modifiers.

## Completed
- [x ] Added `UiResize` struct to store terminal width and height.
- [x ] Added `UiEvent::Resize` variant to signal terminal resizing.
- [x ] Extended `Event` enum with `Resize(u16, u16)` variant.
- [x ] Created `KeyEvent` struct containing code, modifiers, and event kind.
- [x ] Defined `KeyEventKind` enum for press, repeat, and release semantics.
- [x ] Added exhaustive `KeyCode` enum covering common keys and characters.
- [x ] Added `MediaKeyCode` enum for media playback controls.
- [x ] Added `ModifierKeyCode` enum for modifier keys.
- [x ] Implemented `KeyModifiers` bitflags for active modifier keys.
