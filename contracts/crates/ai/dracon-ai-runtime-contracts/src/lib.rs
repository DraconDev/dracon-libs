use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub project_id: String,
    pub messages: Vec<ChatMessage>,
    pub client_intent: Option<dracon_ai_contracts::RoutingTask>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            project_id: "default".to_string(),
            messages: Vec::new(),
            client_intent: None,
            max_tokens: None,
            temperature: None,
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn ask_and_collect(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)>;
}
