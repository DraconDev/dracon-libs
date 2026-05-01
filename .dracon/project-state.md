# Project State

## Current Focus
Expose widget-level command capabilities so AI and external tooling can enumerate and invoke CLI-bound actions per widget.

## Completed
- [x] Add `commands()` method to `Widget` trait returning `Vec<BoundCommand>` with a default empty implementation.
- [x] Wire `BoundCommand` import into the widget module to enable command-driven widget architecture.
