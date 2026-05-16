# dracon-libs

Vertical Rust libraries — a collection of reusable tools and runtimes for AI, system operations, media processing, and terminal UI.

## What's here

```
dracon-libs (workspace root, version 29.1.0)
├── tools/
│   ├── sync/dracon-git              # Git operations (libgit2 + CLI fallback)
│   ├── tui/dracon-terminal-engine  # Terminal compositor with z-index layers
│   ├── system/dracon-system         # System monitoring, SSH, notifications
│   ├── files/dracon-files           # File system operations and FsCatalog
│   ├── media/dracon-tts-runtime    # Text-to-speech (Kitten, Kokoro)
│   ├── media/dracon-stt-runtime    # Speech-to-text (Parakeet, Whisper)
│   ├── media/dracon-video-runtime  # Video processing (FFmpeg + ML)
│   └── memory/dracon-memory-runtime # SQLite + ONNX embeddings
├── contracts/
│   ├── dracon-memory-contracts     # MemoryStore + TextEmbedder traits
│   ├── ai/dracon-ai-contracts     # RoutingTask, SelectionConstraints
│   └── ai/dracon-ai-runtime-contracts  # ChatMessage, AiProvider trait
└── services/
    └── ai/
        ├── ai-service               # Provider registry and AI service layer
        ├── ai-routing-runtime      # SmartRouter for model selection
        ├── ai-runtime-config       # Runtime config types
        └── ai-runtime-adapters     # OpenAI-compatible adapter
```

## Crates

### Tool Crates

| Crate | Description | Feature Flags |
|-------|-------------|---------------|
| `dracon-git` | Git operations with libgit2 and CLI fallback | — |
| `dracon-terminal-engine` | Terminal compositor: z-index layers, TrueColor, SGR mouse, Kitty keyboard | — |
| `dracon-system` | System monitoring, SSH remoting, notifications | — |
| `dracon-files` | File categorization, search, recursive copy | — |
| `dracon-tts-runtime` | Text-to-speech (Kitten/Kokoro) | `kitten`, `kokoro` |
| `dracon-stt-runtime` | Speech-to-text (Parakeet/Whisper) | `parakeet`, `whisper` |
| `dracon-video-runtime` | Video processing with FFmpeg + ML | `ffmpeg` |
| `dracon-memory-runtime` | SQLite + ONNX embeddings for semantic search | — |

### Contract Crates

| Crate | Description |
|-------|-------------|
| `dracon-memory-contracts` | `MemoryStore` and `TextEmbedder` async traits |
| `dracon-ai-contracts` | `RoutingTask`, `SelectionConstraints`, `ServiceLevel` |
| `dracon-ai-runtime-contracts` | `ChatMessage`, `ChatRequest`, `AiProvider` trait |

### Service Crates

| Crate | Description |
|-------|-------------|
| `ai-service` | Provider registry and policy-driven AI service |
| `ai-routing-runtime` | `SmartRouter` with model selection |
| `ai-runtime-config` | `OpenAIProviderSpec` and `AiRuntimeConfig` types |
| `ai-runtime-adapters` | `GenericOpenAIAdapter` for OpenAI-compatible APIs |

## Importing

Add individual crates to your `Cargo.toml`:

```toml
[dependencies]
dracon-git = { git = "https://github.com/DraconDev/dracon-libs" }
dracon-system = { git = "https://github.com/DraconDev/dracon-libs" }
dracon-memory-runtime = { git = "https://github.com/DraconDev/dracon-libs" }
```

Or use the workspace directly:

```toml
[workspace]
members = ["path/to/dracon-libs/tools/sync/dracon-git"]
```

## Quick Start

### Git Operations

```rust
use dracon_git::GitService;

let git = GitService::new("/path/to/repo")?;
let status = git.get_status().await?;
println!("Clean: {}", status.is_clean);
```

### Terminal Engine

```rust
use dracon_terminal_engine::{Terminal, integration::ratatui::RatatuiBackend};
use std::io::stdout;

let mut terminal = Terminal::new(RatatuiBackend::new(stdout())?)?;
terminal.draw(|f| { /* ratatui widgets */ })?;
```

### Memory & Semantic Search

```rust
use dracon_memory_runtime::{MemorySystem, Role};

let mem = MemorySystem::new(":memory:")?;
mem.store_conversation(Role::User, "I prefer dark mode").await?;
let results = mem.recall_relevant("what's my display preference?", 3).await?;
```

## Principles

1. **Vertical ownership** — each crate owns its contracts, types, and implementation
2. **Self-contained** — minimal internal path dependencies between crates
3. **Feature flags** — heavy deps (TTS, STT, video) are opt-in via Cargo features
4. **No kitchen-sink** — no "common" or "utils" crate

## Testing

```bash
# Check the workspace compiles
cargo check --workspace

# Run tests (requires alsa-lib and pkg-config for media crates)
cargo test --workspace --all-targets

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Contributing

- Crates follow the workspace `edition = "2021"` and `version.workspace = true` conventions
- Public APIs must be documented; `#![warn(missing_docs)]` is enabled on all library crates
- Add `#[should_panic]` tests for error paths
- Integration tests live in `tests/` directories (not inline `#[cfg(test)]` modules)

## Breaking Changes (v29.0.0+)

### P1 Reliability — Result-Based APIs

All methods that can fail now return `Result` types instead of panicking on error:

- **`TextToSpeech::speak()`** and **`TextToSpeech::stop()`** now return `TtsResult<()>` (previously `()`)
- **`VoiceProvider::set_voice()`** now returns `TtsResult<bool>` (previously `bool`)
- **`VoiceProvider::current_voice()`** now returns `TtsResult<VoiceInfo>` (previously `VoiceInfo`)
- **`GenericOpenAIAdapter::new_with_auth()`** now returns `anyhow::Result<Self>` (previously `Self`)
- **`ParakeetStt::new()`** now returns `anyhow::Result<Self>` (previously `Self`)
- **`KittenTTS::new_with_voice()`** and **`KokoroTts::new_with_voice()`** constructors now return `Result`
- **`KittenTTS::set_voice()`**, **`KittenTTS::get_voice()`**, **`KokoroTts::set_voice()`**, **`KokoroTts::get_voice()`** now return `Result`

Updated callers must handle `Result` types:

```rust
// Before
tts.speak("hello");

// After
tts.speak("hello")?;
```

### P0 Security — `run_command()` now requires explicit approval

`SystemAgent::run_command()` is marked `unsafe` and requires callers to first call `approve_command()` to whitelist the specific command. See `dracon-system/README.md` for details.

## License

This project is dual-licensed:

- **AGPL-3.0-only** — See [LICENSE](LICENSE) for the full text. This is the default license for open source use.
- **Commercial License** — For organizations that prefer not to comply with AGPLv3's source disclosure requirements. See [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md) for details.

By contributing to this project, you agree to the terms in [CLA.md](CLA.md).