# dracon-memory-runtime

SQLite + ONNX embeddings for conversation storage and semantic search.

## Privacy and retention

`MemoryDb` stores conversations and facts in SQLite. File-backed databases are plaintext unless the deployment places them on encrypted storage or applies OS-level permissions/backups. Use `delete_conversation(id)`, `delete_fact(category, key)`, `delete_facts_by_category(category)`, or `delete_conversations_before(timestamp)` to remove individual records instead of relying only on `clear()`.

## Usage

```rust
use dracon_memory_runtime::{MemorySystem, Role};

let mem = MemorySystem::new(":memory:")?;

// Store and recall
mem.store_conversation(Role::User, "I prefer dark mode").await?;
mem.store_conversation(Role::Assistant, "Got it!").await?;

let results = mem.recall_relevant("display settings?", 3).await?;

// Facts
mem.store_fact("preferences", "theme", "dark", None).await?;
let fact = mem.get_fact("preferences", "theme").await?;
```

## Contract/runtime alignment

The runtime exposes synchronous, embedding-aware methods such as `store_conversation(role, content)` and `OnnxEmbedder::embed(&mut self)`. The separate `dracon-memory-contracts` crate exposes async traits for consumers that should not depend on ONNX/SQLite. Prefer the runtime API for direct use and the contract crate for trait-based integration.

## Environment Variables

- `DRACON_MODEL_PATH` — path to ONNX model (default: `assets/bge-small-en-v1.5.onnx`)
- `DRACON_TOKENIZER_PATH` — path to tokenizer (default: `assets/tokenizer.json`)

## Key Types

- [`MemorySystem`] — main entry point combining DB + embedder
- [`MemoryDb`] — SQLite persistence
- [`OnnxEmbedder`] — ONNX-based text embedding

## License

AGPL-3.0-only
