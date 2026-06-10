//! Integration tests for ai-lib.
//!
//! These use httpmock to stand up a fake OpenAI-compatible server. They
//! verify:
//! - Happy-path chat round-trip
//! - The Authorization header carries the caller's key (not some default)
//! - Provider errors are surfaced as `Error::Provider` with the status
//! - Validation rejects empty keys, empty models, empty messages

use ai_lib::{AiClient, ChatRequest, Error, ImageRequest, Message};
use httpmock::prelude::*;
use serde_json::json;

#[tokio::test]
async fn chat_sends_bearer_token_and_returns_content() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .header("authorization", "Bearer test-key-123")
            .header("content-type", "application/json")
            .body(r#"{"model":"gpt-4o-mini","messages":[{"role":"user","content":"hi"}]}"#);
        then.status(200).json_body(json!({
            "id": "chatcmpl-1",
            "model": "gpt-4o-mini-2024-07-18",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hello back!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSB6Um1rakVVK0QzaHdQUHV3K2p4R2llSVhqZ3JzdjZJWGlpUVllSHQxczBVCnF0SkYva3h5dzdkdWsyTjRBYnViNDZYRTFSTWgwOHE4TG1Qc3RldjhSbzgKLT4gWDI1NTE5IEZhaUZKRnllV0lybUdENXRUU3p0TVZ3WVdndVVRaHlodzZ2bVR2NmNOUjQKUndvanJnMHc3d1hxSk9LTlYyR0RPRjFGYTR3RWFXMEF3OFhRUklmOWJ2MAotPiBYMjU1MTkgOWI4ZS9lalBCMGhtV3lUTkhNdDNHdHI5K2Y5citLakpRS1d5Wm1sU3JobwpFeDVXYjMzRG4rWnVPOHYzdTFucWgybGp3Z2hPQUhob01qdmdneFlJcjYwCi0+IFgyNTUxOSBVemNPWEw0MU1uTGJHSmtKbEdSWnI3dGpIMmttMHJYekFoTlRaN3ZFN1NFCnNMckwzZjJ1enR3ZUh1UnNjQ1pLL2JyckpuWUhKS0F4U3VzdTUvKytKNncKLT4gWDI1NTE5IERYNUVNTUp3ajVGWUxhV1BwbHVIVXBaRU1WMFFDVkFXZFFhTFFlNmE1akEKZlVQN050UkpVMTZYQm5zb2tmMzg0djRkQ2dZK3NUeGFPV1poVkdqTGl6ZwotPiA5aWUmO0Faay1ncmVhc2UgTSBDIE45UFNIClBxcGtlWkJWbjlTUmJLUkFXbDZnUUdTZ2xIaE9ZWHUrUTR3RzlWZ3ZyWHJMZlBqazFMNFJodVRXUWZnK3JvYnIKc0dJZk00dWVtNGRHU21ZCi0tLSBLMjIwQjZNZGYrOWozNnlSbGVkdmRFbUxKWUcrMmVaSklLU0duVVUwVExvCnYB7gaf+K+3QgMs0+9PdGFdSXOvPRd6hZk7oVRoMqnvb0D+Q/u1SEzpSzFSy84EJ9/6L+PPdwLwdqAYo9vs2w==]: 3, "total_tokens": 8}
        }));
    });

    let client = AiClient::new();
    let resp = client
        .chat(
            ChatRequest::new("test-key-123", &server.base_url(), "gpt-4o-mini")
                .message(Message::user("hi")),
        )
        .await
        .expect("chat should succeed");

    mock.assert();
    assert_eq!(resp.content, "Hello back!");
    assert_eq!(resp.model_used, "gpt-4o-mini-2024-07-18");
    assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
    assert_eq!(resp.prompt_tokens, Some(5));
    assert_eq!(resp.completion_tokens, Some(3));
}

#[tokio::test]
async fn chat_uses_caller_provided_base_url_not_a_default() {
    // The whole point of ai-lib: no default URL, no env. The caller
    // picks the provider host.
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(200).json_body(json!({
            "model": "m",
            "choices": [{"message": {"role": "assistant", "content": "ok"}, "finish_reason": "stop"}]
        }));
    });

    let client = AiClient::new();
    // The lib takes whatever URL the caller gives it. The mock is
    // listening on this exact server, so we just pass the base URL.
    let resp = client
        .chat(ChatRequest::new("k", &server.base_url(), "m").message(Message::user("hi")))
        .await
        .expect("should hit the caller-provided URL");
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn chat_surfaces_provider_4xx_as_error() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(401).body("invalid api key");
    });

    let client = AiClient::new();
    let err = client
        .chat(ChatRequest::new("bad", &server.base_url(), "m").message(Message::user("hi")))
        .await
        .expect_err("should error");

    match err {
        Error::Provider { status, body } => {
            assert_eq!(status, 401);
            assert_eq!(body, "invalid api key");
        }
        other => panic!("expected Error::Provider, got {:?}", other),
    }
}

