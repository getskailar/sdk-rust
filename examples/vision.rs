//! Multimodal (vision) chat completion using an image URL.
//!
//! Run with:
//!
//! ```sh
//! SKAILAR_API_KEY=skl_live_... cargo run --example vision
//! ```

use skailar::{ChatCompletionRequest, ChatMessage, ContentPart, Skailar, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?;

    let message = ChatMessage::user(vec![
        ContentPart::text("What is in this image?"),
        ContentPart::image_url(
            "https://upload.wikimedia.org/wikipedia/commons/thumb/3/3a/Cat03.jpg/640px-Cat03.jpg",
        ),
    ]);

    let response = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model(models::CLAUDE_SONNET_4_6)
                .message(message)
                .max_tokens(128)
                .build()?,
        )
        .await?;

    println!("{}", response.choices[0].message.content);
    Ok(())
}
