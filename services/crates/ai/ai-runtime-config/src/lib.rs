#![warn(missing_docs)]

//! AI Runtime Config — configuration types for AI runtime providers.
//!
//! Defines how provider specs (models, endpoints, auth) are structured
//! and how runtime configuration is resolved.
//!
//! ## Types
//!
//! - [`OpenAIProviderSpec`] — OpenAI-compatible provider definition
//! - [`AiRuntimeConfig`] — top-level runtime config combining provider specs
//! - `resolve_ai_runtime_config()` — resolves config from environment or file

use serde::{Deserialize, Serialize};

/// OpenAI-compatible provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProviderSpec {
    /// Model identifier exposed to callers.
    pub model_id: String,
    /// Base endpoint URL for the provider.
    pub endpoint: String,
    /// Model identifier to place in request payloads when different from `model_id`.
    pub payload_model: String,
    /// API keys accepted for this provider.
    pub api_keys: Vec<String>,
    /// HTTP header name carrying the auth token.
    pub auth_header_name: String,
    /// Prefix added to the auth token value, such as `Bearer`.
    pub auth_header_prefix: String,
}

/// Top-level AI runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRuntimeConfig {
    /// Configured OpenAI-compatible providers.
    pub openai_providers: Vec<OpenAIProviderSpec>,
    /// Model IDs available for normal production traffic.
    pub active_model_ids: Vec<String>,
    /// Model IDs reserved for development or experimental traffic.
    pub dev_model_ids: Vec<String>,
    /// Optional lane policy name used by routing code.
    pub lane_model_policy: Option<String>,
}

/// Resolve an empty AI runtime configuration.
///
/// This function currently returns deterministic empty defaults so callers can
/// build a valid config without requiring external files or environment state.
pub fn resolve_ai_runtime_config() -> AiRuntimeConfig {
    AiRuntimeConfig {
        openai_providers: Vec::new(),
        active_model_ids: Vec::new(),
        dev_model_ids: Vec::new(),
        lane_model_policy: None,
    }
}
