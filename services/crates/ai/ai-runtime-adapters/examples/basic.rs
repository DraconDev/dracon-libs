use std::time::Duration;

use ai_runtime_adapters::GenericOpenAIAdapter;

fn main() -> anyhow::Result<()> {
    let _adapter = GenericOpenAIAdapter::new_with_auth(
        "sk-test-key".into(),
        "https://api.openai.com/v1".into(),
        "gpt-4o-mini".into(),
        "Authorization".into(),
        "Bearer ".into(),
    )?;

    println!("GenericOpenAIAdapter initialized");
    println!("  model: gpt-4o-mini");
    println!("  endpoint: https://api.openai.com/v1");
    println!("  timeout: {:?}, connect: {:?}", Duration::from_secs(60), Duration::from_secs(10));

    Ok(())
}
