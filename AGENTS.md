# AGENTS.md — dracon-libs Agent Handbook

This document provides guidance for agents working on the dracon-libs Rust workspace.

## Repository

- **URL**: https://github.com/DraconDev/dracon-libs
- **License**: MIT OR Apache-2.0
- **Edition**: 2021 (all crates)
- **Workspace version**: 29.1.0

## Workspace Structure

```
dracon-libs/
├── tools/
│   ├── sync/dracon-git           # Git operations (libgit2 + CLI fallback)
│   ├── tui/dracon-terminal-engine # Terminal compositor (z-index, TrueColor, SGR mouse)
│   ├── system/dracon-system       # System monitoring, SSH, notifications
│   ├── files/dracon-files         # File system operations and FsCatalog
│   ├── media/dracon-tts-runtime  # Text-to-speech (Kitten, Kokoro)
│   ├── media/dracon-stt-runtime  # Speech-to-text (Parakeet, Whisper)
│   ├── media/dracon-video-runtime # Video processing (FFmpeg + ML)
│   └── memory/dracon-memory-runtime # SQLite + ONNX embeddings
├── contracts/
│   ├── dracon-memory-contracts  # MemoryStore + TextEmbedder traits
│   ├── ai/dracon-ai-contracts   # RoutingTask, SelectionConstraints
│   └── ai/dracon-ai-runtime-contracts # ChatMessage, AiProvider trait
└── services/ai/
    ├── ai-service               # Provider registry and AI service layer
    ├── ai-routing-runtime      # SmartRouter for model selection
    ├── ai-runtime-config       # Runtime config types
    └── ai-runtime-adapters     # GenericOpenAIAdapter for OpenAI-compatible APIs
```

## Key Conventions

### Workspace Metadata

All crates use `*.workspace = true` for shared metadata:

```toml
[package]
version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
edition.workspace = true
```

### Async/Await

- All async functions use `async fn` with `tokio` runtime
- Avoid blocking in async context; use `tokio::task::spawn_blocking` for CPU-bound work
- Use `futures::executor::block_on` only for synchronous entry points (trait impls)

### Error Handling

- **Primary pattern**: `anyhow::Result<T>` for public APIs
- **Constructors** that can fail return `anyhow::Result<Self>`
- **Trait methods** that can fail return `T` or `Result<T>` per the trait signature
- Do NOT use `.expect()` or `.unwrap()` in library code (except in tests)
- Do NOT use `panic!` for expected error conditions

### Thread Safety

- Types used across async boundaries must implement `Send + Sync` where needed
- Mutex guards should be held for minimal duration
- Use `std::sync::Mutex` for sync contexts, `tokio::sync::Mutex` for async contexts

### Documentation

- All public items must have doc comments (`///` or `//!`)
- `#![warn(missing_docs)]` is enabled on all library crates
- Add `SAFETY` comments for `unsafe` blocks explaining invariants

## P0 Security Rules

1. **`SystemAgent::run_command()`** — marked `unsafe`, requires prior `approve_command()` call
2. **SSH paths** — program path must be shell-escaped; args are already escaped via `shell_escape()`
3. **Path traversal** — use `canonicalize()` + prefix check; reject any path that escapes the repo root
4. **PID operations** — verify ownership via `/proc/{pid}/status` `Uid:` field before signaling
5. **Package names** — sanitize to alphanumeric + hyphen, max 100 chars; no special characters

## Testing

```bash
# Check workspace compiles (excludes media crates that need system libs)
cargo check --workspace --exclude dracon-tts-runtime --exclude dracon-stt-runtime

# Full check (requires alsa-lib dev headers)
cargo check --workspace

# Run tests
cargo test --workspace --all-targets

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

**Note**: `dracon-tts-runtime` and `dracon-stt-runtime` require `alsa-lib` development headers and `pkg-config` to compile. On NixOS: `nix-shell -p alsaLib pkgconfig`.

## Common Tasks

### Adding a New Crate

1. Add to `[workspace.members]` in root `Cargo.toml`
2. Add `version.workspace = true` etc. to the crate's `[package]`
3. Run `cargo check -p <crate>` to verify compilation
4. Add to this document if it introduces new patterns agents should know

### Adding Dependencies

1. Add to `[workspace.dependencies]` in root `Cargo.toml`
2. Reference with `dep = { workspace = true }` in crate `Cargo.toml`
3. Run `cargo check --workspace` to verify

### Breaking Changes

When making breaking API changes:
1. Document in this file under the relevant section
2. Update the **Breaking Changes** section in `README.md`
3. Bump the workspace version in `Cargo.toml` following semver

## Version History

- **v29.1.0** (current) — P0 security hardening, P1 reliability (Result-based APIs), P2 workspace consistency
- **v12.6.0**, **v11.0.0** — prior releases
