# Project State

## Current Focus
Improved terminal suspension/resumption handling in the showcase example

## Context
The change refactors terminal suspension/resumption to use the new context methods rather than direct terminal access, aligning with the recently added terminal suspension/resumption support.

## Completed
- [x] Refactored terminal suspension from `ctx.terminal.suspend()` to `ctx.suspend_terminal()`
- [x] Refactored terminal resumption from `ctx.terminal.resume()` to `ctx.resume_terminal()`

## In Progress
- [x] Terminal suspension/resumption implementation in the showcase example

## Blockers
- None identified

## Next Steps
1. Verify terminal state handling remains consistent across all examples
2. Update documentation to reflect the new context methods for terminal management
