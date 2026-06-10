//! Anthropic Messages API.
//!
//! Speaks the Anthropic protocol directly (not via the OpenAI-compatible
//! adapter). Differences from OpenAI:
//!
//! - The system prompt is a **top-level `system` field**, not a message in
//!   the messages array.
//! - Auth is `x-api-key: <key>` plus `anthropic-version: <date>`, not
//!   `Authorization: Bearer <key>`.
//! - `max_tokens` is **required** — the API will 400 without it.
//! - Only `user` and `assistant` roles are valid inside `messages`.

use crate::error::Error;
use crate::types::{AnthropicRequest, AnthropicRole, ChatResponse};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

/// `anthropic-version` header value. Pin to a recent stable date so
/// consumers don't get surprise behavior when Anthropic ships a new
/// default.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Internal: Anthropic request body shape.
#[derive(Debug, Serialize)]
struct AnthropicBody<'a> {
    model: &'a str,
    system: &'a str,
    messages: &'a [AnthropicOutMessage<'a>],
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct AnthropicOutMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct AnthropicResp {
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    /// The content block kind (e.g. "text", "tool_use", "image"). We only
    /// consume text blocks; the field is kept for forward-compat and to
    /// make invalid responses easier to debug.
    #[serde(rename = "type")]
    #[allow(dead_code)]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

/// Send a chat request to the Anthropic Messages API.
pub async fn chat(http: &HttpClient, request: AnthropicRequest) -> Result<ChatResponse, Error> {
    if request.api_key.is_empty() {
        return Err(Error::InvalidRequest("api_key is empty".into()));
    }
    if request.base_url.is_empty() {
        return Err(Error::InvalidRequest("base_url is empty".into()));
    }
    if request.model.is_empty() {
        return Err(Error::InvalidRequest("model is empty".into()));
    }
    if request.system.is_empty() {
        return Err(Error::InvalidRequest("system is empty".into()));
    }
    if request.messages.is_empty() {
        return Err(Error::InvalidRequest("messages is empty".into()));
    }
    if request.max_tokens == 0 {
        return Err(Error::InvalidRequest("max_tokens must be > 0".into()));
    }

    let out_messages: Vec<AnthropicOutMessage> = request
        .messages
        .iter()
        .map(|m| AnthropicOutMessage {
            role: anthropic_role_str(m.role),
            content: &m.content,
        })
        .collect();

    let body = AnthropicBody {
        model: &request.model,
        system: &request.system,
        messages: &out_messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
    };

    let url = format!("{}/v1/messages", request.base_url.trim_end_matches('/'));

    let resp = http
        .post(&url)
        .header("x-api-key", &request.api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::Provider {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: AnthropicResp = resp
        .json()
        .await
        .map_err(|e| Error::Parse(e.to_string()))?;

    // Concatenate any text blocks. Anthropic returns content as an array;
    // we only care about the text blocks for chat.
    let content = parsed
        .content
        .into_iter()
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    Ok(ChatResponse {
        content,
        model_used: parsed.model,
        finish_reason: parsed.stop_reason,
        prompt_tokens: parsed.usage.as_ref().and_then(|u| u.input_tokens),
        completion_tokens: parsed.usage.and_then(|u| u.output_tokens),
    })
}

fn anthropic_role_str(role: AnthropicRole) -> &'static str {
    match role {
        AnthropicRole::User => "user",
        AnthropicRole::Assistant => "assistant",
    }
}
