#![warn(missing_docs)]

//! AI Service — provider registry and routing for AI backends.
//!
//! ## Key Types
//!
//! - [`ProviderRegistry`] — `AiProvider`-typed provider registry for runtime registration
//! - [`AiService`] — high-level service combining registry + policy
//! - [`LaneModelPolicy`] — routing lane policy configuration
//! - [`DEFAULT_PROVIDER`] — constant for the default provider name ("default")
//!
//! ## Example
//!
//! ```ignore
//! use ai_service::{AiService, ProviderRegistry, DEFAULT_PROVIDER};
//! let registry = ProviderRegistry::new();
//! registry.register(DEFAULT_PROVIDER, Arc::new(adapter));
//! let svc = AiService::new(registry, LaneModelPolicy::default());
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use dracon_ai_runtime_contracts::traits::AiProvider;
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
    providers: Vec<(String, Arc<dyn AiProvider>)>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register(&mut self, model_id: &str, provider: Arc<dyn AiProvider>) {
        self.providers.push((model_id.to_string(), provider));
    }

    pub fn get(&self, model_id: &str) -> Option<Arc<dyn AiProvider>> {
        self.providers
            .iter()
            .find(|(id, _)| id == model_id)
            .map(|(_, p)| p.clone())
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
        let provider = self.registry.get(&self.default_provider)
            .ok_or_else(|| anyhow::anyhow!("No provider found for '{}'", self.default_provider))?;
        let (content, _) = provider.ask_and_collect(request).await?;
        Ok(content)
    }
}
