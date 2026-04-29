# Project State

## Current Focus
Refactor Easing apply method to static helper and adjust Animation update to use it

## Completed
- [x] Replace `Easing::apply(self, t)` with `Easing::apply_easing(&easing, t)` in animation logic
- [x] Update `Animation::update` to invoke the new static helper method
- [x] Update Cargo.lock reflecting dependency changes (binary size adjustment)
