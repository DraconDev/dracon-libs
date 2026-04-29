#![warn(missing_docs)]

//! AI Runtime Adapters — adapter implementations for AI backend protocols.
//!
//! Currently provides:
//! - [`GenericOpenAIAdapter`] — OpenAI Chat Completions API adapter with
//!   timeout, retries, and configurable auth headers
//!
//! ## Example
//!
//! ```ignore
//! use ai_runtime_adapters::GenericOpenAIAdapter;
//! let adapter = GenericOpenAIAdapter::new_with_auth(
//!     api_key, endpoint, model,
//!     "Authorization", "Bearer",
//! );
//! let (content, _) = adapter.ask_and_collect(request).await?;
//! ```

use std::time::Duration;

use async_trait::async_trait;
use dracon_ai_runtime_contracts::models::ChatRequest;
use dracon_ai_runtime_contracts::traits::AiProvider;

pub struct GenericOpenAIAdapter {
    api_key: String,
    endpoint: String,
    model: String,
    auth_header_name: String,
    auth_header_prefix: String,
    client: reqwest::Client,
}

impl GenericOpenAIAdapter {
    pub fn new_with_auth(
        api_key: String,
        endpoint: String,
        model: String,
        auth_header_name: String,
        auth_header_prefix: String,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client should build");
        Self {
            api_key,
            endpoint,
            model,
            auth_header_name,
            auth_header_prefix,
            client,
        }
    }
}

#[async_trait]
impl AiProvider for GenericOpenAIAdapter {
    async fn ask_and_collect(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)> {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(200),
            "temperature": request.temperature.unwrap_or(0.7),
        });

        let resp = self
            .client
            .post(&format!("{}/chat/completions", self.endpoint))
            .header(
                &self.auth_header_name,
                format!("{}{}", self.auth_header_prefix, self.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let finish_reason = json["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        Ok((content, finish_reason))
    }
}
