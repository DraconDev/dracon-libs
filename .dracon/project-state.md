# Project State

## Current Focus
Refactor Modal UI to drop lifetime parameters, simplify rendering and event handling, and tighten closure bounds.

## Completed
- [x] Added `'static` bound to the closure in `List::on_select` to enable storing boxed callbacks.
- [x] Removed the duplicate `ModalResult` enum definition and unified it in a single location.
- [x] Simplified `Modal::render` return type by eliminating the unused `'static` lifetime parameter.
- [x] Replaced the functional border‑character logic with a constant character value.
- [x] Removed the intermediate `Rect` calculation and set `Plane` position directly.
- [x] Updated button cell indexing to use explicit usize casting for clarity.
- [x] Replaced the custom `on_click` closure assignment with a direct `HitZone::new` push.
- [x] Eliminated redundant `plane.x` and `plane.y` assignments after positioning.
- [x] Streamlined mouse hit detection by dropping intermediate local coordinate variables.
- [x] Declared `btn_width` with an explicit type annotation for consistency.
