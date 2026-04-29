use serde::{Deserialize, Serialize};

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role sending this message (e.g. "user", "assistant").
    pub role: String,
    /// The text content of the message.
    pub content: String,
}

/// A chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Project this request belongs to.
    pub project_id: String,
    /// Message history for context.
    pub messages: Vec<ChatMessage>,
    /// Optional routing hint for model selection.
    pub client_intent: Option<dracon_ai_contracts::RoutingTask>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<usize>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Whether to stream responses.
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

/// A chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The generated text content.
    pub content: String,
    /// Optional finish reason from the model.
    pub finish_reason: Option<String>,
}
