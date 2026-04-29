# Project State

## Current Focus
Add new runtime crates and feature flags for memory, file handling, speech‑to‑text, text‑to‑speech, system utilities, and streamline the terminal visual subsystem.

## Completed
- [x] Added memory contracts and runtime implementation with ONNX embeddings and SQLite backing.
- [x] Implemented file system operations crate with categorization, search, and recursive copy features.
- [x] Added speech‑to‑text runtime supporting Parakeet and Whisper back‑ends.
- [x] Added text‑to‑speech runtime supporting Kitten and Kokoro engines via feature flags.
- [x] Refactored memory system runtime to use SQLite for persistent stores.
- [x] Enhanced Git service with asynchronous API and libgit2 + CLI fallback, plus new integration tests.
- [x] Added system utilities crate with diagnostics, SSH remote execution, and notification support.
- [x] Stripped down terminal visual subsystem: removed image, shape, slicer, tiles, and rich widget modules; reduced `Icon` enum and simplified asset generation.
- [x] Updated Cargo.toml files across AI, memory, and service crates to reflect new binary sizes, dependencies, and feature flags.
- [x] Revised README to describe new workspace structure, crate purposes, feature flags, and usage examples.
- [x] Added comprehensive README files for the new `dracon-files`, `dracon-stt-runtime`, `dracon-tts-runtime`, `dracon-memory-runtime`, and `dracon-git` crates.
