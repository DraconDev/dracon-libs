use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use dracon_ai_runtime_contracts::models::{ChatMessage, ChatRequest};
pub use dracon_ai_contracts::{RoutingTask, SelectionConstraints};

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
}

impl AiService {
    pub fn new(registry: ProviderRegistry, policy: LaneModelPolicy) -> Self {
        Self { registry, policy }
    }

    pub async fn ask(&self, request: ChatRequest) -> anyhow::Result<String> {
        let provider: Arc<crate::GenericOpenAIAdapter> = self.registry.get("default")
            .ok_or_else(|| anyhow::anyhow!("No provider found"))?;
        let (content, _) = provider.ask_and_collect(request).await?;
        Ok(content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    pub task: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub models: Vec<LeaderboardEntry>,
    pub max_quality_score: f32,
    pub max_coding_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub model_id: String,
    pub quality_score: f32,
    pub coding_score: f32,
    pub latency_ms: u64,
}
