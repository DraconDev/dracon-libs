use serde::{Deserialize, Serialize};

/// A single message in a chat conversation.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role sending this message (e.g. "user", "assistant").
    pub role: String,
    /// The text content of the message.
    pub content: String,
}

impl ChatMessage {
    /// Create a chat message.
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// A chat completion request.
#[non_exhaustive]
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

impl ChatRequest {
    /// Create a chat completion request.
    pub fn new(project_id: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            project_id: project_id.into(),
            messages,
            client_intent: None,
            max_tokens: None,
            temperature: None,
            stream: false,
        }
    }

    /// Set the optional routing task for this request.
    pub fn with_client_intent(
        mut self,
        client_intent: Option<dracon_ai_contracts::RoutingTask>,
    ) -> Self {
        self.client_intent = client_intent;
        self
    }

    /// Set the maximum token budget for this request.
    pub fn with_max_tokens(mut self, max_tokens: Option<usize>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the sampling temperature for this request.
    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set whether the request should be streamed.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
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
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The generated text content.
    pub content: String,
    /// Optional finish reason from the model.
    pub finish_reason: Option<String>,
}
