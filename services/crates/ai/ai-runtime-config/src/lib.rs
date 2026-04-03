use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProviderSpec {
    pub model_id: String,
    pub endpoint: String,
    pub payload_model: String,
    pub api_keys: Vec<String>,
    pub auth_header_name: String,
    pub auth_header_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRuntimeConfig {
    pub openai_providers: Vec<OpenAIProviderSpec>,
    pub active_model_ids: Vec<String>,
    pub dev_model_ids: Vec<String>,
    pub lane_model_policy: Option<String>,
}

pub fn resolve_ai_runtime_config() -> AiRuntimeConfig {
    AiRuntimeConfig {
        openai_providers: Vec::new(),
        active_model_ids: Vec::new(),
        dev_model_ids: Vec::new(),
        lane_model_policy: None,
    }
}
