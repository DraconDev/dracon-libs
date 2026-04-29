# Project State

## Current Focus
Implement a secure process‑kill controller that validates ownership before terminating a process.

## Completed
- [x] Added `ProcessController` implementing `ProcessControlContract` with UID verification and safe kill execution.
- [x] Introduced helper functions `get_process_uid` and `current_uid` to fetch process owner IDs and enforce permission checks.
