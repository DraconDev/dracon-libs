# Project State

# Current Focus
This commit modifies the `AppConfig` struct to include a new `layout` field with optional `LayoutConfig` and updates `widgets` to a vector. It also adjusts the default implementation of `AppConfig::default()`.

## Completed
- Added new `layout` option with default value to allow config flexibility.
- Refactored `default()` method to initialize configuration fields properly.
