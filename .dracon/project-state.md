# Project State

## Current Focus
refactor(context-menu): simplify ContextMenu widget by removing submenu support, adopting owned String labels, dynamic height calculation, and index-based hit zones

## Completed
- [x] Remove unused Color import from context_menu compositor imports
- [x] Drop Submenu variant from ContextAction enum
- [x] Change ContextMenu items to use owned String labels instead of &'static str references
- [x] Remove stored height field from ContextMenu, compute height dynamically as item count
- [x] Update render_at to return HitZone<usize> tracking item indices instead of HitZone<ContextAction>
- [x] Simplify rendering loop by removing per-action Separator handling logic
- [x] Simplify handle_click to use direct index lookup for selected items
- [x] Update Cargo.lock with dependency lockfile changes
