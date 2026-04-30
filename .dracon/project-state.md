# Project State

## Current Focus
Add sixel image rendering support, enhanced event logging, toast severity taxonomy, and widget hierarchy inspection.

## Completed
- [x ] Added RGB pixel getter and setter to `SixelImage` with bounds checking.
- [x ] Implemented `load_sixel` method to decode encoded sixel data into a `SixelImage`.
- [x ] Created `SixelRenderer` struct with constructors, image setting, and loading capabilities.
- [x ] Extended `LoggedEvent` with `timestamp` and `description` fields.
- [x ] Added `ToastKind` variants (`Info`, `Success`, `Warning`, `Error`) and enriched `Toast` with `id`, `message`, `kind`, `created_at`, `duration`, and `theme`.
- [x ] Enhanced `WidgetInspector` with `id`, `label`, `children`, `root`, and `theme` fields.
