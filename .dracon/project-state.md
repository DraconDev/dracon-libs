# Project State

## Current Focus
Add documentation clarifying Terminal behavior for non-TTY environments

## Completed
- [x] docs(terminal): Document that `Terminal::new` falls back to null mode when writer is not a TTY (e.g., piped stdout in tests)
