# Project State

## Current Focus
Replace manual auto‑generated IDs in registry tests with explicit WidgetId values.

## Completed
- [x] Changed test `test_registry_register_and_get` to use `WidgetId::new(1)` instead of `registry.next_id()`
- [x] Changed test `test_registry_unregister` to use `WidgetId::new(2)` instead of `registry.next_id()`
