# Project State

## Current Focus
Enhanced focus change notification to use `Option<WidgetId>` for both new and old IDs and introduced a generic internal callback registration mechanism.

## Completed
- [x] Refactored `notify_focus_change` to accept `Option<WidgetId>` for `new` and `old` parameters
- [x] Added `on_focus_change_internal` method for generic focus change callbacks with `Fn(Option<WidgetId>, Option<WidgetId>) + Send + Sync + 'static`
- [x] Updated callback invocation to process both `on_focus_change` and `on_focus_change_internal` collections
- [x] Introduced `Arc<Box<f>>` push for storing internal callbacks
- [x] Adjusted callback loop to call each stored callback with the provided `new` and `old` options
- [x] Modified function signature and semantics to support more flexible focus change handling
