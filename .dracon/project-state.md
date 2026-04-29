# Project State

## Current Focus
Add default provider configuration and refactor example file

## Completed
- [x] Added `pub const DEFAULT_PROVIDER: &str = "default";` to `services/crates/ai/ai-service/src/lib.rs`
- [x] Added `default_provider: String` field to `AiService` struct and updated its constructor to initialize it
- [x] Implemented `with_default_provider` method to allow runtime override of the default provider
- [x] Modified `AiService::ask` to use `self.default_provider` instead of hardcoded `"default"` and updated error message
- [x] Renamed `tools/tui/dracon-terminal-engine/src/main.rs` to `tools/tui/dracon-terminal-engine/examples/demo.rs`
- [x] Removed launch banner printing and initialization snippet from the demo example
- [x] Added `#![warn(missing_docs)]` attribute to `tools/tui/dracon-terminal-engine/src/lib.rs`
