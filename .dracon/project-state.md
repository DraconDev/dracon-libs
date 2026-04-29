# Project State

## Current Focus
Add Tokio-based async support and expand framework documentation to explicitly mention compositor, input parsing, and event loop while updating Cargo metadata and parser doc comment.

## Completed
- [x] Updated Cargo.lock to reflect new dependency resolutions
- [x] Bumped version in tools/tui/dracon-terminal-engine/Cargo.toml (e.g., to 0.26)
- [x] Modified framework module comment to describe `App` as owning the terminal, compositor, input parsing, and event loop
- [x] Changed async feature flag from `async` to `tokio` in input module
- [x] Updated parser doc comment to use inline code formatting for `Option<Event>`
- [x] Added clarity around async handling in documentation
