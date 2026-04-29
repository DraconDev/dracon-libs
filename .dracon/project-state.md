# Project State

## Current Focus
Replace poll-based async input handling with a direct async read loop in the tty backend.

## Completed
- [x] Rewrote `poll_input` in `tools/tui/dracon-terminal-engine/src/backend/tty.rs` to use `tokio::time::timeout` and a loop that reads directly from `stdin`, removing `PollEvented` and `libc::poll`.
- [x] Updated `tools/tui/dracon-terminal-engine/Cargo.toml` binary size from 852 to 877 bytes due to the code modifications.
