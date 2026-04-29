# Project State

## Current Focus
Improve voice resolution and Git information handling, update dependency synchronization, and optimize remote execution.

## Completed
- **`tools/media/dracon-tts-runtime/src/kokoro.rs`**: Implement voice resolution and fetch voice description from a list.
- **`tools/sync/dracon-git/src/cli.rs`**: Ignore assignment of default values and address synthesizer naming - add check for `changed` in line parsing and populate git stats if line is `changed`.
- **`tools/sync/dracon-git/src/intent.rs`**: Address syntax error related to usage of indentation symbols in Rust, and handle git log extraction from active board.
- **`tools/sync/dracon-git/src/lib.rs`**: Synchronize git service information including status tracking, delegates, and finding file pattern. Allow tracking of git commit stats.
- **`tools/system/dracon-system/src/lib.rs`**: Include evaluation of system OS release data to generate system agent response.
- **`tools/system/dracon-system/src/remote.rs`**: Incorporate secure remote execution with system SSH settings and program using reverse ssh tunnel.
- **Chore sync(Cargo.lock)**: Synchronize dependencies across core crates after security speculation, apply changes across tool.
