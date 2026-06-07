//! Minimal chat completion.
//!
//! Run with:
//!
//! ```sh
//! SKAILAR_API_KEY=skl_live_... cargo run --example chat
//! # against a local gateway:
//! SKAILAR_API_KEY=skl_live_... SKAILAR_BASE_URL=http://localhost:8080 cargo run --example chat
//! ```

use skailar::{ChatCompletionRequest, ChatMessage, Skailar, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?;

    let response = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model(models::CLAUDE_SONNET_4_6)
                .message(ChatMessage::system("You are concise."))
                .message(ChatMessage::user("Say hello in one short sentence."))
                .max_tokens(64)
                .build()?,
        )
        .await?;

    println!("{}", response.choices[0].message.content);
    println!("(tokens: {})", response.usage.total_tokens);
    Ok(())
}
