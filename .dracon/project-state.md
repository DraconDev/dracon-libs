# Project State

## Current Focus
Refactor mouse button parsing to use full button codes and add release detection.

## Completed
- [x] Replace `let last_char = *self.buffer.last()?;` with `let base = b & 0b0000_0011; let is_release = last_char == b'm';` to capture release state and compute a base value.
- [x] Change button match from `match b & 0b0000_0011 {` to `match b {` and remove the `// Fallback for release` comment, enabling direct use of full button codes.
