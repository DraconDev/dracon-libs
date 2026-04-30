# Project State

## Current Focus
Introduce a null terminal constructor for headless testing environments.

## Completed
- [x] Added `new_null` method to `Terminal` that creates a terminal with zeroed termios and discards output, gated to test builds.
