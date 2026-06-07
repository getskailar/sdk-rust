//! Storage upload request and response types.

use serde::{Deserialize, Serialize};

/// Content type accepted by `POST /v1/uploads/images`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageContentType {
    /// `image/png`
    #[serde(rename = "image/png")]
    Png,
    /// `image/jpeg`
    #[serde(rename = "image/jpeg")]
    Jpeg,
    /// `image/gif`
    #[serde(rename = "image/gif")]
    Gif,
    /// `image/webp`
    #[serde(rename = "image/webp")]
    Webp,
}

/// Content type accepted by `POST /v1/uploads/files`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileContentType {
    /// `application/pdf`
    #[serde(rename = "application/pdf")]
    Pdf,
    /// `text/plain`
    #[serde(rename = "text/plain")]
    Text,
}

/// A request to upload a base64 payload to Skailar storage.
///
/// The `content_type` field is a free-form string on the wire; the resource
/// methods accept the [`ImageContentType`] / [`FileContentType`] enums and set
/// it for you.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadRequest {
    /// Base64-encoded payload (no `data:` prefix).
    pub base64: String,
    /// MIME type of the payload.
    pub content_type: String,
}

/// The response from an upload endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Skailar-relative URL of the stored asset, ready to embed in subsequent
    /// calls.
    pub url: String,
    /// MIME type of the stored asset.
    pub content_type: String,
}
