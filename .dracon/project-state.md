# Project State

## Current Focus
Added running state tracking to the terminal engine context

## Context
This change enables better control over the application lifecycle by providing access to a shared running state flag within the terminal engine context.

## Completed
- [x] Added `running` field to `Ctx` struct to track application state
- [x] Made the field pub(crate) to allow internal access while maintaining encapsulation

## In Progress
- [x] Implementation of running state management

## Blockers
- None identified

## Next Steps
1. Implement proper shutdown handling using the running state
2. Add integration tests for the running state management
