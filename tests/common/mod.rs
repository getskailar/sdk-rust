//! Shared helpers for integration tests.
//!
//! Each integration test file compiles this module as part of its own crate and
//! uses a different subset of the helpers, so unused-item warnings here are
//! expected and suppressed.
#![allow(dead_code)]

use serde_json::{Value, json};
use skailar::Skailar;
use wiremock::MockServer;

/// API key used across all mocked requests.
pub const TEST_KEY: &str = "skl_live_0123456789abcdefghijklmnopqrstuvwxyz01234";

/// Builds a client pointed at the mock server with retries disabled by default.
pub fn client(server: &MockServer) -> Skailar {
    Skailar::builder()
        .api_key(TEST_KEY)
        .base_url(server.uri())
        .max_retries(0)
        .build()
        .expect("client builds")
}

/// Builds a client with a specific retry budget.
pub fn client_with_retries(server: &MockServer, max_retries: u32) -> Skailar {
    Skailar::builder()
        .api_key(TEST_KEY)
        .base_url(server.uri())
        .max_retries(max_retries)
        .build()
        .expect("client builds")
}

/// A minimal non-streamed completion body with the given assistant content.
pub fn sample_completion(content: &str) -> Value {
    json!({
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "created": 1_700_000_000,
        "model": "claude-sonnet-4-6",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8 }
    })
}

/// A model-list body with a single entry.
pub fn sample_model_list() -> Value {
    json!({
        "object": "list",
        "data": [ sample_model_summary("claude-sonnet-4-6") ]
    })
}

/// A model-summary body.
pub fn sample_model_summary(id: &str) -> Value {
    json!({
        "id": id,
        "object": "model",
        "created": 1_700_000_000,
        "owned_by": "anthropic",
        "display_name": "Claude Sonnet 4.6",
        "context_window": 200_000,
        "max_output_tokens": 64_000,
        "capabilities": {
            "streaming": true,
            "tool_calls": true,
            "vision": true,
            "json_mode": true,
            "reasoning": true
        },
        "pricing": {
            "input_per_mtok": 3.0,
            "output_per_mtok": 15.0,
            "currency": "USD"
        },
        "status": "active"
    })
}

/// A structured error body in the gateway's nested shape.
pub fn sample_error(code: &str, message: &str) -> Value {
    json!({ "error": { "type": code, "message": message } })
}

/// Builds an SSE body from content pieces, terminated by `[DONE]`.
///
/// `terminator` lets tests exercise `\n`, `\r\n`, and `\r`.
pub fn sse_stream(pieces: &[&str], terminator: &str) -> String {
    let mut out = String::new();
    for (i, piece) in pieces.iter().enumerate() {
        let chunk = json!({
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "created": 1_700_000_000,
            "model": "claude-sonnet-4-6",
            "choices": [{
                "index": 0,
                "delta": { "content": piece },
                "finish_reason": if i + 1 == pieces.len() { Value::String("stop".into()) } else { Value::Null }
            }]
        });
        out.push_str("data: ");
        out.push_str(&chunk.to_string());
        out.push_str(terminator);
        out.push_str(terminator);
    }
    out.push_str("data: [DONE]");
    out.push_str(terminator);
    out
}
