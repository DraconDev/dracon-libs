# Project State

## Current Focus
Update documentation examples to use the new `App` builder pattern and reflect the version bump to v26.0.1.

## Completed
- [x] Replace old `Terminal::new` and manual `Plane` rendering with `App::new().unwrap().title(...).fps(...).on_tick(...).run(...)`
- [x] Add example of creating a list pane using `List` and `add_plane`
- [x] Remove deprecated `split_h` example code
- [x] Update version comment from `v19.2.2` to `v26.0.1` in the lib.rs example
- [x] Adjust imports to reference `framework::prelude::*` instead of older modules
- [x] Clean up example comments to reflect current API usage
No ongoing or pending items. All listed changes are fully implemented.
