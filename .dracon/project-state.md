# Project State## Current Focus
Adds a new `apply_command_output` method to the `Widget` trait, imports `ParsedOutput`, and documents that the method is called automatically by the app tick loop after `refresh_seconds` to let widgets update their internal state from a bound command's parsed output.

## Completed
- [x] Added `apply_command_output(&mut self, _output: &ParsedOutput)` method to `Widget` with documentation and a default no‑op implementation.
- [x] Updated the `Widget` trait import to include `ParsedOutput`, enabling command output handling.
