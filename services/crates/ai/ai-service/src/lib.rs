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

pub use ai_routing_runtime::traits::{LeaderboardEntry, LeaderboardRequest, LeaderboardResponse};
pub use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
pub use dracon_ai_runtime_contracts::models::{ChatMessage, ChatRequest};
pub use dracon_ai_runtime_contracts::traits::AiProvider;

/// Default provider id used by [`AiService`].
pub const DEFAULT_PROVIDER: &str = "default";

/// Routing lane policy controlling provider selection.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaneModelPolicy {
    /// Whether lane-based routing is enabled.
    pub enabled: bool,
    /// Default lane name used when lane routing is enabled.
    pub default_lane: Option<String>,
}

impl LaneModelPolicy {
    /// Create a lane model policy.
    pub fn new(enabled: bool, default_lane: Option<String>) -> Self {
        Self {
            enabled,
            default_lane,
        }
    }

    /// Enable lane-based routing with an optional default lane.
    pub fn enabled_with_default_lane(default_lane: impl Into<String>) -> Self {
        Self::new(true, Some(default_lane.into()))
    }
}

/// Provider registry for AI backends.
#[non_exhaustive]
pub struct ProviderRegistry {
    providers: Vec<(String, Arc<dyn AiProvider>)>,
}

impl ProviderRegistry {
    /// Create an empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider for `model_id`.
    pub fn register(&mut self, model_id: &str, provider: Arc<dyn AiProvider>) {
        self.providers.push((model_id.to_string(), provider));
    }

    /// Return the provider registered for `model_id`.
    pub fn get(&self, model_id: &str) -> Option<Arc<dyn AiProvider>> {
        self.providers
            .iter()
            .find(|(id, _)| id == model_id)
            .map(|(_, p)| p.clone())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level AI service that sends requests through a registered provider.
#[non_exhaustive]
pub struct AiService {
    registry: ProviderRegistry,
    policy: LaneModelPolicy,
    default_provider: String,
}

impl AiService {
    /// Create an AI service with a registry and lane policy.
    pub fn new(registry: ProviderRegistry, policy: LaneModelPolicy) -> Self {
        Self {
            registry,
            policy,
            default_provider: DEFAULT_PROVIDER.to_string(),
        }
    }

    /// Override the provider id used by [`AiService::ask`].
    pub fn with_default_provider(mut self, provider: impl Into<String>) -> Self {
        self.default_provider = provider.into();
        self
    }

    /// Send a chat request through the configured provider and return generated text.
    pub async fn ask(&self, request: ChatRequest) -> anyhow::Result<String> {
        let provider = self
            .registry
            .get(&self.default_provider)
            .ok_or_else(|| anyhow::anyhow!("No provider found for '{}'", self.default_provider))?;
        let (content, _) = provider.ask_and_collect(request).await?;
        Ok(content)
    }

    /// Return the configured lane policy.
    pub fn lane_policy(&self) -> &LaneModelPolicy {
        &self.policy
    }
}
