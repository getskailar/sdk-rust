mod common;

use common::*;
use futures_util::StreamExt;
use skailar::{ChatCompletionRequest, ChatMessage, Error};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn collect_content(server: &MockServer) -> String {
    let client = client(server);
    let mut stream = client
        .chat()
        .completions()
        .create_stream(
            ChatCompletionRequest::builder()
                .model("claude-sonnet-4-6")
                .message(ChatMessage::user("hi"))
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let mut text = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if let Some(piece) = chunk
            .choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
        {
            text.push_str(piece);
        }
    }
    text
}

fn mount_sse(server_body: String) -> Mock {
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(server_body),
        )
}

#[tokio::test]
async fn streams_chunks_lf() {
    let server = MockServer::start().await;
    mount_sse(sse_stream(&["Hel", "lo", "!"], "\n"))
        .mount(&server)
        .await;
    assert_eq!(collect_content(&server).await, "Hello!");
}

#[tokio::test]
async fn streams_chunks_crlf() {
    let server = MockServer::start().await;
    mount_sse(sse_stream(&["Hel", "lo", "!"], "\r\n"))
        .mount(&server)
        .await;
    assert_eq!(collect_content(&server).await, "Hello!");
}

#[tokio::test]
async fn streams_chunks_cr() {
    let server = MockServer::start().await;
    mount_sse(sse_stream(&["Hel", "lo", "!"], "\r"))
        .mount(&server)
        .await;
    assert_eq!(collect_content(&server).await, "Hello!");
}

#[tokio::test]
async fn stops_at_done_sentinel() {
    let server = MockServer::start().await;
    // Extra data after [DONE] must be ignored.
    let mut body = sse_stream(&["a", "b"], "\n");
    body.push_str("data: {\"should\":\"be ignored\"}\n\n");
    mount_sse(body).mount(&server).await;
    assert_eq!(collect_content(&server).await, "ab");
}

#[tokio::test]
async fn final_chunk_has_finish_reason() {
    let server = MockServer::start().await;
    mount_sse(sse_stream(&["x"], "\n")).mount(&server).await;

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

    let mut saw_finish = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if chunk.choices[0].finish_reason.is_some() {
            saw_finish = true;
        }
    }
    assert!(saw_finish);
}

#[tokio::test]
async fn in_band_error_surfaces() {
    let server = MockServer::start().await;
    let body = "data: {\"error\":{\"type\":\"upstream_error\",\"message\":\"boom\"}}\n\n";
    mount_sse(body.to_string()).mount(&server).await;

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

    let first = stream.next().await.unwrap();
    match first {
        Err(Error::Api(api)) => {
            assert_eq!(api.message, "boom");
            assert_eq!(api.code.as_deref(), Some("upstream_error"));
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_event_surfaces() {
    let server = MockServer::start().await;
    let body = "data: {not valid json}\n\n";
    mount_sse(body.to_string()).mount(&server).await;

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

    match stream.next().await.unwrap() {
        Err(Error::MalformedStreamEvent(_)) => {}
        other => panic!("expected malformed stream event, got {other:?}"),
    }
}

#[tokio::test]
async fn early_drop_does_not_panic() {
    let server = MockServer::start().await;
    mount_sse(sse_stream(&["a", "b", "c", "d"], "\n"))
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

    // Consume one chunk, then drop the stream to cancel the body mid-flight.
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.choices[0].delta.content.as_deref(), Some("a"));
    drop(stream);
}
