use ai_runtime_config::{AiRuntimeConfig, OpenAIProviderSpec};

fn main() -> anyhow::Result<()> {
    let config = AiRuntimeConfig::new(
        vec![OpenAIProviderSpec::new(
            "gpt-4o-mini",
            "https://api.openai.com/v1",
            "gpt-4o-mini",
            vec!["sk-test".into()],
            "Authorization",
            "Bearer ",
        )],
        vec!["gpt-4o-mini".into()],
        vec!["gpt-4o-mini-dev".into()],
        None,
    );

    println!(
        "Config loaded: {} provider(s)",
        config.openai_providers.len()
    );
    println!("Active models: {:?}", config.active_model_ids);

    let json = serde_json::to_string_pretty(&config)?;
    println!("\nJSON:\n{}", json);

    Ok(())
}
