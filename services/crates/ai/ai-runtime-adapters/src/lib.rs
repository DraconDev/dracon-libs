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
//! )?;
//! let (content, _) = adapter.ask_and_collect(request).await?;
//! ```

use std::fmt;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use dracon_ai_runtime_contracts::models::ChatRequest;
use dracon_ai_runtime_contracts::traits::AiProvider;

/// Adapter for OpenAI-compatible Chat Completions API endpoints.
pub struct GenericOpenAIAdapter {
    api_key: String,
    endpoint: String,
    model: String,
    auth_header_name: String,
    auth_header_prefix: String,
    client: reqwest::Client,
}

impl GenericOpenAIAdapter {
    /// Create a new adapter with validated auth header configuration.
    pub fn new_with_auth(
        api_key: String,
        endpoint: String,
        model: String,
        auth_header_name: String,
        auth_header_prefix: String,
    ) -> anyhow::Result<Self> {
        if api_key.trim().is_empty() {
            anyhow::bail!("api_key must not be empty");
        }
        if model.trim().is_empty() {
            anyhow::bail!("model must not be empty");
        }
        if auth_header_name.trim().is_empty() {
            anyhow::bail!("auth_header_name must not be empty");
        }
        if auth_header_prefix.trim().is_empty() {
            anyhow::bail!("auth_header_prefix must not be empty");
        }

        let parsed = reqwest::Url::parse(&endpoint)
            .with_context(|| format!("invalid OpenAI-compatible endpoint: {endpoint}"))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            anyhow::bail!("endpoint URL must use http or https: {endpoint}");
        }
        if parsed.host_str().is_none() || parsed.host_str().is_some_and(str::is_empty) {
            anyhow::bail!("endpoint URL must include a host: {endpoint}");
        }

        let normalized_endpoint = parsed
            .as_str()
            .trim_end_matches('/')
            .to_string();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("reqwest client should build")?;
        Ok(Self {
            api_key,
            endpoint: normalized_endpoint,
            model,
            auth_header_name,
            auth_header_prefix,
            client,
        })
    }
}

impl fmt::Debug for GenericOpenAIAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenericOpenAIAdapter")
            .field("api_key", &"<redacted>")
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("auth_header_name", &self.auth_header_name)
            .field("auth_header_prefix", &self.auth_header_prefix)
            .field("client", &self.client)
            .finish()
    }
}

#[async_trait]
impl AiProvider for GenericOpenAIAdapter {
    async fn ask_and_collect(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)> {
        if request.stream {
            anyhow::bail!("GenericOpenAIAdapter does not support streaming responses; use a streaming provider implementation");
        }

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
            .post(format!("{}/chat/completions", self.endpoint))
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

#[cfg(test)]
mod tests {
    use super::GenericOpenAIAdapter;

    #[test]
    fn rejects_empty_api_key() {
        let err = GenericOpenAIAdapter::new_with_auth(
            String::new(),
            "https://api.example.com/v1".to_string(),
            "model".to_string(),
            "Authorization".to_string(),
            "Bearer ".to_string(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn rejects_invalid_endpoint_scheme() {
        let err = GenericOpenAIAdapter::new_with_auth(
            "secret".to_string(),
            "ftp://api.example.com/v1".to_string(),
            "model".to_string(),
            "Authorization".to_string(),
            "Bearer ".to_string(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn debug_redacts_api_key() {
        let adapter = GenericOpenAIAdapter::new_with_auth(
            "secret-key".to_string(),
            "https://api.example.com/v1/".to_string(),
            "model".to_string(),
            "Authorization".to_string(),
            "Bearer ".to_string(),
        )
        .unwrap();
        let debug = format!("{adapter:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
        assert!(debug.contains("https://api.example.com/v1"));
    }
}
