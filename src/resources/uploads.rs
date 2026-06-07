//! Storage uploads.

use crate::client::{Idempotency, Skailar};
use crate::error::Error;
use crate::types::uploads::{FileContentType, ImageContentType, UploadRequest, UploadResponse};

/// Handle for the uploads resource. Obtain via [`Skailar::uploads`].
#[derive(Debug, Clone, Copy)]
pub struct Uploads<'a> {
    client: &'a Skailar,
}

impl<'a> Uploads<'a> {
    pub(crate) fn new(client: &'a Skailar) -> Self {
        Uploads { client }
    }

    /// Access the image-uploads sub-resource.
    pub fn images(&self) -> ImageUploads<'a> {
        ImageUploads {
            client: self.client,
        }
    }

    /// Access the file-uploads sub-resource.
    pub fn files(&self) -> FileUploads<'a> {
        FileUploads {
            client: self.client,
        }
    }
}

/// Handle for `uploads.images`. Obtain via [`Uploads::images`].
#[derive(Debug, Clone, Copy)]
pub struct ImageUploads<'a> {
    client: &'a Skailar,
}

impl ImageUploads<'_> {
    /// Uploads a base64-encoded image and returns its stored URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure. This is a side-effecting call and is not retried on
    /// `5xx`.
    pub async fn create(
        &self,
        base64: impl Into<String>,
        content_type: ImageContentType,
    ) -> Result<UploadResponse, Error> {
        let content_type = serde_content_type(&content_type);
        let request = UploadRequest {
            base64: base64.into(),
            content_type,
        };
        self.client
            .post_json("v1/uploads/images", &request, Idempotency::SideEffect)
            .await
    }
}

/// Handle for `uploads.files`. Obtain via [`Uploads::files`].
#[derive(Debug, Clone, Copy)]
pub struct FileUploads<'a> {
    client: &'a Skailar,
}

impl FileUploads<'_> {
    /// Uploads a base64-encoded document and returns its stored URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure. This is a side-effecting call and is not retried on
    /// `5xx`.
    pub async fn create(
        &self,
        base64: impl Into<String>,
        content_type: FileContentType,
    ) -> Result<UploadResponse, Error> {
        let content_type = serde_content_type(&content_type);
        let request = UploadRequest {
            base64: base64.into(),
            content_type,
        };
        self.client
            .post_json("v1/uploads/files", &request, Idempotency::SideEffect)
            .await
    }
}

/// Renders a content-type enum to its wire string via its serde representation.
fn serde_content_type<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => s,
        _ => String::new(),
    }
}
