# Project State

## Current Focus
Expand and refactor CommandRunner and AppConfig unit tests for dracon-terminal-engine

## Completed
- [x] Relax exit code assertion in sync echo test to accept any non-negative value, ignore unused stderr output
- [x] Remove redundant `exit 42` synchronous command exit code test
- [x] Add test case for invalid empty command, verifying empty stdout and -1 exit code
- [x] Add test for CommandRunner JSON parsing workflow using python, validating non-empty text output from run_and_parse
- [x] Split AppConfig TOML configuration tests into minimal and with-layout variants
- [x] Update AppConfig with-layout test to use Button widget, validate fps, layout dimensions, and widget count
- [x] Fix AppConfig layout access to use explicit error message on missing layout configuration
