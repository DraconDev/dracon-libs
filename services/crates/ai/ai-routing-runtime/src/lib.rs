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

use std::collections::HashMap;
use std::sync::Arc;
use dracon_ai_runtime_contracts::traits::AiProvider;
use serde::{Deserialize, Serialize};

pub use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
pub use routing::ServiceLevel;
pub use traits::AiModelStore;

#[derive(Debug, Clone)]
pub struct RoutingMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTrace {
    pub selected_model: String,
    pub lane: Option<RoutingTask>,
}

pub struct ProviderRegistry<T: ?Sized> {
    providers: HashMap<String, Arc<T>>,
}

impl<T: AiProvider + ?Sized> ProviderRegistry<T> {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, model_id: &str, provider: Arc<T>) {
        self.providers.insert(model_id.to_string(), provider);
    }

    pub fn get(&self, model_id: &str) -> Option<&Arc<T>> {
        self.providers.get(model_id)
    }
}

pub struct SmartRouter<T: ?Sized> {
    registry: ProviderRegistry<T>,
    dev_models: Vec<String>,
    active_models: Vec<String>,
    policy: Option<String>,
}

impl<T: AiProvider + ?Sized> SmartRouter<T> {
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

    pub async fn route_with_trace(
        &self,
        _project: &str,
        lane: Option<RoutingTask>,
        _preferred: Option<String>,
        _messages: &[RoutingMessage],
        _constraints: dracon_ai_contracts::SelectionConstraints,
    ) -> anyhow::Result<(Arc<T>, RoutingTrace)> {
        let model_id = self.active_models.first()
            .or_else(|| self.dev_models.first())
            .ok_or_else(|| anyhow::anyhow!("No models available"))?
            .clone();

        let provider = self.registry.get(&model_id)
            .ok_or_else(|| anyhow::anyhow!("Provider not found for model: {}", model_id))?
            .clone();

        let trace = RoutingTrace {
            selected_model: model_id,
            lane,
        };

        Ok((provider, trace))
    }
}
