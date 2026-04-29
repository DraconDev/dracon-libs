use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use dracon_ai_runtime_contracts::models::{ChatMessage, ChatRequest};
pub use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
pub use ai_routing_runtime::traits::{LeaderboardRequest, LeaderboardResponse, LeaderboardEntry};

pub const DEFAULT_PROVIDER: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneModelPolicy {
    pub enabled: bool,
    pub default_lane: Option<String>,
}

impl Default for LaneModelPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            default_lane: None,
        }
    }
}

pub struct ProviderRegistry {
    providers: Vec<(String, Arc<dyn std::any::Any + Send + Sync>)>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, model_id: &str, provider: Arc<T>) {
        self.providers.push((model_id.to_string(), provider));
    }

    pub fn get<T: 'static>(&self, model_id: &str) -> Option<Arc<T>> {
        self.providers
            .iter()
            .find(|(id, _)| id == model_id)
            .and_then(|(_, p)| p.clone().downcast::<T>().ok())
    }
}

pub struct AiService {
    registry: ProviderRegistry,
    policy: LaneModelPolicy,
    default_provider: String,
}

impl AiService {
    pub fn new(registry: ProviderRegistry, policy: LaneModelPolicy) -> Self {
        Self {
            registry,
            policy,
            default_provider: DEFAULT_PROVIDER.to_string(),
        }
    }

    pub fn with_default_provider(mut self, provider: impl Into<String>) -> Self {
        self.default_provider = provider.into();
        self
    }

    pub async fn ask(&self, request: ChatRequest) -> anyhow::Result<String> {
        let provider: Arc<crate::GenericOpenAIAdapter> = self.registry.get(&self.default_provider)
            .ok_or_else(|| anyhow::anyhow!("No provider found for '{}'", self.default_provider))?;
        let (content, _) = provider.ask_and_collect(request).await?;
        Ok(content)
    }
}
