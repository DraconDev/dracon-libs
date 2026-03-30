# dracon-libs

Vertical Rust libraries - public collection of reusable tools and runtimes.

## What's here

```
dracon-libs (public)
  └── tools/
      ├── sync/          → dracon-git
      ├── tui/           → dracon-terminal-engine
      ├── system/        → dracon-system
      ├── media/         → dracon-tts-runtime, dracon-stt-runtime, dracon-video-runtime
      └── memory/        → dracon-memory-runtime
```

**Services** (AI, email, bucket) moved to [dracon-private-services](https://github.com/DraconDev/dracon-private-services).

## Crates

| Crate | Purpose |
|-------|---------|
| `dracon-git` | Git operations (libgit2 with CLI fallback) |
| `dracon-terminal-engine` | Terminal compositor with built-in code editor |
| `dracon-system` | System monitoring, SSH, notifications |
| `dracon-memory-runtime` | Embeddings and memory utilities |
| `dracon-tts-runtime` | Text-to-speech (feature-gated, requires audio deps) |
| `dracon-stt-runtime` | Speech-to-text (feature-gated) |
| `dracon-video-runtime` | Video processing (feature-gated) |

## Importing

```toml
[dependencies]
dracon-git = { git = "https://github.com/DraconDev/dracon-libs" }
dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-libs" }
```

Or import multiple:
```toml
[dependencies]
dracon-libs = { git = "https://github.com/DraconDev/dracon-libs", features = ["git", "terminal"] }
```

## Usage

```rust
use dracon_git::GitService;
use dracon_terminal_engine::Terminal;

// Git operations
let git = GitService::new("/path/to/repo");
git.status()?;

// Terminal
let terminal = Terminal::new()?;
```

## Testing

```bash
cargo check -p dracon-git -p dracon-terminal-engine -p dracon-system
cargo test --workspace
```

## Principles

1. **Vertical ownership** — each crate owns its contracts, types, and implementation
2. **Self-contained** — minimal internal path dependencies
3. **Feature flags** — heavy deps (TTS, STT, video) are opt-in
4. **No kitchen-sink** — no "common" or "utils" crate

## Archived

Deprecated crates in `archive/`.