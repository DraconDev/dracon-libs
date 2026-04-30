# Project State

## Current Focus
Add a headless `App` constructor for unit tests and clean up test imports.

## Completed
- [x] feat(app): provide `App::new_for_testing()` to create an app without requiring a real terminal, enabling headless testing.
- [x] fix(test): update test to import `App` from its module path and remove unused imports.