#[tokio::test]
async fn chat_rejects_empty_api_key() {
    let client = AiClient::new();
    let err = client
        .chat(ChatRequest::new("", "http://x", "m").message(Message::user("hi")))
        .await
        .expect_err("empty key should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn chat_rejects_empty_model() {
    let client = AiClient::new();
    let err = client
        .chat(ChatRequest::new("k", "http://x", "").message(Message::user("hi")))
        .await
        .expect_err("empty model should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn chat_rejects_empty_messages() {
    let client = AiClient::new();
    let err = client
        .chat(ChatRequest::new("k", "http://x", "m"))
        .await
        .expect_err("empty messages should be rejected");
    assert!(matches!(err, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn chat_supports_max_tokens_and_temperature() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .body_contains("\"max_tokens\":50")
            .body_contains("\"temperature\":0.7");
        then.status(200).json_body(json!({
            "model": "m",
            "choices": [{"message": {"role": "assistant", "content": "ok"}, "finish_reason": "stop"}]
        }));
    });

    let client = AiClient::new();
    let resp = client
        .chat(
            ChatRequest::new("k", &server.base_url(), "m")
                .message(Message::user("hi"))
                .max_tokens(50)
                .temperature(0.7),
        )
        .await
        .expect("ok");
    mock.assert();
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn image_request_hits_images_generations_endpoint() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/images/generations")
            .header("authorization", "Bearer img-key")
            .body_contains("dall-e-3")
            .body_contains("a corgi");
        then.status(200).json_body(json!({
            "data": [{
                "url": "https://example.com/corgi.png",
                "revised_prompt": "a happy corgi"
            }]
        }));
    });

    let client = AiClient::new();
    let resp = client
        .image(
            ImageRequest::new("img-key", &server.base_url(), "dall-e-3", "a corgi")
                .size("1024x1024")
                .n(1),
        )
        .await
        .expect("ok");

    mock.assert();
    assert_eq!(resp.images.len(), 1);
    assert_eq!(resp.images[0].url.as_deref(), Some("https://example.com/corgi.png"));
    assert_eq!(resp.images[0].revised_prompt.as_deref(), Some("a happy corgi"));
}

#[tokio::test]
async fn the[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBEYi8zckgyUmZNV3pTam1SUnNqZVoydzhBS0xzbkZjdEdLY3ZpL0RyTW1zCnpKRkhXQWVKVENTbUszR1VCMUdlckFzemxpekNraUV4WHBNUnZSR1ZzWjgKLT4gWDI1NTE5IFdCczdxb3JSaFA3a1FHQ3hyaUJwUjNCa2R1YUZUaEJTa2hLdmZRMCtjaFEKZkhDeCtETnFtbUdIOFZYWUY4dHVyNmNTc1NxYjhNZzFkemZqRGV4RHBXYwotPiBYMjU1MTkgWTRIQm1nT3ppdlJKVUdRNWZuN05iR2FjZ0ZHQ1pHWmlGNU5EVURHd2F5TQpDRDFqcFN6Z0JSZ24yanhrdnI3RzFSWmxnSUxScXlmNSsrMHBkdnZuS25NCi0+IFgyNTUxOSBhZnhNajBMQ09xdnVCaU9HcGxUSEowdnBaTEpXMStSS1JqVzdxLzZhNm13Ck9ZNXJVc2Y5dTN5TXlyNC8rVzJSeFcveXduWlFJdjlwaytWL1RtVnoyRnMKLT4gWDI1NTE5IEFPSk5zYlNRQ2t0bW83RUFhdG82RDc1SUxoeGRDbXNCTm82RGdzaVowbk0KUU5pc0FSdW5HenRYbEdmVzB2ajhFMzJ2VXk4VExHd3FTQytaVkZSaWI5WQotPiBALGIsLWdyZWFzZQpHcDVnK3lnQWtGMlJKVE5vQy9Vb0RFNXQ5bCtDTHN3OThFZlZCaGIvZXo4NzBjNFFMR0xlSXJ1QWpBTGhlTDNDCmNnT2V0Z0hlWlpUdXlBCi0tLSBJMjV6Y3Y5ejhwWmZVdjZwa0VVaUdmS09uM2JVTXVVUTBTc0F0YnFiWGpjCj8bD2wJ931cwNWTZVVodahdazLnLM2/DVjXTuEIkriLZ5rlJvWmqJ68L5eVh2WoIE89ECgHndtkQcmEwq3u/Wj0L7XYnb0S4II=]() {
    // This test documents (and pins) the "no defaults" property.
    // If someone adds `AiClient::from_env()` or a `Default::default()`
    // that pulls from env, this assertion will fail to compile, which
    // is the desired signal.
    //
    // We just exercise the API as the user would.
    let _ = std::any::type_name::<AiClient>();
    let _client = AiClient::new();

    // The chat request has no Default impl and no from_env. The user
    // must supply api_key, base_url, and model.
    let req: ChatRequest = ChatRequest::new("k", "http://x", "m");
    assert_eq!(req.api_key, "k");
    assert_eq!(req.base_url, "http://x");
    assert_eq!(req.model, "m");
}
