//! Audio transcription and speech synthesis.

use bytes::Bytes;
use futures_util::Stream;

use crate::client::Skailar;
use crate::error::Error;
use crate::types::audio::{SpeechRequest, TranscriptionRequest, TranscriptionResponse};

/// Handle for the audio resource. Obtain via [`Skailar::audio`].
#[derive(Debug, Clone, Copy)]
pub struct Audio<'a> {
    client: &'a Skailar,
}

impl<'a> Audio<'a> {
    pub(crate) fn new(client: &'a Skailar) -> Self {
        Audio { client }
    }

    /// Access the transcriptions sub-resource.
    pub fn transcriptions(&self) -> Transcriptions<'a> {
        Transcriptions {
            client: self.client,
        }
    }

    /// Access the speech sub-resource.
    pub fn speech(&self) -> Speech<'a> {
        Speech {
            client: self.client,
        }
    }
}

/// Handle for `audio.transcriptions`. Obtain via [`Audio::transcriptions`].
#[derive(Debug, Clone, Copy)]
pub struct Transcriptions<'a> {
    client: &'a Skailar,
}

impl Transcriptions<'_> {
    /// Transcribes base64-encoded audio to text.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure. This is a billable side-effecting call and is not
    /// retried on `5xx`.
    pub async fn create(
        &self,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResponse, Error> {
        self.client
            .post_json(
                "v1/audio/transcriptions",
                &request,
                crate::client::Idempotency::SideEffect,
            )
            .await
    }
}

/// Handle for `audio.speech`. Obtain via [`Audio::speech`].
#[derive(Debug, Clone, Copy)]
pub struct Speech<'a> {
    client: &'a Skailar,
}

impl Speech<'_> {
    /// Synthesizes speech and returns a stream of MP3 (`audio/mpeg`) bytes.
    ///
    /// Use [`create_bytes`](Self::create_bytes) to collect the whole clip into
    /// memory instead of streaming it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] if the initial response is non-2xx, or a transport
    /// variant on connection failure. Mid-stream transport failures surface as
    /// [`Error`] items from the stream.
    pub async fn create(
        &self,
        request: SpeechRequest,
    ) -> Result<impl Stream<Item = Result<Bytes, Error>> + use<>, Error> {
        self.client
            .post_binary("v1/audio/speech", &request, "audio/mpeg")
            .await
    }

    /// Synthesizes speech and collects the full MP3 clip into a [`Bytes`].
    ///
    /// # Errors
    ///
    /// As [`create`](Self::create), plus any mid-stream transport error
    /// encountered while collecting.
    pub async fn create_bytes(&self, request: SpeechRequest) -> Result<Bytes, Error> {
        use futures_util::StreamExt;

        let mut stream = Box::pin(self.create(request).await?);
        let mut buf = Vec::new();
        while let Some(chunk) = stream.next().await {
            buf.extend_from_slice(&chunk?);
        }
        Ok(Bytes::from(buf))
    }
}
