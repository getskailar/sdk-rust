//! Speech synthesis: write an MP3 to disk.
//!
//! Run with:
//!
//! ```sh
//! SKAILAR_API_KEY=skl_live_... cargo run --example speech
//! ```

use skailar::{Skailar, SpeechRequest, Voice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Skailar::new()?;

    let audio = client
        .audio()
        .speech()
        .create_bytes(
            SpeechRequest::builder()
                .input("Hello from the Skailar Rust SDK.")
                .voice(Voice::Nova)
                .build()?,
        )
        .await?;

    let path = "speech.mp3";
    std::fs::write(path, &audio)?;
    println!("wrote {} bytes to {path}", audio.len());
    Ok(())
}
