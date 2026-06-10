//! Request and response types for ai-lib.

use serde::{Deserialize, Serialize};

/// A role in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }
}

/// A chat request. Holds the API key, endpoint, model, and messages.
///
/// Construct via [`ChatRequest::new`] and chain `.message(...)` to build up
/// the conversation.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl ChatRequest {
    /// Start a new chat request. All three parameters are required — there
    /// are no defaults.
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: model.into(),
            messages: Vec::new(),
            max_tokens: None,
            temperature: None,
        }
    }

    /// Append a message to the conversation.
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Cap the number of tokens in the response. Provider-specific defaults
    /// apply if unset.
    pub fn max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// Sampling temperature. Provider-specific defaults apply if unset.
    pub fn temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }
}

/// A chat response. Carries the assistant's reply and basic metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The text the assistant said.
    pub content: String,
    /// Which model actually served the request (may differ from the
    /// requested model if the provider substituted).
    pub model_used: String,
    /// Why the response ended: "stop", "length", "content_filter", "tool_calls".
    pub finish_reason: Option<String>,
    /// Input tokens billed, if the provider reported it.
    pub prompt_tokens: Option<u32>,
    /// Output tokens billed, if the provider reported it.
    pub completion_tokens: Option<u32>,
}

/// A request to generate one or more images.
#[derive(Debug, Clone)]
pub struct ImageRequest {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub prompt: String,
    pub n: Option<u32>,
    pub size: Option<String>,
}

impl ImageRequest {
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: model.into(),
            prompt: prompt.into(),
            n: None,
            size: None,
        }
    }

    /// Number of images to generate. Provider default is 1.
    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Image size, e.g. "1024x1024". Provider default applies.
    pub fn size(mut self, s: impl Into<String>) -> Self {
        self.size = Some(s.into());
        self
    }
}

/// A single generated image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// Either a URL to download the image, or a base64-encoded body.
    pub url: Option<String>,
    pub b64_json: Option<String>,
    /// The revised prompt the model actually used (DALL·E 3 returns this).
    pub revised_prompt: Option<String>,
}

/// Response to an image generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    pub images: Vec<GeneratedImage>,
}

// ── Anthropic Messages API ──
//
// Anthropic uses a different shape than OpenAI:
//   - the system prompt is a top-level `system` field, not a message
//   - the last `user`/`assistant` turn must end with `user`
//   - `max_tokens` is required
//   - auth is `x-api-key: <key>` + `anthropic-version: <date>`, not Bearer

/// A message in an Anthropic conversation. Only `user` and `assistant`
/// roles are valid here — `system` lives on the request, not in the
/// message list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,
    pub content: String,
}

impl AnthropicMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: AnthropicRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: AnthropicRole::Assistant,
            content: content.into(),
        }
    }
}

/// Role for an Anthropic message. Anthropic has only `user` and `assistant`
/// inside the messages array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicRole {
    User,
    Assistant,
}

/// A request to the Anthropic Messages API.
///
/// Unlike [`ChatRequest`], the system prompt is a top-level field (Anthropic's
/// protocol), and `max_tokens` is required (Anthropic refuses to default it).
#[derive(Debug, Clone)]
pub struct AnthropicRequest {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub system: String,
    pub messages: Vec<AnthropicMessage>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
}

impl AnthropicRequest {
    /// Start a new Anthropic request. `system`, `model`, and `max_tokens` are
    /// all required by the protocol — no defaults.
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        system: impl Into<String>,
        max_tokens: u32,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: model.into(),
            system: system.into(),
            messages: Vec::new(),
            max_tokens,
            temperature: None,
        }
    }

    /// Append a message to the conversation.
    pub fn message(mut self, message: AnthropicMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Sampling temperature. Provider default applies if unset.
    pub fn temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }
}
