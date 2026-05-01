# Project State
##Current Focus
Refactor and modernize unit tests for the TUI framework: remove an obsolete full‑config TOML test, rename and simplify input widget tests to emphasize callback invocation, and update dependency metadata after aligning versions.

## Completed
- [x] Removed `test_app_config_from_toml_str_all_fields` test and trimmed remaining assertions in the App config test.
- [x] Renamed `test_password_input_handle_key_enter` to `test_password_input_handle_key_enter_triggers_callback` and eliminated the unused submission‑cell boilerplate.
- [x] Renamed `test_base_input_handle_key_enter` to `test_base_input_handle_key_enter_triggers_callback` and simplified the on‑submit closure handling.
- [x] Updated `tools/tui/dracon-terminal-engine/Cargo.toml` and its `Cargo.lock` to reflect the revised dependency set.
