# Project State

## Current Focus
feat(async): add async reader spawning with graceful shutdown via guard

## Completed
- [x] Introduce `spawn_with_shutdown` method that spawns the async reader task and returns a handle plus a guard.
- [x] Add `ShutdownGuard` struct with a `shutdown` method that drops the internal channel to terminate the task.
