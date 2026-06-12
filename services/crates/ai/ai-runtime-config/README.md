# ai-runtime-config

Configuration types for AI runtime providers.

This crate defines `OpenAIProviderSpec` and `AiRuntimeConfig`, plus `resolve_ai_runtime_config()`.

## Usage

```rust
use ai_runtime_config::{resolve_ai_runtime_config, AiRuntimeConfig, OpenAIProviderSpec};

let config = resolve_ai_runtime_config();
assert!(config.openai_providers.is_empty());

let provider = OpenAIProviderSpec {
    model_id: "example-model".to_string(),
    endpoint: "https://api.example.com/v1".to_string(),
    payload_model: "example-model".to_string(),
    api_keys: Vec::new(),
    auth_header_name: "Authorization".to_string(),
    auth_header_prefix: "Bearer ".to_string(),
};

let config = AiRuntimeConfig {
    openai_providers: vec![provider],
    active_model_ids: vec!["example-model".to_string()],
    dev_model_ids: Vec::new(),
    lane_model_policy: None,
};
```

## Configuration resolution

`resolve_ai_runtime_config()` currently returns deterministic empty defaults. It does not read environment variables or files yet. Callers that need environment/file loading should build `AiRuntimeConfig` explicitly until that behavior is implemented.

## Features

This crate has no optional Cargo features today.

## Security notes

Do not commit API keys in config files. Prefer environment variables, secret managers, or encrypted configuration for production deployments.

## License

AGPL-3.0-only
