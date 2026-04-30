# Project State

## Current Focus
Add mutable widget ID support across all widget types.

## Completed
- [x] feat(widget_id): add `set_id` method to Breadcrumbs, Button, Checkbox, ContextMenu, DebugOverlay, EventLogger, Form, Hud, Label, List, MenuBar, Modal, PasswordInput, Profiler, ProgressBar, Radio, SearchInput, Select, Slider, Spinner, Split, StatusBar, TabBar, Table, TextEditorAdapter, Toast, Toggle, Tooltip, Tree, WidgetInspector
- [x] docs(widget_id): explain that widget IDs can now be changed after construction
- [x] test(support): update tests to use mutable IDs where needed
- [x] chore(binaries): update Cargo.lock to reflect stable dependencies
