# ai-lib

Super-simple Rust client for direct AI provider access.

**Bring your own keys. No defaults. No auto-config. No gateway.**

## What this is

A thin async client that calls OpenAI, OpenRouter, NVIDIA, Mistral, DeepSeek,
Apertis, and other OpenAI-compatible providers **directly** from your Rust
code. You pass the API key at the call site. That's it.

## What this is NOT

- **Not** the `ai-api` gateway. That lives in `services/ai-api/`. Use that
  when you want centralized key management and per-consumer rate limits.
- **Not** a key store. The lib never persists your keys.
- **Not** pre-configured. There is no `from_env()`, no `dotenv`, no default
  provider, no model catalog. You must supply every parameter yourself.

## When to use this

Use `ai-lib` when:
- You're building a single-consumer tool that wants direct provider access.
- You want to pay the provider's list price (no gateway markup).
- You want zero moving parts — just your code, the provider, and the wire.

Use `ai-api` (the gateway) when:
- Multiple consumers share a pool of provider keys.
- You want per-consumer auth, rate limits, and usage tracking.
- You want to swap providers without changing consumer code.

## Quick start

Add to `Cargo.toml`:

```toml
[dependencies]
ai-lib = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Chat example:

```rust
use ai_lib::{AiClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), ai_lib::Error> {
    let client = AiClient::new();

    let response = client
        .chat(
            ChatRequest::new("sk-your-openai-key", "https://api.openai.com/v1", "gpt-4o-mini")
                .message(Message::system("You are a helpful assistant."))
                .message(Message::user("Tell me a joke.")),
        )
        .await?;

    println!("{}", response.content);
    Ok(())
}
```

Image example:

```rust
use ai_lib::{AiClient, ImageRequest};

#[tokio::main]
async fn main() -> Result<(), ai_lib::Error> {
    let client = AiClient::new();

    let response = client
        .image(
            ImageRequest::new(
                "sk-your-openai-key",
                "https://api.openai.com/v1",
                "dall-e-3",
                "a corgi wearing a tiny hat",
            )
            .size("1024x1024")
            .n(1),
        )
        .await?;

    for img in response.images {
        println!("URL: {:?}", img.url);
        println!("Revised prompt: {:?}", img.revised_prompt);
    }
    Ok(())
}
```

Anthropic example (uses the Messages API directly — different protocol
from OpenAI: `x-api-key` header, system prompt as a top-level field,
`max_tokens` required):

```rust
use ai_lib::{AiClient, AnthropicRequest, AnthropicMessage};

#[tokio::main]
async fn main() -> Result<(), ai_lib::Error> {
    let client = AiClient::new();

    let response = client
        .anthropic_chat(
            AnthropicRequest::new(
                "sk-ant-your-key",
                "https://api.anthropic.com",
                "claude-sonnet-4-5",
                "You are a helpful assistant.",
                1024, // max_tokens is required by Anthropic
            )
            .message(AnthropicMessage::user("Tell me a joke.")),
        )
        .await?;

    println!("{}", response.content);
    Ok(())
}
```

## Provider metadata

`ai-lib` re-exports the `ai-models-catalog` crate types (`Catalog`, `Provider`, `Model`, `AiModelsConfig`) so callers can share one schema for provider metadata and config files. The catalog crate is a pure data layer — no HTTP, no env reads, no provider adapters — and lives in `DraconDev/dracon-libs` so `dracon-code` and other consumers can use it without depending on the Dracon AI API.

## Provider compatibility

### OpenAI-compatible (`chat()`)

The `chat()` method speaks the OpenAI Chat Completions API. It works with
any provider that implements that protocol — you just point `base_url` at
their host:

| Provider  | base_url                                  |
|-----------|-------------------------------------------|
| OpenAI    | `https://api.openai.com/v1`                |
| OpenRouter| `https://openrouter.ai/api/v1`            |
| NVIDIA NIM| `https://integrate.api.nvidia.com/v1`      |
| Mistral   | `https://api.mistral.ai/v1`                |
| DeepSeek  | `https://api.deepseek.com/v1`              |
| Apertis   | `https://api.apertis.ai/v1`                |

### Anthropic (`anthropic_chat()`)

The `anthropic_chat()` method speaks the Anthropic Messages API directly
(different protocol — see example above). One provider:

| Provider  | base_url                          |
|-----------|-----------------------------------|
| Anthropic | `https://api.anthropic.com`        |

For other providers with non-OpenAI protocols (Google Gemini, MiniMax),
use the OpenAI adapter with a proxy, or call them directly with `reqwest`.

## License

AGPL-3.0.
