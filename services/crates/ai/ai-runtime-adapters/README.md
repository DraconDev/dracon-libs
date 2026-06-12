# ai-runtime-adapters

OpenAI-compatible AI provider adapters.

This crate currently provides `GenericOpenAIAdapter`, a non-streaming adapter for OpenAI Chat Completions-compatible endpoints.

## Usage

```rust
use ai_runtime_adapters::GenericOpenAIAdapter;

# fn example() -> anyhow::Result<()> {
let adapter = GenericOpenAIAdapter::new_with_auth(
    std::env::var("EXAMPLE_AI_API_KEY")?,
    "https://api.example.com/v1".to_string(),
    "example-model".to_string(),
    "Authorization".to_string(),
    "Bearer ".to_string(),
)?;
# Ok(())
# }
```

Use an environment variable for the API key in real applications. Do not commit secrets or fake keys that resemble production credentials.

## Features

This crate has no optional Cargo features today.

## Streaming behavior

`ChatRequest::stream` is accepted by the shared AI contract, but `GenericOpenAIAdapter` collects the full response and returns it through `AiProvider::ask_and_collect()`. It does not currently expose a real streaming API.

## License

AGPL-3.0-only
