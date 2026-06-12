# AI consumer impact verification

Date: 2026-06-12

Goal: clarify and address consumer impact for users of the AI library and AI API.

## Consumer question answered

Consumers fall into two groups:

1. **AI library consumers** — Rust applications that depend directly on the AI crates, register providers, build requests, or route models in-process.
2. **AI API consumers** — users or services that call an OpenAI-compatible AI API through `GenericOpenAIAdapter` or through an application built on `ai-service`.

The consumer-facing behavior is now documented and verified:

- Public AI structs/enums use `#[non_exhaustive]`; consumers should use constructors/builders instead of struct literals.
- `ai-runtime-adapters::GenericOpenAIAdapter` validates API key, endpoint, model, and auth header configuration at construction time.
- The adapter uses workspace `reqwest` with rustls-only TLS posture and redacts API keys in `Debug`.
- The adapter is explicitly non-streaming and rejects `ChatRequest::stream = true`.
- `ai-runtime-config`, `ai-service`, and `ai-routing-runtime` expose constructors for common consumer setup.
- README examples were corrected to use constructors rather than non-exhaustive struct literals.
- A new AI crates consumer guide was added at `services/crates/ai/README.md`.

## Files changed

- `services/crates/ai/README.md` — new consumer guide for AI library and AI API consumers.
- `services/crates/ai/ai-service/README.md` — updated usage example and compatibility note.
- `services/crates/ai/ai-runtime-adapters/README.md` — compatibility note.
- `services/crates/ai/ai-runtime-config/README.md` — updated constructor-based example.
- `services/crates/ai/ai-routing-runtime/README.md` — compatibility note.
- `services/crates/ai/ai-service/src/lib.rs` — added `LaneModelPolicy::new` and `enabled_with_default_lane`.
- `services/crates/ai/ai-routing-runtime/src/lib.rs` — added `RoutingTrace::new`.
- `contracts/crates/ai/dracon-ai-runtime-contracts/src/models.rs` — added `ChatResponse::new`.

## Validation evidence

| Check | Command | Result |
|---|---|---:|
| Formatting | `cargo fmt --all -- --check` | Pass |
| Build/type check | `cargo check --workspace --all-targets --all-features` | Pass |
| Strict clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| Strict rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | Pass |
| Workspace tests | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass |
| Consumer API grep | `rg -n "OpenAIProviderSpec \\{|AiRuntimeConfig \\{|RoutingTrace \\{|LaneModelPolicy \\{|ChatRequest \\{|ChatMessage \\{" services/crates/ai contracts/crates/ai -g '*.md' -g '*.rs'` | Only definitions/internal struct literals remain |

## Remaining decisions

No remaining consumer/API blocker was found for the documented behavior. Real streaming remains intentionally unsupported by `GenericOpenAIAdapter`; consumers that need streaming need a separate provider/trait implementation.
