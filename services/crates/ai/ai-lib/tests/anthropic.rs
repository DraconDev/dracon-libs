//! Integration tests for the Anthropic Messages API adapter.
//!
//! These use httpmock to stand up a fake Anthropic server. They verify:
//! - Happy-path chat round trip
//! - The `x-api-key` and `anthropic-version` headers are sent
//! - The system prompt is a top-level `system` field, NOT a message
//! - Provider errors are surfaced as `Error::Provider` with the status
//! - Validation rejects empty key, empty model, empty system, empty messages,
//!   zero max_tokens

use ai_lib::{
    AiClient, AnthropicMessage, AnthropicRequest, Error,
};
use httpmock::prelude::*;
use serde_json::json;

#[tokio::test]
async fn anthropic_sends_api_key_header_and_returns_content() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "sk-ant-test-123")
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");
        then.status(200).json_body(json!({
            "id": "msg_01",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-5-20250929",
            "content": [{"type": "text", "text": "Hello back!"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 12, "output_tokens": 7}
        }));
    });

    let client = AiClient::new();
    let resp = client
        .anthropic_chat(
            AnthropicRequest::new(
                "sk-ant-test-123",
                &server.base_url(),
                "claude-sonnet-4-5",
                "You are helpful.",
                256,
            )
            .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect("anthropic chat should succeed");

    mock.assert();
    assert_eq!(resp.content, "Hello back!");
    assert_eq!(resp.model_used, "claude-sonnet-4-5-20250929");
    assert_eq!(resp.finish_reason.as_deref(), Some("end_turn"));
    assert_eq!(resp.prompt_tokens, Some(12));
    assert_eq!(resp.completion_tokens, Some(7));
}

#[tokio::test]
async fn anthropic_sends_system_as_top_level_field_not_a_message() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            // The body must contain the system prompt as a top-level field,
            // and the messages array must contain only user/assistant roles
            // (no role=system anywhere).
            .body_contains("\"system\":\"You are a pirate.\"")
            .body_contains("\"messages\":[")
            .body_contains("\"role\":\"user\"")
            .body_contains("\"role\":\"assistant\"");
        then.status(200).json_body(json!({
            "model": "claude-sonnet-4-5",
            "content": [{"type": "text", "text": "Arrr!"}],
            "stop_reason": "end_turn"
        }));
    });

    let client = AiClient::new();
    let resp = client
        .anthropic_chat(
            AnthropicRequest::new("k", &server.base_url(), "claude-sonnet-4-5", "You are a pirate.", 256)
                .message(AnthropicMessage::assistant("Hello!"))
                .message(AnthropicMessage::user("Tell me about your ship.")),
        )
        .await
        .expect("ok");
    mock.assert();
    assert_eq!(resp.content, "Arrr!");
}

#[tokio::test]
async fn anthropic_surfaces_provider_4xx_as_error() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#);
    });

    let client = AiClient::new();
    let err = client
        .anthropic_chat(
            AnthropicRequest::new("bad", &server.base_url(), "claude-sonnet-4-5", "sys", 256)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect_err("should error");

    match err {
        Error::Provider { status, body } => {
            assert_eq!(status, 401);
            assert!(body.contains("authentication_error"));
        }
        other => panic!("expected Error::Provider, got {:?}", other),
    }
}

#[tokio::test]
async fn anthropic_rejects_empty_api_key() {
    let client = AiClient::new();
    let err = client
        .anthropic_chat(
            AnthropicRequest::new("", "http://x", "claude-sonnet-4-5", "sys", 256)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect_err("empty key should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn anthropic_rejects_empty_model() {
    let client = AiClient::new();
    let err = client
        .anthropic_chat(
            AnthropicRequest::new("k", "http://x", "", "sys", 256)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect_err("empty model should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn anthropic_rejects_empty_system() {
    let client = AiClient::new();
    let err = client
        .anthropic_chat(
            AnthropicRequest::new("k", "http://x", "claude-sonnet-4-5", "", 256)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect_err("empty system should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn anthropic_rejects_empty_messages() {
    let client = AiClient::new();
    let err = client
        .anthropic_chat(AnthropicRequest::new("k", "http://x", "claude-sonnet-4-5", "sys", 256))
        .await
        .expect_err("empty messages should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn anthropic_rejects_zero_max_tokens() {
    let client = AiClient::new();
    let err = client
        .anthropic_chat(
            AnthropicRequest::new("k", "http://x", "claude-sonnet-4-5", "sys", 0)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect_err("max_tokens=0 should be rejected (Anthropic requires it)");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn anthropic_sends_max_tokens_and_temperature_in_body() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .body_contains("\"max_tokens\":512")
            .body_contains("\"temperature\":0.3");
        then.status(200).json_body(json!({
            "model": "m",
            "content": [{"type": "text", "text": "ok"}],
            "stop_reason": "end_turn"
        }));
    });

    let client = AiClient::new();
    let resp = client
        .anthropic_chat(
            AnthropicRequest::new("k", &server.base_url(), "m", "sys", 512)
                .message(AnthropicMessage::user("hi"))
                .temperature(0.3),
        )
        .await
        .expect("ok");
    mock.assert();
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn anthropic_concatenates_multiple_text_content_blocks() {
    // Anthropic sometimes returns multiple text blocks (e.g. when streaming
    // is reconstructed). Our adapter should concatenate them.
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200).json_body(json!({
            "model": "m",
            "content": [
                {"type": "text", "text": "Hello "},
                {"type": "text", "text": "world!"}
            ],
            "stop_reason": "end_turn"
        }));
    });

    let client = AiClient::new();
    let resp = client
        .anthropic_chat(
            AnthropicRequest::new("k", &server.base_url(), "m", "sys", 256)
                .message(AnthropicMessage::user("hi")),
        )
        .await
        .expect("ok");
    assert_eq!(resp.content, "Hello world!");
}
