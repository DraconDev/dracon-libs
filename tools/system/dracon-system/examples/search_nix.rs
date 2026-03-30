use dracon_system::SystemAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let agent = SystemAgent::default();
    let info = agent.get_system_info().await?;
    println!("{info}");

    Ok(())
}
