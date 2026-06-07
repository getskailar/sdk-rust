//! Image generation request and response types.

use serde::{Deserialize, Serialize};

use crate::types::chat::BuildError;

/// A request to [`generate`](crate::resources::images::Images::generate)
/// images.
///
/// Construct with [`ImageGenerationRequest::builder`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Image model identifier (e.g. `"gpt-image-1"`).
    pub model: String,
    /// Text prompt describing the desired image.
    pub prompt: String,
    /// Number of images to generate (1–10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Output size, e.g. `"1024x1024"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Provider-specific quality (e.g. `"standard"`, `"hd"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Provider-specific background (e.g. `"transparent"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

impl ImageGenerationRequest {
    /// Starts a builder for an image generation request.
    pub fn builder() -> ImageGenerationRequestBuilder {
        ImageGenerationRequestBuilder::default()
    }
}

/// Builder for [`ImageGenerationRequest`].
///
/// `model` and `prompt` are required.
#[derive(Debug, Clone, Default)]
pub struct ImageGenerationRequestBuilder {
    model: Option<String>,
    prompt: Option<String>,
    n: Option<u32>,
    size: Option<String>,
    quality: Option<String>,
    background: Option<String>,
}

impl ImageGenerationRequestBuilder {
    /// Sets the image model.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the prompt.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Sets the number of images to generate.
    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the output size.
    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the quality.
    pub fn quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    /// Sets the background.
    pub fn background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Finalizes the request.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::MissingModel`] or [`BuildError::MissingPrompt`] if
    /// the respective field was not set.
    pub fn build(self) -> Result<ImageGenerationRequest, BuildError> {
        let model = self.model.ok_or(BuildError::MissingModel)?;
        let prompt = self.prompt.ok_or(BuildError::MissingPrompt)?;
        Ok(ImageGenerationRequest {
            model,
            prompt,
            n: self.n,
            size: self.size,
            quality: self.quality,
            background: self.background,
        })
    }
}

/// The response from `POST /v1/images/generations`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Unix epoch seconds at creation.
    pub created: u64,
    /// One entry per generated image.
    pub data: Vec<GeneratedImage>,
}

/// A single generated image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// URL to the generated image, when the provider returns a hosted asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Base64-encoded image bytes, when the provider inlines the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    /// Prompt as rewritten by the provider, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}
