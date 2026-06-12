#![warn(missing_docs)]

//! AI Routing Runtime — SmartRouter and ProviderRegistry for model selection.
//!
//! ## Key Types
//!
//! - [`SmartRouter<T>`] — routes requests to the best available model
//! - [`ProviderRegistry<T>`] — generic, type-safe provider registry
//! - [`RoutingMessage`] — message envelope for routing
//! - [`RoutingTrace`] — routing decision trace for observability
//!
//! ## Example
//!
//! ```ignore
//! use ai_routing_runtime::{SmartRouter, ProviderRegistry};
//! let registry = ProviderRegistry::new();
//! registry.register("gpt-4", Arc::new(provider));
//! let router = SmartRouter::new(registry, dev_models, active_models, None);
//! let (provider, trace) = router.route_with_trace(...).await?;
//! ```

pub mod routing;
pub mod traits;

use dracon_ai_runtime_contracts::traits::AiProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
pub use routing::ServiceLevel;
pub use traits::AiModelStore;

/// Message envelope used by routing code.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RoutingMessage {
    /// Participant role label.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Observability trace for a routing decision.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTrace {
    /// Selected model identifier.
    pub selected_model: String,
    /// Optional routing task or lane that influenced the decision.
    pub lane: Option<RoutingTask>,
}

/// Generic registry mapping model ids to provider implementations.
#[non_exhaustive]
pub struct ProviderRegistry<T: ?Sized> {
    providers: HashMap<String, Arc<T>>,
}

impl<T: AiProvider + ?Sized> ProviderRegistry<T> {
    /// Create an empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register or replace a provider for `model_id`.
    pub fn register(&mut self, model_id: &str, provider: Arc<T>) {
        self.providers.insert(model_id.to_string(), provider);
    }

    /// Return the provider registered for `model_id`.
    pub fn get(&self, model_id: &str) -> Option<&Arc<T>> {
        self.providers.get(model_id)
    }
}

impl<T: AiProvider + ?Sized> Default for ProviderRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Router that selects an active or development model from a provider registry.
#[non_exhaustive]
pub struct SmartRouter<T: ?Sized> {
    registry: ProviderRegistry<T>,
    dev_models: Vec<String>,
    active_models: Vec<String>,
    policy: Option<String>,
}

impl<T: AiProvider + ?Sized> SmartRouter<T> {
    /// Create a router with active models, development models, and optional policy.
    pub fn new(
        registry: ProviderRegistry<T>,
        dev_models: Vec<String>,
        active_models: Vec<String>,
        policy: Option<String>,
    ) -> Self {
        Self {
            registry,
            dev_models,
            active_models,
            policy,
        }
    }

    /// Select the first available active or development model and return its provider.
    pub async fn route_with_trace(
        &self,
        _project: &str,
        lane: Option<RoutingTask>,
        _preferred: Option<String>,
        _messages: &[RoutingMessage],
        _constraints: dracon_ai_contracts::SelectionConstraints,
    ) -> anyhow::Result<(Arc<T>, RoutingTrace)> {
        let model_id = self
            .active_models
            .first()
            .or_else(|| self.dev_models.first())
            .ok_or_else(|| anyhow::anyhow!("No models available"))?
            .clone();

        let provider = self
            .registry
            .get(&model_id)
            .ok_or_else(|| anyhow::anyhow!("Provider not found for model: {}", model_id))?
            .clone();

        let trace = RoutingTrace {
            selected_model: model_id,
            lane,
        };

        Ok((provider, trace))
    }

    /// Return the optional routing policy name.
    pub fn policy(&self) -> Option<&str> {
        self.policy.as_deref()
    }
}
