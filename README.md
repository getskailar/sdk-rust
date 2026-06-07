# Skailar SDK for Rust

[![crates.io](https://img.shields.io/crates/v/skailar.svg)](https://crates.io/crates/skailar)
[![docs.rs](https://img.shields.io/docsrs/skailar)](https://docs.rs/skailar)

The Skailar SDK for Rust provides access to the [Skailar API](https://skailar.com) — an OpenAI-compatible, multi-provider LLM gateway — from async Rust applications.

## Installation

```sh
cargo add skailar
```

This SDK is async-only and runs on any [tokio](https://tokio.rs)-compatible runtime.

## Getting started

```rust,no_run
use skailar::{ChatCompletionRequest, ChatMessage, Skailar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?; // reads SKAILAR_API_KEY

    let completion = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model("claude-sonnet-4-6")
                .message(ChatMessage::user("Hello, Skailar"))
                .build()?,
        )
        .await?;

    println!("{}", completion.choices[0].message.content);
    Ok(())
}
```

### Streaming

```rust,no_run
use futures_util::StreamExt;
use skailar::{ChatCompletionRequest, ChatMessage, Skailar};

# async fn run(client: Skailar) -> Result<(), Box<dyn std::error::Error>> {
let mut stream = client
    .chat()
    .completions()
    .create_stream(
        ChatCompletionRequest::builder()
            .model("claude-sonnet-4-6")
            .message(ChatMessage::user("Count to ten."))
            .build()?,
    )
    .await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
        print!("{content}");
    }
}
# Ok(())
# }
```

### Drop-in OpenAI replacement

The wire format mirrors OpenAI's. There is no official OpenAI Rust SDK, but if you already call OpenAI with a hand-rolled `reqwest` client, point this one at the same shape by setting the base URL:

```rust,no_run
# use skailar::Skailar;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let client = Skailar::builder()
    .api_key("skl_live_...")
    .base_url("https://api.skailar.com")
    .build()?;
# Ok(())
# }
```

## Requirements

Rust 1.85+ (Edition 2024). The library targets this MSRV; running the test suite needs a newer toolchain because a dev-dependency (`wiremock`) has a higher MSRV.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
