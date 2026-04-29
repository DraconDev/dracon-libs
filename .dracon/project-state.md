# Project State

## Current Focus
Use Unicode width to calculate display‑limited string lengths in breadcrumbs, modal, and tabbar widgets.

## Completed
- [x] Added `unicode_width::UnicodeWidthStr` import to `breadcrumbs.rs`.
- [x] Replaced `title.len()` with `title.width()` and applied `saturating_sub` in Modal title length calculation.
- [x] Replaced `label.len()` with `label.width()` and applied `saturating_sub` for button label length in Modal.
- [x] Added `unicode_width::UnicodeWidthStr` import to `modal.rs`.
- [x] Added `unicode_width::UnicodeWidthStr` import to `tabbar.rs`.
- [x] Replaced `tab.len()` with `tab.width()` and applied `saturating_sub` for label length in TabBar.
