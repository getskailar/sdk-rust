//! Image generation.

use crate::client::{Idempotency, Skailar};
use crate::error::Error;
use crate::types::images::{ImageGenerationRequest, ImageGenerationResponse};

/// Handle for the images resource. Obtain via [`Skailar::images`].
#[derive(Debug, Clone, Copy)]
pub struct Images<'a> {
    client: &'a Skailar,
}

impl<'a> Images<'a> {
    pub(crate) fn new(client: &'a Skailar) -> Self {
        Images { client }
    }

    /// Generates images from a prompt.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure. This is a billable side-effecting call and is not
    /// retried on `5xx`.
    pub async fn generate(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, Error> {
        self.client
            .post_json("v1/images/generations", &request, Idempotency::SideEffect)
            .await
    }
}
