//! Request and response types for every resource.

pub mod audio;
pub mod chat;
pub mod images;
pub mod models;
pub mod ping;
pub mod shared;
pub mod uploads;

pub use audio::{
    Mime, SpeechRequest, SpeechRequestBuilder, TranscriptionRequest, TranscriptionRequestBuilder,
    TranscriptionResponse, Voice,
};
pub use chat::{
    BuildError, ChatCompletionChunk, ChatCompletionRequest, ChatCompletionRequestBuilder,
    ChatCompletionResponse, ChatMessage, Choice, ChunkChoice, ContentPart, Delta, FinishReason,
    FunctionCallDelta, ImageUrl, MessageContent, ReasoningEffort, ResponseMessage, Role,
    StopSequence, ToolCallDelta,
};
pub use images::{
    GeneratedImage, ImageGenerationRequest, ImageGenerationRequestBuilder, ImageGenerationResponse,
};
pub use models::{Modalities, Model, ModelCapabilities, ModelList, ModelPricing, ModelSummary};
pub use ping::PingKeyResponse;
pub use shared::{
    FunctionCall, FunctionDef, NamedFunction, NamedToolChoice, Tool, ToolCall, ToolChoice, Usage,
};
pub use uploads::{FileContentType, ImageContentType, UploadRequest, UploadResponse};
