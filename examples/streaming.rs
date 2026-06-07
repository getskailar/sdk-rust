//! Streamed chat completion.
//!
//! Run with:
//!
//! ```sh
//! SKAILAR_API_KEY=skl_live_... cargo run --example streaming
//! ```

use std::io::Write;

use futures_util::StreamExt;
use skailar::{ChatCompletionRequest, ChatMessage, Skailar, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?;

    let mut stream = client
        .chat()
        .completions()
        .create_stream(
            ChatCompletionRequest::builder()
                .model(models::CLAUDE_SONNET_4_6)
                .message(ChatMessage::user("Count to five, one number per line."))
                .build()?,
        )
        .await?;

    let mut stdout = std::io::stdout();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(piece) = chunk
            .choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
        {
            print!("{piece}");
            stdout.flush()?;
        }
    }
    println!();
    Ok(())
}
