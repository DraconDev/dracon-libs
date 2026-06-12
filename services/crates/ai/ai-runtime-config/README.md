# ai-runtime-config

Configuration types for AI runtime providers.

See the AI crates consumer guide for compatibility expectations for library and AI API consumers.

This crate defines `OpenAIProviderSpec` and `AiRuntimeConfig`, plus `resolve_ai_runtime_config()`.

## Usage

```rust
use ai_runtime_config::{resolve_ai_runtime_config, AiRuntimeConfig, OpenAIProviderSpec};

let config = resolve_ai_runtime_config();
assert!(config.openai_providers.is_empty());

let provider = OpenAIProviderSpec::new(
    "example-model",
    "https://api.example.com/v1",
    "example-model",
    Vec::new(),
    "Authorization",
    "Bearer ",
);

let config = AiRuntimeConfig::new(
    vec![provider],
    vec!["example-model".to_string()],
    Vec::new(),
    None,
);
```

## Configuration resolution

`resolve_ai_runtime_config()` currently returns deterministic empty defaults. It does not read environment variables or files yet. Callers that need environment/file loading should build `AiRuntimeConfig` explicitly until that behavior is implemented.

## Features

This crate has no optional Cargo features today.

## Security notes

Do not commit API keys in config files. Prefer environment variables, secret managers, or encrypted configuration for production deployments.

## License

AGPL-3.0-only
