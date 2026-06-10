use std::sync::Arc;

use ai_routing_runtime::{ProviderRegistry, RoutingMessage, SmartRouter};
use async_trait::async_trait;
use dracon_ai_runtime_contracts::{models::ChatRequest, traits::AiProvider};

struct DummyProvider;

#[async_trait]
impl AiProvider for DummyProvider {
    async fn ask_and_collect(
        &self,
        _request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)> {
        Ok(("dummy".to_string(), Some("stop".to_string())))
    }
}

#[tokio::main]
async fn main() {
    let mut registry = ProviderRegistry::new();
    registry.register("dummy", Arc::new(DummyProvider));

    let router = SmartRouter::new(
        registry,
        vec!["dummy".to_string()],
        vec!["dummy".to_string()],
        None,
    );

    let (provider, trace) = router
        .route_with_trace(
            "example",
            None,
            None,
            &[RoutingMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            dracon_ai_contracts::SelectionConstraints::default(),
        )
        .await
        .expect("route");

    println!("routed to {}", trace.selected_model);
    println!("provider registered");
}
