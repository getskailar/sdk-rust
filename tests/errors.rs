mod common;

use common::*;
use skailar::{ChatCompletionRequest, ChatMessage, Error};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn trigger_get_error(status: u16, body: serde_json::Value) -> Error {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&server)
        .await;
    client(&server).models().list().await.unwrap_err()
}

#[tokio::test]
async fn maps_401_to_auth_error() {
    let err = trigger_get_error(401, sample_error("invalid_api_key", "bad key")).await;
    let api = err.as_api().expect("api error");
    assert!(api.is_auth());
    assert_eq!(api.code.as_deref(), Some("invalid_api_key"));
    assert_eq!(api.message, "bad key");
}

#[tokio::test]
async fn maps_404_to_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models/missing"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(sample_error("not_found", "no such model")),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .models()
        .retrieve("missing")
        .await
        .unwrap_err();
    assert!(err.as_api().unwrap().is_not_found());
}

#[tokio::test]
async fn maps_400_to_bad_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(sample_error("bad_request", "missing model")),
        )
        .mount(&server)
        .await;

    let err = client(&server)
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
    assert!(err.as_api().unwrap().is_bad_request());
}

#[tokio::test]
async fn maps_500_to_upstream() {
    let err = trigger_get_error(503, sample_error("upstream_error", "provider down")).await;
    assert!(err.as_api().unwrap().is_upstream());
}

#[tokio::test]
async fn rate_limit_exposes_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "7")
                .set_body_json(sample_error("rate_limited", "slow down")),
        )
        .mount(&server)
        .await;

    let err = client(&server).models().list().await.unwrap_err();
    let api = err.as_api().unwrap();
    assert!(api.is_rate_limit());
    assert_eq!(api.retry_after, Some(7));
}

#[tokio::test]
async fn captures_request_id_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(500)
                .insert_header("x-request-id", "req_abc")
                .set_body_json(sample_error("upstream_error", "boom")),
        )
        .mount(&server)
        .await;

    let err = client(&server).models().list().await.unwrap_err();
    assert_eq!(err.as_api().unwrap().request_id.as_deref(), Some("req_abc"));
}

#[tokio::test]
async fn tolerates_non_json_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(502).set_body_string("upstream timeout"))
        .mount(&server)
        .await;

    let err = client(&server).models().list().await.unwrap_err();
    let api = err.as_api().unwrap();
    assert_eq!(api.status, 502);
    assert_eq!(api.message, "upstream timeout");
}

#[tokio::test]
async fn raw_body_is_preserved() {
    let err = trigger_get_error(400, sample_error("bad_request", "nope")).await;
    let api = err.as_api().unwrap();
    let raw = api.raw.as_ref().expect("raw body");
    assert_eq!(raw["error"]["type"], "bad_request");
}

#[tokio::test]
async fn error_display_includes_status_and_message() {
    let err = trigger_get_error(401, sample_error("invalid_api_key", "bad key")).await;
    let rendered = err.to_string();
    assert!(rendered.contains("401"));
    assert!(rendered.contains("bad key"));
}
