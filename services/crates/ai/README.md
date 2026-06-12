# AI crates consumer guide

This directory contains the AI contract, config, adapter, routing, and service crates. It is intended for two consumer groups:

- **AI library consumers**: Rust code that depends on these crates directly and registers providers, builds requests, or routes models inside an application.
- **AI API consumers**: users or services that call an OpenAI-compatible AI API through `GenericOpenAIAdapter` or an application built on `ai-service`.

## Compatibility expectations

- Public structs and enums are `#[non_exhaustive]`. Use constructors and builder methods instead of struct literals so consumers keep compiling across minor releases.
- Prefer crate-level constructors such as `ChatRequest::new`, `ChatMessage::new`, `AiRuntimeConfig::new`, `OpenAIProviderSpec::new`, `LaneModelPolicy::new`, and `RoutingMessage::new`.
- The generic OpenAI adapter validates endpoint URL, model, API key, and auth header configuration at construction time.
- `GenericOpenAIAdapter` is non-streaming. It rejects `ChatRequest::stream = true` rather than pretending to stream.
- API keys are held in memory by adapters. Do not commit secrets; use environment variables or secret managers.

## Library consumer path

Use the crates directly when you need in-process provider registration and routing:

1. Create or implement an `AiProvider`.
2. Register it with `ProviderRegistry` or `ai_service::ProviderRegistry`.
3. Build a `ChatRequest` with constructors/builders.
4. Call `AiProvider::ask_and_collect()` or `AiService::ask()` for collected responses.

## AI API consumer path

Use `ai-runtime-adapters::GenericOpenAIAdapter` when your provider exposes an OpenAI-compatible Chat Completions endpoint:

```rust
let adapter = ai_runtime_adapters::GenericOpenAIAdapter::new_with_auth(
    std::env::var("AI_API_KEY")?,
    "https://api.example.com/v1".to_string(),
    "example-model".to_string(),
    "Authorization".to_string(),
    "Bearer ".to_string(),
)?;
```

The adapter posts to `<endpoint>/chat/completions`, sends the configured auth header, and returns collected text plus an optional finish reason. It does not expose a streaming response API.

## Crate map

| Crate | Consumer use |
|---|---|
| `dracon-ai-runtime-contracts` | Shared `ChatRequest`, `ChatMessage`, `ChatResponse`, and `AiProvider` traits. |
| `ai-runtime-adapters` | OpenAI-compatible adapter implementation. |
| `ai-runtime-config` | Provider/config structs and constructors. |
| `ai-routing-runtime` | Provider registry and model routing utilities. |
| `ai-service` | High-level service combining registry, default provider selection, and lane policy. |

## License

AGPL-3.0-only
