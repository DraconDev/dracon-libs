# Project State

## Current Focus
Refactor terminal List widget to optionalize selection callback, simplify render output, and harden scroll/selection logic

## Completed
- [x] Remove generic lifetime parameter from List widget, make on_select selection callback optional (wrapped in Option instead of required)
- [x] Update List constructor to no longer require on_select closure, add builder-method to set callback post-initialization
- [x] Switch item rendering from debug format to ToString, update type bound for list items from Clone + 'static to Clone + ToString
- [x] Simplify List::render to return only Plane, remove HitZoneGroup generation and associated click handling zones
- [x] Add public set_visible_count method to adjust the number of visible list items at runtime
- [x] Replace manual scroll and selection boundary checks with saturating arithmetic to prevent underflow/overflow errors
- [x] Clean up rendering logic to use provided area width for cell fills, add 1-column text offset, set plane z-index to 10
- [x] Streamline selection callback invocation to directly access optional on_select closure
