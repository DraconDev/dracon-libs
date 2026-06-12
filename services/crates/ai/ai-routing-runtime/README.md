# ai-routing-runtime

Generic AI provider routing utilities.

See the AI crates consumer guide for compatibility expectations for library and AI API consumers.

This crate provides `ProviderRegistry<T>` and `SmartRouter<T>` for selecting providers by active/dev model lists and optional lane policy.

## Usage

```rust
use std::sync::Arc;
use ai_routing_runtime::{ProviderRegistry, SmartRouter};
use ai_runtime_adapters::GenericOpenAIAdapter;

# fn example() -> anyhow::Result<()> {
let adapter = GenericOpenAIAdapter::new_with_auth(
    std::env::var("EXAMPLE_AI_API_KEY")?,
    "https://api.example.com/v1".to_string(),
    "example-model".to_string(),
    "Authorization".to_string(),
    "Bearer ".to_string(),
)?;

let mut registry = ProviderRegistry::new();
registry.register("example-model", Arc::new(adapter));

let router = SmartRouter::new(
    registry,
    Vec::new(),
    vec!["example-model".to_string()],
    None,
);
# Ok(())
# }
```

## Routing behavior

`route_with_trace()` currently selects the first available active model, then the first dev model if no active models exist. It returns a `RoutingTrace` containing the selected model and optional lane.

## Features

This crate has no optional Cargo features today.

## License

AGPL-3.0-only
