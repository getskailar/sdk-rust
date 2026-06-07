//! Audio transcription and speech synthesis types.

use serde::{Deserialize, Serialize};

use crate::types::chat::BuildError;

/// Audio MIME type for a transcription request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mime {
    /// `audio/wav`
    #[serde(rename = "audio/wav")]
    Wav,
    /// `audio/webm`
    #[serde(rename = "audio/webm")]
    Webm,
    /// `audio/mp4`
    #[serde(rename = "audio/mp4")]
    Mp4,
    /// `audio/m4a`
    #[serde(rename = "audio/m4a")]
    M4a,
    /// `audio/mpeg`
    #[serde(rename = "audio/mpeg")]
    Mpeg,
    /// `audio/mp3`
    #[serde(rename = "audio/mp3")]
    Mp3,
}

/// A request to
/// [`create`](crate::resources::audio::Transcriptions::create) a transcription.
///
/// Construct with [`TranscriptionRequest::builder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    /// Base64-encoded audio bytes (no `data:` prefix).
    pub base64: String,
    /// Source audio MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<Mime>,
}

impl TranscriptionRequest {
    /// Starts a builder for a transcription request.
    pub fn builder() -> TranscriptionRequestBuilder {
        TranscriptionRequestBuilder::default()
    }
}

/// Builder for [`TranscriptionRequest`].
///
/// The base64 audio payload is required.
#[derive(Debug, Clone, Default)]
pub struct TranscriptionRequestBuilder {
    base64: Option<String>,
    mime: Option<Mime>,
}

impl TranscriptionRequestBuilder {
    /// Sets the base64-encoded audio payload.
    pub fn base64(mut self, base64: impl Into<String>) -> Self {
        self.base64 = Some(base64.into());
        self
    }

    /// Sets the source MIME type.
    pub fn mime(mut self, mime: Mime) -> Self {
        self.mime = Some(mime);
        self
    }

    /// Finalizes the request.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::MissingInput`] if no audio payload was set.
    pub fn build(self) -> Result<TranscriptionRequest, BuildError> {
        let base64 = self.base64.ok_or(BuildError::MissingInput)?;
        Ok(TranscriptionRequest {
            base64,
            mime: self.mime,
        })
    }
}

/// The response from `POST /v1/audio/transcriptions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    /// Transcribed text.
    pub text: String,
}

/// A voice for speech synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Voice {
    /// `alloy`
    Alloy,
    /// `ash`
    Ash,
    /// `ballad`
    Ballad,
    /// `coral`
    Coral,
    /// `echo`
    Echo,
    /// `fable`
    Fable,
    /// `nova`
    Nova,
    /// `onyx`
    Onyx,
    /// `sage`
    Sage,
    /// `shimmer`
    Shimmer,
}

/// A request to [`create`](crate::resources::audio::Speech::create) synthesized
/// speech.
///
/// Construct with [`SpeechRequest::builder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeechRequest {
    /// Text to synthesize (max 4000 characters).
    pub input: String,
    /// Voice to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
}

impl SpeechRequest {
    /// Starts a builder for a speech request.
    pub fn builder() -> SpeechRequestBuilder {
        SpeechRequestBuilder::default()
    }
}

/// Builder for [`SpeechRequest`].
///
/// `input` is required.
#[derive(Debug, Clone, Default)]
pub struct SpeechRequestBuilder {
    input: Option<String>,
    voice: Option<Voice>,
}

impl SpeechRequestBuilder {
    /// Sets the text to synthesize.
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.input = Some(input.into());
        self
    }

    /// Sets the voice.
    pub fn voice(mut self, voice: Voice) -> Self {
        self.voice = Some(voice);
        self
    }

    /// Finalizes the request.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::MissingInput`] if no input text was set.
    pub fn build(self) -> Result<SpeechRequest, BuildError> {
        let input = self.input.ok_or(BuildError::MissingInput)?;
        Ok(SpeechRequest {
            input,
            voice: self.voice,
        })
    }
}
