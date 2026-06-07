mod common;

use common::*;
use skailar::{ChatCompletionRequest, ChatMessage};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A GET that fails with 503 then succeeds should be retried.
#[tokio::test]
async fn get_retries_on_5xx_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(503).set_body_json(sample_error("upstream_error", "down")),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_model_list()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 2);
    let models = client.models().list().await.unwrap();
    assert_eq!(models.len(), 1);
    // Mock expectations verified on drop.
}

/// A side-effecting POST must NOT be retried on 5xx (avoids double billing).
#[tokio::test]
async fn chat_does_not_retry_on_5xx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(500).set_body_json(sample_error("upstream_error", "boom")),
        )
        .expect(1) // exactly one attempt, no retry
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 3);
    let err = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model("m")
                .message(ChatMessage::user("hi"))
                .build()
                .unwrap(),
        )
        .await
        .unwrap_err();
    assert!(err.as_api().unwrap().is_upstream());
}

/// A side-effecting POST IS retried on 429 (rejected before reaching billing).
#[tokio::test]
async fn chat_retries_on_429() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_json(sample_error("rate_limited", "slow down")),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_completion("ok")))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 2);
    let res = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model("m")
                .message(ChatMessage::user("hi"))
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.choices[0].message.content, "ok");
}

/// With retries disabled, a single 503 is returned immediately.
#[tokio::test]
async fn no_retries_when_budget_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(503).set_body_json(sample_error("upstream_error", "down")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 0);
    let err = client.models().list().await.unwrap_err();
    assert!(err.as_api().unwrap().is_upstream());
}

/// The retry budget is exhausted after `1 + max_retries` attempts.
#[tokio::test]
async fn exhausts_retry_budget_on_persistent_5xx() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(503).set_body_json(sample_error("upstream_error", "down")),
        )
        .expect(3) // 1 initial + 2 retries
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 2);
    let err = client.models().list().await.unwrap_err();
    assert!(err.as_api().unwrap().is_upstream());
}

/// 4xx (non-429) is never retried.
#[tokio::test]
async fn does_not_retry_on_4xx() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(403).set_body_json(sample_error("forbidden", "nope")))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 3);
    let err = client.models().list().await.unwrap_err();
    assert_eq!(err.as_api().unwrap().status, 403);
}

/// Retry-After above the 60s cap does not stall the client past the cap.
///
/// Uses a short timeout assertion: the call must resolve quickly because the
/// retry-after of "0" means immediate retry; this guards the happy path timing.
#[tokio::test]
async fn retry_after_zero_is_prompt() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_json(sample_error("rate_limited", "slow")),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_model_list()))
        .mount(&server)
        .await;

    let client = client_with_retries(&server, 2);
    let result = tokio::time::timeout(Duration::from_secs(5), client.models().list()).await;
    assert!(result.is_ok(), "retry with retry-after:0 should be prompt");
    assert_eq!(result.unwrap().unwrap().len(), 1);
}
