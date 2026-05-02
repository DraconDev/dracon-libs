# Project State

## Current Focus
Fix terminal corruption when launching child processes from the showcase example.

## Context
The showcase example launches child TUI applications using `Command::spawn()`, which inherits the parent's raw terminal state (alternate screen, raw mode, mouse capture, hidden cursor), causing immediate corruption when the child starts.

## Completed
- [x] Added `suspend()` and `resume()` methods to the `Terminal` struct to properly manage terminal state
- [x] Modified the showcase example to use these methods when launching child processes
- [x] Created a plan document outlining the problem, solution, and verification steps

## In Progress
- [x] Implementation of the terminal state management methods

## Blockers
- None identified

## Next Steps
1. Verify the fix works by running the showcase example and launching a child process
2. Consider adding similar state management for other terminal operations
