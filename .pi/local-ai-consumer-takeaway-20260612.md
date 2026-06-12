# Local AI consumer inspection

Date: 2026-06-12

Goal: inspect local consumers of the AI library/API inside this workspace and summarize the consumer takeaway.

## Local consumer map

Evidence source: `cargo metadata --no-deps --format-version 1` plus reverse `cargo tree -i` checks.

| Local consumer | Consumes AI crates | Use |
|---|---|---|
| `dracon-ai-runtime-contracts` | `dracon-ai-contracts` | Runtime chat contracts reference routing task types. |
| `ai-routing-runtime` | `dracon-ai-contracts`, `dracon-ai-runtime-contracts` | Routing types and `SmartRouter` use routing constraints and `AiProvider`. |
| `ai-runtime-adapters` | `dracon-ai-runtime-contracts` | `GenericOpenAIAdapter` implements `AiProvider` for `ChatRequest`. |
| `ai-service` | `ai-routing-runtime`, `ai-runtime-adapters`, `dracon-ai-contracts`, `dracon-ai-runtime-contracts` | High-level provider registry/service layer. |
| `ai-runtime-config` | none | Standalone config types; no local dependents. |

## Non-AI local consumers

No non-AI package in the workspace depends directly on:

- `dracon-ai-contracts`
- `dracon-ai-runtime-contracts`
- `ai-runtime-adapters`
- `ai-runtime-config`
- `ai-routing-runtime`
- `ai-service`

The only non-AI source hit for AI-related text is a routing label in `tools/sync/dracon-git/src/dracon_sync_commit.rs`, which is not a crate dependency or API consumer.

## Takeaway

1. **There are no local downstream consumers yet.** The AI crates are internally consumed only by the AI crate family.
2. **Migration risk is concentrated in the AI crate boundary.** Existing local consumers should move to constructor-based setup and avoid struct literals for `#[non_exhaustive]` public types.
3. **`ai-runtime-config` is the biggest loose end.** It has no local dependents yet, so now is the safest time to decide whether it should remain a pure config type crate or gain environment/file resolution behavior.
4. **`GenericOpenAIAdapter` remains the API consumer boundary.** It validates construction inputs and is explicitly non-streaming; consumers needing streaming need a separate provider/trait implementation.
5. **No code changes were required to existing non-AI crates.** The only local documentation change was adding the consumer takeaway to the root README and pointing readers at the AI consumer guide.

## Files updated

- `README.md` — added local AI consumer takeaway and link to `services/crates/ai/README.md`.
- `.pi/local-ai-consumer-takeaway-20260612.md` — this durable inspection report.

## Validation evidence

| Check | Command | Result |
|---|---|---:|
| Local dependency map | `cargo metadata --no-deps --format-version 1` | Shows only AI crates as local AI consumers; `ai-runtime-config` has no local dependents. |
| Reverse dependency check | `cargo tree -i <ai-crate> --workspace --prefix none` | Confirms the same local consumer graph. |
| Source import scan | `/etc/profiles/per-user/dracon/bin/rg -n "use (ai_service|ai_runtime_adapters|ai_runtime_config|ai_routing_runtime|dracon_ai_runtime_contracts|dracon_ai_contracts)|extern crate ...|\\b(... )::" -g '*.rs'` | No non-AI Rust consumers found. |
| Formatting | `cargo fmt --all -- --check` | Pass |
| Build/type check | `cargo check --workspace --all-targets --all-features` | Pass |
| Strict clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Strict rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | Pass |
| Workspace tests | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass |

## Remaining decisions

No blocker was found for local consumers. The only decision to make is whether future local consumers should be added now, especially for `ai-runtime-config`, or whether it should remain standalone until an application needs config resolution.
