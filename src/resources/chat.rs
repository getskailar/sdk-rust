//! Chat completions.

use crate::client::{Idempotency, Skailar};
use crate::error::Error;
use crate::streaming::ChatCompletionStream;
use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse};

/// Handle for the chat resource. Obtain via [`Skailar::chat`].
#[derive(Debug, Clone, Copy)]
pub struct Chat<'a> {
    client: &'a Skailar,
}

impl<'a> Chat<'a> {
    pub(crate) fn new(client: &'a Skailar) -> Self {
        Chat { client }
    }

    /// Access the completions sub-resource.
    pub fn completions(&self) -> Completions<'a> {
        Completions {
            client: self.client,
        }
    }
}

/// Handle for `chat.completions`. Obtain via [`Chat::completions`].
#[derive(Debug, Clone, Copy)]
pub struct Completions<'a> {
    client: &'a Skailar,
}

impl Completions<'_> {
    /// Creates a non-streamed chat completion.
    ///
    /// For streaming, set `stream(true)` on the request and call
    /// [`create_stream`](Self::create_stream) instead.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure. This is a billable side-effecting call and is not
    /// retried on `5xx`.
    pub async fn create(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, Error> {
        self.client
            .post_json("v1/chat/completions", &request, Idempotency::SideEffect)
            .await
    }

    /// Creates a streamed chat completion.
    ///
    /// Sets `stream: true` on the wire regardless of the request's `stream`
    /// field, and returns a [`ChatCompletionStream`] of incremental chunks.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] if the initial response is non-2xx, or a transport
    /// variant on connection failure. Per-chunk decode failures surface as
    /// [`Error`] items from the stream.
    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, Error> {
        request.stream = Some(true);
        self.client
            .post_stream("v1/chat/completions", &request)
            .await
    }
}
