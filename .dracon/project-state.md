# Project State

## Current Focus
Add a password input widget that masks characters, manages cursor movement, and triggers a submission callback.

## Completed
- [x] Refactor `App::remove_widget` to call `on_unmount` on the widget before retaining the updated widget list and eliminate the unnecessary mutable binding.
- [x] Implement `PasswordInput` with theme, mask character, submission callback, and full key handling for typing, navigation, backspace, enter, and cursor positioning.
- [x] Export `PasswordInput` via `pub use` in `widgets/mod.rs` to make the widget accessible throughout the codebase.
- [x] Bump project version to 27.0.0 in `Cargo.lock`, updating dependencies accordingly.
