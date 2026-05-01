# Project State

## Current Focus
feat(framework): add command-driven widget architecture with TOML-serializable command bindings and output parsing for dracon-terminal-engine

## Completed
- [x] Implement `OutputParser` enum with 8 CLI command output parsing strategies: JSON key, JSON path, JSON array, regex, line count, exit code, severity-tagged lines, and plain text
- [x] Define `ParsedOutput` enum for structured command results (scalars, string lists, severity-tagged log lines, raw text) with `is_empty` helper method
- [x] Add `LoggedLine` struct to represent individual log lines with associated severity tags
- [x] Implement serializable `BoundCommand` struct to bind CLI commands to widgets, including output parsers and optional confirmation messages
- [x] Add `command` module to dracon-terminal-engine framework with TOML-first, AI-inspectable command-driven widget architecture
- [x] Update framework `mod.rs` to publicly expose the new `command` module
- [x] Update dracon-terminal-engine Cargo.toml and lockfile with required dependencies (serde, regex, etc.)
