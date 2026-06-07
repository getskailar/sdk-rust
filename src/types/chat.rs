//! Chat completion request and response types.

use serde::{Deserialize, Serialize};

use crate::types::shared::{Tool, ToolCall, ToolChoice, Usage};

/// Role of a chat message author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System / developer instructions.
    System,
    /// End-user input.
    User,
    /// Model output.
    Assistant,
    /// Result of a tool call, paired with a `tool_call_id`.
    Tool,
}

/// Reasoning budget for reasoning-capable models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Minimal reasoning.
    Low,
    /// Balanced reasoning.
    Medium,
    /// Maximum reasoning.
    High,
}

/// The content of a [`ChatMessage`]: either plain text, a list of parts, or
/// absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// A single text string.
    Text(String),
    /// An ordered list of multimodal parts.
    Parts(Vec<ContentPart>),
}

impl From<String> for MessageContent {
    fn from(value: String) -> Self {
        MessageContent::Text(value)
    }
}

impl From<&str> for MessageContent {
    fn from(value: &str) -> Self {
        MessageContent::Text(value.to_owned())
    }
}

impl From<Vec<ContentPart>> for MessageContent {
    fn from(value: Vec<ContentPart>) -> Self {
        MessageContent::Parts(value)
    }
}

/// One part of a multimodal message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// A run of text.
    Text {
        /// The text content.
        text: String,
    },
    /// An image, by `data:` URI or HTTPS URL.
    ImageUrl {
        /// The image reference and optional detail hint.
        image_url: ImageUrl,
    },
}

impl ContentPart {
    /// A text part.
    pub fn text(text: impl Into<String>) -> Self {
        ContentPart::Text { text: text.into() }
    }

    /// An image part from a `data:` URI or HTTPS URL.
    pub fn image_url(url: impl Into<String>) -> Self {
        ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }
}

/// An image reference within a [`ContentPart::ImageUrl`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    /// A `data:` URI or an HTTPS URL (e.g. from `/v1/uploads/images`).
    pub url: String,
    /// Optional detail hint: `"low"`, `"high"`, or `"auto"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Author role.
    pub role: Role,
    /// Message content; omitted on assistant messages that only carry tool
    /// calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    /// Tool calls requested by an assistant message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Identifier of the tool call this message responds to; required when
    /// `role` is [`Role::Tool`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// A `system` message.
    pub fn system(content: impl Into<MessageContent>) -> Self {
        ChatMessage {
            role: Role::System,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// A `user` message.
    pub fn user(content: impl Into<MessageContent>) -> Self {
        ChatMessage {
            role: Role::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// An `assistant` message.
    pub fn assistant(content: impl Into<MessageContent>) -> Self {
        ChatMessage {
            role: Role::Assistant,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// A `tool` result message, paired with the originating call id.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<MessageContent>) -> Self {
        ChatMessage {
            role: Role::Tool,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// A request to [`create`](crate::resources::chat::Completions::create) a chat
/// completion.
///
/// Construct with [`ChatCompletionRequest::builder`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier or alias. See [`crate::models`] for known constants.
    pub model: String,
    /// Conversation so far.
    pub messages: Vec<ChatMessage>,
    /// Request an SSE stream of chunks instead of a single response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Sampling temperature in `[0, 2]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Nucleus sampling probability in `[0, 1]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Reasoning budget for reasoning-capable models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Tool definitions the model may call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Constraint on tool calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// OpenAI-compatible response format object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,
    /// Number of completions to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Penalty for token presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Penalty for token frequency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Per-token logit bias map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<serde_json::Value>,
    /// End-user identifier for abuse monitoring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Seed for best-effort determinism.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Stop sequence(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequence>,
}

/// A stop condition: one sequence or several.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopSequence {
    /// A single stop string.
    One(String),
    /// Up to a handful of stop strings.
    Many(Vec<String>),
}

impl ChatCompletionRequest {
    /// Starts a builder for a chat completion request.
    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::default()
    }
}

/// Builder for [`ChatCompletionRequest`].
///
/// `model` and at least one message are required; [`build`](Self::build)
/// returns [`BuildError`] if `model` was never set.
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionRequestBuilder {
    model: Option<String>,
    messages: Vec<ChatMessage>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    reasoning_effort: Option<ReasoningEffort>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    response_format: Option<serde_json::Value>,
    n: Option<u32>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    logit_bias: Option<serde_json::Value>,
    user: Option<String>,
    seed: Option<i64>,
    stop: Option<StopSequence>,
}

impl ChatCompletionRequestBuilder {
    /// Sets the model identifier or alias.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Appends one message.
    pub fn message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Appends many messages.
    pub fn messages(mut self, messages: impl IntoIterator<Item = ChatMessage>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Requests streaming.
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the maximum tokens to generate.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the sampling temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the nucleus sampling probability.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the reasoning effort.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Sets the tool definitions.
    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools = Some(tools.into_iter().collect());
        self
    }

    /// Sets the tool-calling constraint.
    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the response format object.
    pub fn response_format(mut self, response_format: serde_json::Value) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Sets the number of completions.
    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the presence penalty.
    pub fn presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    /// Sets the frequency penalty.
    pub fn frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    /// Sets the logit bias map.
    pub fn logit_bias(mut self, logit_bias: serde_json::Value) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    /// Sets the end-user identifier.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets the determinism seed.
    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets a single stop sequence.
    pub fn stop(mut self, stop: impl Into<String>) -> Self {
        self.stop = Some(StopSequence::One(stop.into()));
        self
    }

    /// Sets several stop sequences.
    pub fn stop_sequences(mut self, stop: impl IntoIterator<Item = String>) -> Self {
        self.stop = Some(StopSequence::Many(stop.into_iter().collect()));
        self
    }

    /// Finalizes the request.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::MissingModel`] if [`model`](Self::model) was never
    /// called.
    pub fn build(self) -> Result<ChatCompletionRequest, BuildError> {
        let model = self.model.ok_or(BuildError::MissingModel)?;
        Ok(ChatCompletionRequest {
            model,
            messages: self.messages,
            stream: self.stream,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            reasoning_effort: self.reasoning_effort,
            tools: self.tools,
            tool_choice: self.tool_choice,
            response_format: self.response_format,
            n: self.n,
            presence_penalty: self.presence_penalty,
            frequency_penalty: self.frequency_penalty,
            logit_bias: self.logit_bias,
            user: self.user,
            seed: self.seed,
            stop: self.stop,
        })
    }
}

/// Error returned when a request builder is missing a required field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
    /// `model` was not set on the builder.
    #[error("`model` is required")]
    MissingModel,
    /// `prompt` was not set on the builder.
    #[error("`prompt` is required")]
    MissingPrompt,
    /// `input` was not set on the builder.
    #[error("`input` is required")]
    MissingInput,
}

/// Reason a completion stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop or a stop sequence was hit.
    Stop,
    /// The token limit was reached.
    Length,
    /// The model emitted tool calls.
    ToolCalls,
    /// Output was filtered.
    ContentFilter,
}

/// A non-streamed chat completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique completion identifier.
    pub id: String,
    /// Object type; always `"chat.completion"`.
    pub object: String,
    /// Unix epoch seconds at creation.
    pub created: u64,
    /// Model that produced the completion.
    pub model: String,
    /// One entry per generated choice.
    pub choices: Vec<Choice>,
    /// Token accounting.
    pub usage: Usage,
}

/// One choice within a [`ChatCompletionResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    /// Position of this choice in the list.
    pub index: u32,
    /// The generated message.
    pub message: ResponseMessage,
    /// Why generation stopped.
    pub finish_reason: FinishReason,
}

