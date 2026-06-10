//! models.dev API schema.
//!
//! These types mirror `https://models.dev/api.json` so callers can parse
//! the catalog without depending on the live endpoint.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The full models.dev catalog.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Catalog {
    #[serde(flatten)]
    pub providers: BTreeMap<String, Provider>,
}

impl Catalog {
    /// Look up a provider by id.
    pub fn get_provider(&self, provider_id: &str) -> Option<&Provider> {
        self.providers.get(provider_id)
    }

    /// Look up a model by `provider_id/model_id`.
    pub fn get_model(&self, provider_id: &str, model_id: &str) -> Option<&Model> {
        self.providers
            .get(provider_id)
            .and_then(|provider| provider.models.get(model_id))
    }

    /// Iterate over all providers.
    pub fn providers(&self) -> impl Iterator<Item = (&str, &Provider)> {
        self.providers.iter().map(|(id, provider)| (id.as_str(), provider))
    }
}

/// Provider metadata from models.dev.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provider {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub models: BTreeMap<String, Model>,
}

/// Model metadata from models.dev.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub knowledge: Option<String>,
    #[serde(default)]
    pub attachment: Option<bool>,
    #[serde(default)]
    pub reasoning: Option<bool>,
    #[serde(default)]
    pub tool_call: Option<bool>,
    #[serde(default)]
    pub structured_output: Option<bool>,
    #[serde(default)]
    pub temperature: Option<bool>,
    #[serde(default)]
    pub open_weights: Option<bool>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub limit: Option<Limit>,
    #[serde(default)]
    pub modalities: Option<Modalities>,
    #[serde(default)]
    pub links: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub weights: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub benchmarks: Vec<Benchmark>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub provider: Option<ProviderDetails>,
    #[serde(default)]
    pub options: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub variants: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cost {
    #[serde(default)]
    pub input: Option<f64>,
    #[serde(default)]
    pub output: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
    #[serde(default)]
    pub context_over_200k: Option<ContextOver200k>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextOver200k {
    #[serde(default)]
    pub input: Option<f64>,
    #[serde(default)]
    pub output: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Limit {
    #[serde(default)]
    pub context: Option<u64>,
    #[serde(default)]
    pub input: Option<u64>,
    #[serde(default)]
    pub output: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Modalities {
    #[serde(default)]
    pub input: Option<Vec<String>>,
    #[serde(default)]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Benchmark {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub metric: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderDetails {
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub api: Option<String>,
}
