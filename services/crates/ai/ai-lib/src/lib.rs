//! # ai-lib
//!
//! Super-simple Rust client for direct AI provider access.
//!
//! ## What this is
//!
//! A thin async client that lets you call OpenAI, Anthropic, Google, and
//! friends **directly** — no gateway, no auth layer, no key store.
//! You pass the API key, you pick the provider, you get a response.
//!
//! ## What this is NOT
//!
//! - Not a config loader. There is no `from_env()` and no defaults.
//! - Not a key store. The lib never persists your keys.
//! - Not a gateway. It calls providers directly; you pay their prices.
//! - Not pre-configured. You must supply every parameter yourself.
//!
//! ## Quick start
//!
//! ```no_run
//! use ai_lib::{AiClient, ChatRequest, Message};
//!
//! # async fn run() -> Result<(), ai_lib::Error> {
//! let client = AiClient::new();
//!
//! let response = client
//!     .chat(
//!         ChatRequest::new("sk-your-openai-key", "https://api.openai.com/v1", "gpt-4o-mini")
//!             .message(Message::user("Hello!"))
//!             .message(Message::assistant("Hi! How can I help?"))
//!             .message(Message::user("Tell me a joke.")),
//!     )
//!     .await?;
//!
//! println!("{}", response.content);
//! # Ok(()) }
//! ```
//!
//! ## Provider compatibility
//!
//! `chat()` speaks the OpenAI Chat Completions API. It works verbatim with
//! any OpenAI-compatible endpoint:
//!
//! - OpenAI (`https://api.openai.com/v1`)
//! - OpenRouter (`https://openrouter.ai/api/v1`)
//! - NVIDIA NIM (`https://integrate.api.nvidia.com/v1`)
//! - Mistral (`https://api.mistral.ai/v1`)
//! - DeepSeek (`https://api.deepseek.com/v1`)
//! - Apertis (`https://api.apertis.ai/v1`)
//!
//! `anthropic_chat()` speaks the Anthropic Messages API directly:
//!
//! - Anthropic (`https://api.anthropic.com`)

pub mod error;
pub mod providers;
pub mod types;

pub use ai_models_catalog::{
    builtin_env_vars, env_var_names_for, AiModelsConfig, BuiltinProvider, Catalog, Cost, Limit,
    Model, ModelOverride, Modalities, Provider, ProviderConfig,
};
pub use error::Error;
pub use types::{
    AnthropicMessage, AnthropicRequest, AnthropicRole, ChatRequest, ChatResponse, ImageRequest,
    ImageResponse, Message, Role,
};

use reqwest::Client as HttpClient;
use std::sync::Arc;

/// The client. Cheap to clone (the inner `reqwest::Client` is `Arc`-backed).
#[derive(Clone)]
pub struct AiClient {
    http: Arc<HttpClient>,
}

impl AiClient {
    /// Create a new client with default settings.
    ///
    /// There is no `from_env()` — supply your keys at the call site.
    pub fn new() -> Self {
        Self {
            http: Arc::new(
                HttpClient::builder()
                    .build()
                    .expect("reqwest client should build"),
            ),
        }
    }

    /// Create a client with a custom reqwest client (for testing, timeouts, proxies).
    pub fn with_http(http: HttpClient) -> Self {
        Self {
            http: Arc::new(http),
        }
    }

    /// Send a chat request to an OpenAI-compatible endpoint.
    ///
    /// `request` carries the API key, base URL, model, and messages.
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
        providers::openai::chat(&self.http, request).await
    }

    /// Send an image generation request to an OpenAI-compatible endpoint.
    pub async fn image(&self, request: ImageRequest) -> Result<ImageResponse, Error> {
        providers::openai::image(&self.http, request).await
    }

    /// Send a chat request to the Anthropic Messages API.
    ///
    /// Anthropic uses a different protocol than OpenAI (system prompt as a
    /// top-level field, `x-api-key` header, `max_tokens` required). Use this
    /// method instead of [`AiClient::chat`] when calling Anthropic directly.
    pub async fn anthropic_chat(
        &self,
        request: AnthropicRequest,
    ) -> Result<ChatResponse, Error> {
        providers::anthropic::chat(&self.http, request).await
    }
}

impl Default for AiClient {
    fn default() -> Self {
        Self::new()
    }
}