/// The assistant message inside a [`Choice`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Author role; always [`Role::Assistant`].
    pub role: Role,
    /// Generated text.
    pub content: String,
    /// Reasoning trace, for reasoning-capable models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Tool calls requested by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// One event in a streamed completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Completion identifier, stable across the stream.
    pub id: String,
    /// Object type; always `"chat.completion.chunk"`.
    pub object: String,
    /// Unix epoch seconds at creation.
    pub created: u64,
    /// Model producing the stream.
    pub model: String,
    /// Incremental choices.
    pub choices: Vec<ChunkChoice>,
    /// Cumulative token accounting, present on the final chunk(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// One choice within a [`ChatCompletionChunk`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// Position of this choice in the list.
    pub index: u32,
    /// The incremental delta for this choice.
    pub delta: Delta,
    /// Why generation stopped, on the final chunk for this choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// The incremental payload of a [`ChunkChoice`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    /// Author role, present on the first delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Text fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Reasoning trace fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Incremental tool-call fragments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// An incremental tool-call fragment within a [`Delta`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Index of the tool call being assembled.
    pub index: u32,
    /// Tool-call id, present on the first fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Discriminator, present on the first fragment.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Function name/argument fragments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// Incremental function name/arguments within a [`ToolCallDelta`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// Function name, present on the first fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Argument string fragment to be concatenated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_model() {
        let err = ChatCompletionRequest::builder()
            .message(ChatMessage::user("hi"))
            .build()
            .unwrap_err();
        assert_eq!(err, BuildError::MissingModel);
    }

    #[test]
    fn builder_sets_fields() {
        let req = ChatCompletionRequest::builder()
            .model("m")
            .message(ChatMessage::system("be brief"))
            .message(ChatMessage::user("hi"))
            .temperature(0.5)
            .stream(true)
            .build()
            .unwrap();
        assert_eq!(req.model, "m");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.temperature, Some(0.5));
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn omits_none_fields_in_json() {
        let req = ChatCompletionRequest::builder()
            .model("m")
            .message(ChatMessage::user("hi"))
            .build()
            .unwrap();
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("temperature").is_none());
        assert!(json.get("stream").is_none());
        assert_eq!(json["model"], "m");
    }

    #[test]
    fn user_message_serializes_role_lowercase() {
        let msg = ChatMessage::user("hi");
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "hi");
    }

    #[test]
    fn multimodal_content_roundtrips() {
        let msg = ChatMessage::user(vec![
            ContentPart::text("look:"),
            ContentPart::image_url("https://example.com/a.png"),
        ]);
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][1]["type"], "image_url");
        assert_eq!(
            json["content"][1]["image_url"]["url"],
            "https://example.com/a.png"
        );
    }

    #[test]
    fn tool_message_carries_call_id() {
        let msg = ChatMessage::tool("call_1", "42");
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "tool");
        assert_eq!(json["tool_call_id"], "call_1");
    }
}
