use std::sync::Arc;

use ai_routing_runtime::{
    ProviderRegistry, RoutingMessage, RoutingTask, SelectionConstraints, ServiceLevel, SmartRouter,
};
use ai_runtime_adapters::GenericOpenAIAdapter;

struct DummyProvider;

#[tokio::main]
async fn main() {
    let adapter: Arc<dyn ai_routing_runtime::AiModelStore> =
        Arc::new(GenericOpenAIAdapter::new_with_auth(
            "sk-test".into(),
            "https://api.openai.com/v1".into(),
            "gpt-4o-mini".into(),
            "Authorization".into(),
            "Bearer ".into(),
        ));

    println!(
        "RoutingTask lanes: General={:?}, Code={:?}, Research={:?}, Creative={:?}",
        RoutingTask::General,
        RoutingTask::Code,
        RoutingTask::Research,
        RoutingTask::Creative
    );
    println!("ServiceLevel: {:?}", ServiceLevel::default());

    println!("SmartRouter and ProviderRegistry ready");
}
