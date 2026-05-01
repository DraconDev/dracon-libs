# Project State

## Current Focus
Remove a deprecated App test and update input widget tests to use concurrency‑safe state capture mechanisms.

## Completed
- [x] Omit `test_app_run_command` from the App test suite, removing an obsolete command‑execution test.
- [x] Revise the `PasswordInput` widget test to use a `Cell<bool>` for the `submitted` flag, enabling mutation inside a moved closure.
- [x] Refactor the `BaseInput` widget test to employ an `Rc<Cell<bool>>` and an inner `RefCell`‑wrapped closure, allowing shared mutable state across the test’s moved closure.
