mod common;

use common::*;
use skailar::{
    ChatCompletionRequest, ChatMessage, FileContentType, ImageContentType, ImageGenerationRequest,
    Mime, Skailar, SpeechRequest, TranscriptionRequest, Voice,
};
use wiremock::matchers::{body_json_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn chat_completion_returns_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header(
            "authorization",
            format!("Bearer {TEST_KEY}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_completion("Hi!")))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model("claude-sonnet-4-6")
                .message(ChatMessage::user("hi"))
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.choices[0].message.content, "Hi!");
    assert_eq!(res.usage.total_tokens, 8);
}

#[tokio::test]
async fn chat_completion_sends_expected_body() {
    let server = MockServer::start().await;
    let expected = serde_json::json!({
        "model": "m",
        "messages": [{ "role": "user", "content": "hi" }],
        "temperature": 0.5
    })
    .to_string();

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_json_string(expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_completion("ok")))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model("m")
                .message(ChatMessage::user("hi"))
                .temperature(0.5)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.choices[0].message.content, "ok");
}

#[tokio::test]
async fn default_header_is_sent() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("x-trace-id", "abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_completion("ok")))
        .mount(&server)
        .await;

    let client = Skailar::builder()
        .api_key(TEST_KEY)
        .base_url(server.uri())
        .max_retries(0)
        .default_header("x-trace-id", "abc123")
        .build()
        .unwrap();

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

#[tokio::test]
async fn models_list_unwraps_data() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_model_list()))
        .mount(&server)
        .await;

    let client = client(&server);
    let models = client.models().list().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "claude-sonnet-4-6");
    assert!(models[0].capabilities.vision);
}

#[tokio::test]
async fn models_retrieve_returns_detail() {
    let server = MockServer::start().await;
    let mut body = sample_model_summary("claude-sonnet-4-6");
    body["description"] = serde_json::json!("A capable model");
    body["aliases"] = serde_json::json!(["claude-sonnet"]);

    Mock::given(method("GET"))
        .and(path("/v1/models/claude-sonnet-4-6"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = client(&server);
    let model = client.models().retrieve("claude-sonnet-4-6").await.unwrap();
    assert_eq!(model.summary.id, "claude-sonnet-4-6");
    assert_eq!(model.description.as_deref(), Some("A capable model"));
}

#[tokio::test]
async fn models_retrieve_preserves_slash_in_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models/google/gemini-2.5-pro"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(sample_model_summary("google/gemini-2.5-pro")),
        )
        .mount(&server)
        .await;

    let client = client(&server);
    let model = client
        .models()
        .retrieve("google/gemini-2.5-pro")
        .await
        .unwrap();
    assert_eq!(model.summary.id, "google/gemini-2.5-pro");
}

#[tokio::test]
async fn images_generate_returns_data() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "created": 1_700_000_000,
        "data": [{ "url": "https://cdn.skailar.com/a.png", "revised_prompt": "a cat" }]
    });
    Mock::given(method("POST"))
        .and(path("/v1/images/generations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .images()
        .generate(
            ImageGenerationRequest::builder()
                .model("gpt-image-1")
                .prompt("a cat")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.data[0].url.as_deref(),
        Some("https://cdn.skailar.com/a.png")
    );
}

#[tokio::test]
async fn transcription_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "text": "hello world" })),
        )
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .audio()
        .transcriptions()
        .create(
            TranscriptionRequest::builder()
                .base64("AAAA")
                .mime(Mime::Wav)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.text, "hello world");
}

#[tokio::test]
async fn speech_returns_audio_bytes() {
    let server = MockServer::start().await;
    let audio = b"ID3\x04\x00fake-mp3-bytes".to_vec();
    Mock::given(method("POST"))
        .and(path("/v1/audio/speech"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/mpeg")
                .set_body_bytes(audio.clone()),
        )
        .mount(&server)
        .await;

    let client = client(&server);
    let bytes = client
        .audio()
        .speech()
        .create_bytes(
            SpeechRequest::builder()
                .input("hello")
                .voice(Voice::Nova)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bytes.as_ref(), audio.as_slice());
}

#[tokio::test]
async fn upload_image_returns_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/images"))
        .and(body_json_string(
            serde_json::json!({ "base64": "AAAA", "content_type": "image/png" }).to_string(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://cdn.skailar.com/u/1.png",
            "content_type": "image/png"
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .uploads()
        .images()
        .create("AAAA", ImageContentType::Png)
        .await
        .unwrap();
    assert_eq!(res.url, "https://cdn.skailar.com/u/1.png");
}

#[tokio::test]
async fn upload_file_returns_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/files"))
        .and(body_json_string(
            serde_json::json!({ "base64": "JVBER", "content_type": "application/pdf" }).to_string(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://cdn.skailar.com/u/1.pdf",
            "content_type": "application/pdf"
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client
        .uploads()
        .files()
        .create("JVBER", FileContentType::Pdf)
        .await
        .unwrap();
    assert_eq!(res.content_type, "application/pdf");
}

#[tokio::test]
async fn ping_returns_user_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ping-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "user_id": "11111111-1111-1111-1111-111111111111"
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let res = client.ping().await.unwrap();
    assert_eq!(res.status, "ok");
    assert_eq!(res.user_id, "11111111-1111-1111-1111-111111111111");
}
