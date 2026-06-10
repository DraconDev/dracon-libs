//! OpenAI Chat Completions + Image Generation API.
//!
//! Speaks the standard OpenAI protocol, which is also used by OpenRouter,
//! NVIDIA NIM, Mistral, DeepSeek, and Apertis. Just point `base_url` at
//! the right host.

use crate::error::Error;
use crate::types::{ChatRequest, ChatResponse, ImageRequest, ImageResponse, Role};
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};

/// Internal: the request body shape OpenAI expects.
#[derive(Debug, Serialize)]
struct OpenAiChatBody<'a> {
    model: &'a str,
    messages: &'a [OpenAiMessage<'a>],
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResp {
    model: String,
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiRespMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiRespMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

/// Send a chat request to an OpenAI-compatible endpoint.
pub async fn chat(http: &HttpClient, request: ChatRequest) -> Result<ChatResponse, Error> {
    if request.api_key.is_empty() {
        return Err(Error::InvalidRequest("api_key is empty".into()));
    }
    if request.base_url.is_empty() {
        return Err(Error::InvalidRequest("base_url is empty".into()));
    }
    if request.model.is_empty() {
        return Err(Error::InvalidRequest("model is empty".into()));
    }
    if request.messages.is_empty() {
        return Err(Error::InvalidRequest("messages is empty".into()));
    }

    let messages: Vec<OpenAiMessage> = request
        .messages
        .iter()
        .map(|m| OpenAiMessage {
            role: role_str(m.role),
            content: &m.content,
        })
        .collect();

    let body = OpenAiChatBody {
        model: &request.model,
        messages: &messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
    };

    let url = format!("{}/chat/completions", request.base_url.trim_end_matches('/'));

    let resp = http
        .post(&url)
        .bearer_auth(&request.api_key)
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::Provider {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: OpenAiChatResp = resp
        .json()
        .await
        .map_err(|e| Error::Parse(e.to_string()))?;

    let content = parsed
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok(ChatResponse {
        content,
        model_used: parsed.model,
        finish_reason: parsed.choices.first().and_then(|c| c.finish_reason.clone()),
        prompt_tokens: parsed.usage.as_ref().and_then(|u| u.prompt_tokens),
        completion_tokens: parsed.usage.and_then(|u| u.completion_tokens),
    })
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

// ── Images ──

#[derive(Debug, Serialize)]
struct OpenAiImageBody<'a> {
    model: &'a str,
    prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<&'a str>,
    response_format: &'a str, // "url" or "b64_json"
}

#[derive(Debug, Deserialize)]
struct OpenAiImageResp {
    data: Vec<OpenAiImageEntry>,
}

#[derive(Debug, Deserialize)]
struct OpenAiImageEntry {
    url: Option<String>,
    b64_json: Option<String>,
    #[serde(default)]
    revised_prompt: Option<String>,
}

/// Send an image generation request.
pub async fn image(http: &HttpClient, request: ImageRequest) -> Result<ImageResponse, Error> {
    if request.api_key.is_empty() {
        return Err(Error::InvalidRequest("api_key is empty".into()));
    }
    if request.prompt.is_empty() {
        return Err(Error::InvalidRequest("prompt is empty".into()));
    }

    let body = OpenAiImageBody {
        model: &request.model,
        prompt: &request.prompt,
        n: request.n,
        size: request.size.as_deref(),
        response_format: "url",
    };

    let url = format!("{}/images/generations", request.base_url.trim_end_matches('/'));

    let resp = http
        .post(&url)
        .bearer_auth(&request.api_key)
        .json(&body)
        .send()
        .await?;

    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(Error::Provider {
            status: 429,
            body: "rate limited".into(),
        });
    }

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::Provider {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: OpenAiImageResp = resp
        .json()
        .await
        .map_err(|e| Error::Parse(e.to_string()))?;

    Ok(ImageResponse {
        images: parsed
            .data
            .into_iter()
            .map(|e| crate::types::GeneratedImage {
                url: e.url,
                b64_json: e.b64_json,
                revised_prompt: e.revised_prompt,
            })
            .collect(),
    })
}
