use std::sync::Arc;

use ai_runtime_adapters::GenericOpenAIAdapter;
use ai_service::{AiService, LaneModelPolicy, ProviderRegistry, DEFAULT_PROVIDER};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let adapter = GenericOpenAIAdapter::new_with_auth(
        "sk-test".into(),
        "https://api.openai.com/v1".into(),
        "gpt-4o-mini".into(),
        "Authorization".into(),
        "Bearer ".into(),
    )?;

    let mut registry = ProviderRegistry::new();
    registry.register(DEFAULT_PROVIDER, Arc::new(adapter));

    let svc = AiService::new(registry, LaneModelPolicy::default());

    let request = ai_service::ChatRequest {
        project_id: "example".into(),
        messages: vec![ai_service::ChatMessage {
            role: "user".into(),
            content: "What is 2+2?".into(),
        }],
        client_intent: None,
        max_tokens: Some(50),
        temperature: Some(0.7),
        stream: false,
    };

    let response = svc.ask(request).await?;
    println!("Response: {}", response);

    Ok(())
}
