mod common;

use common::*;
use futures_util::StreamExt;
use skailar::{ChatCompletionRequest, ChatMessage, Error, Skailar};
use std::time::Duration;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A default `Authorization` header must not override the SDK bearer token.
#[tokio::test]
async fn default_authorization_header_cannot_override_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ping-key"))
        .and(header(
            "authorization",
            format!("Bearer {TEST_KEY}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "user_id": "u1"
        })))
        .mount(&server)
        .await;

    let client = Skailar::builder()
        .api_key(TEST_KEY)
        .base_url(server.uri())
        .max_retries(0)
        // Attempt to hijack the credential; must be ignored.
        .default_header("Authorization", "Bearer attacker-token")
        .default_header("authorization", "Bearer attacker-token-lower")
        .build()
        .unwrap();

    let res = client.ping().await.unwrap();
    assert_eq!(res.status, "ok");
}

/// An internal timeout maps to `Error::Timeout`, distinct from `Error::Network`.
#[tokio::test]
async fn slow_response_times_out() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_json(sample_model_list()),
        )
        .mount(&server)
        .await;

    let client = Skailar::builder()
        .api_key(TEST_KEY)
        .base_url(server.uri())
        .max_retries(0)
        .timeout(Duration::from_millis(150))
        .build()
        .unwrap();

    match client.models().list().await {
        Err(Error::Timeout { timeout_secs }) => assert_eq!(timeout_secs, 0),
        other => panic!("expected timeout, got {other:?}"),
    }
}

/// Dropping a stream early cancels it without panicking or hanging.
#[tokio::test]
async fn dropping_stream_cancels_cleanly() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_stream(&["a", "b", "c", "d", "e"], "\n")),
        )
        .mount(&server)
        .await;

    let client = client(&server);
    let mut stream = client
        .chat()
        .completions()
        .create_stream(
            ChatCompletionRequest::builder()
                .model("m")
                .message(ChatMessage::user("hi"))
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let _first = stream.next().await.unwrap().unwrap();
    // Explicit close mirrors dropping; must return promptly.
    let closed = tokio::time::timeout(Duration::from_secs(2), async move {
        stream.close();
    })
    .await;
    assert!(closed.is_ok());
}

/// A connection failure surfaces as a transport error, never an API error.
#[tokio::test]
async fn connection_refused_is_network_error() {
    // Port 1 is reserved and not listening, so the connect attempt is refused.
    let client = Skailar::builder()
        .api_key(TEST_KEY)
        .base_url("http://127.0.0.1:1")
        .max_retries(0)
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    match client.models().list().await {
        Err(Error::Network(_)) => {}
        // Some platforms report a refused connection as a timeout-ish error;
        // accept either transport variant but never a success or API error.
        Err(Error::Timeout { .. }) => {}
        other => panic!("expected transport error, got {other:?}"),
    }
}
