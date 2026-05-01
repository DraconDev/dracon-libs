# Project State

## Current Focus
feat(command-driven): implement CLI command binding architecture with TOML-based dashboard configuration and AI-accessible command surface

## Completed
- [x] feat(core): add command-driven architecture with BoundCommand and output parsers (JsonKey, JsonPath, Regex, ExitCode, SeverityLine, Plain)
- [x] feat(toml): enable dashboard definition via TOML configuration without Rust code
- [x] feat(api): expose `App::from_toml`, `App::add_command`, `App::available_commands`, `App::run_command` and corresponding `Ctx` methods
- [x] feat(widgets): add 6 new widgets (ConfirmDialog, Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText) to framework
- [x] docs(ai-guide): document AI surface for command enumeration and execution with examples
- [x] docs(readme): update widget count from 29 to 35 and document new command-driven API methods
- [x] chore(version): bump version from 27.0.1 to 27.0.2
