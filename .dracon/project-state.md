# Project State

## Current Focus
Added graceful shutdown capability to the showcase example

## Context
This change implements the application shutdown functionality that was previously added to the terminal engine context. It allows the showcase example to properly terminate when requested.

## Completed
- [x] Added shutdown check in the showcase example's tick handler
- [x] Implemented proper context termination when `should_quit` is true

## In Progress
- [ ] None (this is a complete implementation)

## Blockers
- None (this change completes the graceful shutdown feature)

## Next Steps
1. Verify the shutdown behavior in the showcase example
2. Consider adding more graceful shutdown examples for other components
