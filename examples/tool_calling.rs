//! Function/tool calling round trip.
//!
//! Run with:
//!
//! ```sh
//! SKAILAR_API_KEY=skl_live_... cargo run --example tool_calling
//! ```

use serde_json::json;
use skailar::{ChatCompletionRequest, ChatMessage, Skailar, Tool, ToolChoice, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?;

    let weather = Tool::function(
        "get_weather",
        "Get the current weather for a city.",
        json!({
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"]
        }),
    );

    let response = client
        .chat()
        .completions()
        .create(
            ChatCompletionRequest::builder()
                .model(models::CLAUDE_SONNET_4_6)
                .message(ChatMessage::user("What's the weather in Paris?"))
                .tools([weather])
                .tool_choice(ToolChoice::auto())
                .build()?,
        )
        .await?;

    let choice = &response.choices[0];
    if let Some(tool_calls) = &choice.message.tool_calls {
        for call in tool_calls {
            println!(
                "model wants to call {}({})",
                call.function.name, call.function.arguments
            );
        }
    } else {
        println!("{}", choice.message.content);
    }
    Ok(())
}
