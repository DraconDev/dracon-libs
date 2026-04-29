# dracon-memory-runtime

SQLite + ONNX embeddings for conversation storage and semantic search.

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

## Environment Variables

- `DRACON_MODEL_PATH` — path to ONNX model (default: `assets/bge-small-en-v1.5.onnx`)
- `DRACON_TOKENIZER_PATH` — path to tokenizer (default: `assets/tokenizer.json`)

## Key Types

- [`MemorySystem`] — main entry point combining DB + embedder
- [`MemoryDb`] — SQLite persistence
- [`OnnxEmbedder`] — ONNX-based text embedding

## License

MIT OR Apache-2.0
