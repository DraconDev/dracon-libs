# Project State

## Current Focus
Refactor command execution to separate exit signal channel and clean up stderr handling

## Completed
- [x] Replace deprecated `stderr_tx2` with `exit_tx` derived from `stdout_tx` for sending exit code
- [x] Send exit code via `exit_tx` instead of `stdout_tx`
- [x] Use cloned `stderr_tx` as `tx2` in stderr thread for line transmission
- [x] Reorder assignment of `self.stdout_rx` and `self.stderr_rx` after modifications
- [x] Simplify variable usage and ensure proper closure captures
