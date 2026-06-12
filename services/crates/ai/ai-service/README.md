# ai-service

High-level AI service layer for `dracon-libs`.

See the AI crates consumer guide for compatibility expectations for library and AI API consumers.

`ai-service` combines a provider registry with a simple lane model policy and exposes `AiService::ask()` for OpenAI-compatible providers.

## Usage

```rust
use std::sync::Arc;
use ai_runtime_adapters::GenericOpenAIAdapter;
use ai_service::{AiService, ProviderRegistry, DEFAULT_PROVIDER};

# async fn example() -> anyhow::Result<()> {
let adapter = GenericOpenAIAdapter::new_with_auth(
    std::env::var("EXAMPLE_AI_API_KEY")?,
    "https://api.example.com/v1".to_string(),
    "example-model".to_string(),
    "Authorization".to_string(),
    "Bearer ".to_string(),
)?;

let mut registry = ProviderRegistry::new();
registry.register(DEFAULT_PROVIDER, Arc::new(adapter));
let service = AiService::new(registry, Default::default());

let request = ai_service::ChatRequest::new(
    "example-project",
    vec![ai_service::ChatMessage::new("user", "What is 2+2?")],
)
.with_max_tokens(Some(50));
let text = service.ask(request).await?;
# Ok(())
# }
```

The example uses an environment-provided API key. Do not commit real credentials.

## Features

This crate has no optional Cargo features today.

## Security notes

- API keys are held in memory by the selected provider.
- Prefer environment variables or secret managers for real deployments.
- Provider `Debug` output redacts secret values where the adapter implements redaction.

## License

AGPL-3.0-only
