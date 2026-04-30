# Project State

## Current Focus
Add a default `WidgetId` helper and refactor `List::new` to automatically use it, while also providing an explicit‑ID constructor.

## Completed
- [x] Add `WidgetId::default_id()` method that returns `WidgetId(0)`.
- [x] Simplify `List::new` to accept only `items: Vec<T>` and internally set a default ID and default theme fields.
- [x] Add `List::new_with_id(id: WidgetId, items: Vec<T>)` constructor for callers that need to specify the widget ID.
