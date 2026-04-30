# Project State

## Current Focus
Update widget tests to use the new Widget API with WidgetId and .with_theme, and verify theme‑dependent colors by extracting rendered cell colors.

## Completed
- [x] Replace Button constructor calls with `.with_theme(Theme)`
- [x] Use `WidgetId::default_id()` for Checkbox and Toggle widget IDs
- [x] Make Checkbox and Toggle instances mutable and call `.check()`/`.toggle()` before rendering
- [x] Extract foreground colors from the rendered plane rather than from the widget before rendering
- [x] Update imports to bring `Widget` and `WidgetId` into scope
- [x] Adjust bracket‑character and non‑reset foreground assertions to use rendered output
