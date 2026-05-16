use ai_runtime_config::{AiRuntimeConfig, OpenAIProviderSpec};

fn main() -> anyhow::Result<()> {
    let config = AiRuntimeConfig {
        openai_providers: vec![OpenAIProviderSpec {
            model_id: "gpt-4o-mini".into(),
            endpoint: "https://api.openai.com/v1".into(),
            payload_model: "gpt-4o-mini".into(),
            api_keys: vec!["sk-test".into()],
            auth_header_name: "Authorization".into(),
            auth_header_prefix: "Bearer ".into(),
        }],
        active_model_ids: vec!["gpt-4o-mini".into()],
        dev_model_ids: vec!["gpt-4o-mini-dev".into()],
        lane_model_policy: None,
    };

    println!(
        "Config loaded: {} provider(s)",
        config.openai_providers.len()
    );
    println!("Active models: {:?}", config.active_model_ids);

    let json = serde_json::to_string_pretty(&config)?;
    println!("\nJSON:\n{}", json);

    Ok(())
}
