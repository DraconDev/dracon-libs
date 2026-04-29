# Project State## Current Focus
Migrating AI service error handling to `anyhow`, standardizing provider registry type, adding a runtime example, and refactoring the terminal UI codebase.

## Completed- [x] Updated `services/crates/ai/ai-routing-runtime` imports to use `dracon_ai_contracts` types (`RoutingTask`, `SelectionConstraints`, `ServiceLevel`) and removed unused dependencies.
- [x] Simplified routing module exports to expose only `SelectionConstraints`.
- [x] Added `services/crates/ai/ai-service/examples/basic.rs` showcasing `GenericOpenAIAdapter` usage with provider registration and a sample async call.
- [x] Refactored `ProviderRegistry` in `services/crates/ai/ai-service/src/lib.rs` to store `Arc<dyn AiProvider>` directly, removing downcast logic and updating `register`/`get` signatures accordingly.
- [x] Modified `AiService::ask` to retrieve the provider via the new generic registry API, eliminating the concrete `GenericOpenAIAdapter` type.
- [x] Converted error handling in `tools/system/dracon-system/src/lib.rs` to `anyhow::Result` and `Context`, streamlining error propagation and providing context for failures.
- [x] Refactored several terminal UI components (`engine.rs`, `filter.rs`, `plane.rs`, `utils.rs`, `widgets/editor.rs`, `widgets/input.rs`, etc.) by removing legacy code paths and simplifying imports.
- [x] Adjusted numerous module-level comments and documentation to reflect the new type‑definitions and removed dependencies.
- [x] Updated Cargo.lock (binary change only, no functional impact).
