# Project State

## Current Focus
Introduce a command‑driven architecture with TOML configuration and CLI execution support for AI integration

## Completed
- [x] Added `commands: RefCell<Vec<BoundCommand>>` field to `App`
- [x] Implemented `App::from_toml` constructor that builds an `App` from a TOML configuration file
- [x] Added `App::add_command` method to register a `BoundCommand` globally
- [x] Added `App::available_commands` method to enumerate all registered commands across widgets
- [x] Modified `render_frame` invocation to pass `&self.commands` to the rendering context
- [x] Extended `Ctx<'a>` struct with `commands: &'a RefCell<Vec<BoundCommand>>` field
- [x] Added `Ctx::run_command` method that executes a shell command synchronously and returns `(stdout, stderr, exit_code)`
- [x] Added `Ctx::available_commands` method to retrieve all widget commands for AI introspection
- [x] Updated all rendering and tick handling calls to forward the `commands` reference
- [x] Introduced `CommandRunner` usage for synchronous command execution
- [x] Provided documentation comments for the new command‑related APIs
- [x] Integrated command handling into the frame rendering pipeline via `commands` parameter
- [x] Exposed global command registry to AI through `available_commands` on both `App` and `Ctx`
