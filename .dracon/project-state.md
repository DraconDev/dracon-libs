# Project State

## Current Focus
Refactor event handling and thread spawning to use struct fields and simplify API

## Completed
- [x] Use dereferenced key reference `*k` and wrap dispatch in `let _ =` to suppress result
- [x] Restructure mouse event dispatch by pattern‑matching into a struct and forwarding its fields
- [x] Remove the explicit `app: &mut self` argument from the `spawn_thread` call
- [x] Update Cargo.lock (binary update, no functional change)
