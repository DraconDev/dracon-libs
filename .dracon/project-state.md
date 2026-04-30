# Project State

## Current Focus
Introduce release 27.0.1: a patch fixing dirty‑state propagation, improving theme handling, adding a layout helper, and updating documentation and versioning.

## Completed
- [x] fix(widget): ensure all 23+ framework widgets call `mark_dirty()` on state changes (Checkbox, Slider, Radio, Toggle)
- [x] fix(test): eliminate parallel test race in theme propagation by using per‑widget `Rc<Cell>` tracking
- [x] feat(terminal): `Terminal::new()` now falls back to null mode when stdout is not a TTY (headless/CI environments)
- [x] feat(app): `App::add_widget` now assigns widget IDs via `widget.set_id(id)` to sync App‑assigned IDs
- [x] feat(ctx): add `Ctx::layout()` constraint‑based layout helper for use in `App::run` callbacks
- [x] feat:test: add 8 new dirty‑tracking integration tests in `tests/phase1_widget_test.rs`
- [x] docs(widgets): bump framework widget count from 23 to 28 in README
- [x] docs(cargo): update Cargo.toml and README to depend on `dracon-terminal-engine = "27.0.1"` and tag `v27.0.1`
- [x] docs(changelog): append v27.0.1 notes to CHANGELOG.md
- [x] docs(lib): update `on_tick` callback signature comment to include `ctx` argument and bump version doc to 27.0.1
- [x] chore(lock): update Cargo.lock to reflect dependency tree changes (binary diff)
