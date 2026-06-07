//! Official Rust SDK for the [Skailar](https://skailar.com) API.
//!
//! Skailar is a multi-provider LLM gateway with an OpenAI-compatible surface.
//! This crate is an async-only client built on [`reqwest`] and any reqwest-
//! compatible async runtime ([`tokio`](https://tokio.rs) recommended).
//!
//! # Quickstart
//!
//! ```no_run
//! use skailar::{ChatCompletionRequest, ChatMessage, Skailar};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Skailar::new()?; // reads SKAILAR_API_KEY
//!
//!     let response = client
//!         .chat()
//!         .completions()
//!         .create(
//!             ChatCompletionRequest::builder()
//!                 .model("claude-sonnet-4-6")
//!                 .message(ChatMessage::user("Hello!"))
//!                 .build()?,
//!         )
//!         .await?;
//!
//!     println!("{}", response.choices[0].message.content);
//!     Ok(())
//! }
//! ```
//!
//! # Streaming
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use skailar::{ChatCompletionRequest, ChatMessage, Skailar};
//!
//! # async fn run(client: Skailar) -> Result<(), Box<dyn std::error::Error>> {
//! let mut stream = client
//!     .chat()
//!     .completions()
//!     .create_stream(
//!         ChatCompletionRequest::builder()
//!             .model("claude-sonnet-4-6")
//!             .message(ChatMessage::user("Count to 5"))
//!             .build()?,
//!     )
//!     .await?;
//!
//! while let Some(chunk) = stream.next().await {
//!     let chunk = chunk?;
//!     if let Some(piece) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
//!         print!("{piece}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Authentication
//!
//! Pass an API key explicitly via [`Skailar::builder`], or set `SKAILAR_API_KEY`
//! and use [`Skailar::new`]. Keys have the form `skl_live_<43 url-safe base64>`
//! and are sent as `Authorization: Bearer …`.
//!
//! # Errors
//!
//! Every fallible call returns [`Error`]. API-level failures are carried by
//! [`Error::Api`] wrapping an [`ApiError`] with status-predicate helpers; see
//! the [`error`] module.
//!
//! # Feature flags
//!
//! - `tracing` (off by default): emit `tracing` debug spans for requests and
//!   retries.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod resources;
pub mod streaming;
pub mod types;

#[doc(inline)]
pub use client::{Skailar, SkailarBuilder};
#[doc(inline)]
pub use error::{ApiError, Error};
#[doc(inline)]
pub use streaming::ChatCompletionStream;

#[doc(inline)]
pub use types::audio::{
    Mime, SpeechRequest, SpeechRequestBuilder, TranscriptionRequest, TranscriptionRequestBuilder,
    TranscriptionResponse, Voice,
};
#[doc(inline)]
pub use types::chat::{
    BuildError, ChatCompletionChunk, ChatCompletionRequest, ChatCompletionRequestBuilder,
    ChatCompletionResponse, ChatMessage, Choice, ChunkChoice, ContentPart, Delta, FinishReason,
    FunctionCallDelta, ImageUrl, MessageContent, ReasoningEffort, ResponseMessage, Role,
    StopSequence, ToolCallDelta,
};
#[doc(inline)]
pub use types::images::{
    GeneratedImage, ImageGenerationRequest, ImageGenerationRequestBuilder, ImageGenerationResponse,
};
#[doc(inline)]
pub use types::models::{
    Modalities, Model, ModelCapabilities, ModelList, ModelPricing, ModelSummary,
};
#[doc(inline)]
pub use types::ping::PingKeyResponse;
#[doc(inline)]
pub use types::shared::{
    FunctionCall, FunctionDef, NamedFunction, NamedToolChoice, Tool, ToolCall, ToolChoice, Usage,
};
#[doc(inline)]
pub use types::uploads::{FileContentType, ImageContentType, UploadRequest, UploadResponse};
