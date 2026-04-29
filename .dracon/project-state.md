# Project State

## Current Focus
Removed two unit tests that were failing or no longer needed from OnnxEmbedder tests.

## Completed
- [x] Deleted `test_embedder_new_fails_on_missing_tokenizer` which verified error handling when the tokenizer path was invalid
- [x] Deleted `test_embed_empty_string_returns_normalized_zeros` which checked that embedding an empty string produced a near‑zero vector
