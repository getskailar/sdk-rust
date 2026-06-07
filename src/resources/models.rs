//! Model catalog.

use crate::client::Skailar;
use crate::error::Error;
use crate::types::models::{Model, ModelList, ModelSummary};

/// Handle for the models resource. Obtain via [`Skailar::models`].
#[derive(Debug, Clone, Copy)]
pub struct Models<'a> {
    client: &'a Skailar,
}

impl<'a> Models<'a> {
    pub(crate) fn new(client: &'a Skailar) -> Self {
        Models { client }
    }

    /// Lists every model the gateway can route to.
    ///
    /// Returns the unwrapped `data` array from the `{ object: "list", data }`
    /// envelope.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] for a non-2xx response, or a transport/decoding
    /// variant on failure.
    pub async fn list(&self) -> Result<Vec<ModelSummary>, Error> {
        let list: ModelList = self.client.get_json("v1/models").await?;
        Ok(list.data)
    }

    /// Retrieves a model's full detail card.
    ///
    /// The id may contain slashes (e.g. `"google/gemini-2.5-pro"`); each segment
    /// is percent-encoded so the path is preserved.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] with status 404 if the model does not exist, or a
    /// transport/decoding variant on failure.
    pub async fn retrieve(&self, id: impl AsRef<str>) -> Result<Model, Error> {
        let path = format!("v1/models/{}", encode_path(id.as_ref()));
        self.client.get_json(&path).await
    }
}

/// Percent-encodes each segment of an id while keeping `/` separators intact.
fn encode_path(id: &str) -> String {
    id.split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_slashes_encodes_specials() {
        assert_eq!(
            encode_path("google/gemini-2.5-pro"),
            "google/gemini-2.5-pro"
        );
        assert_eq!(encode_path("a b"), "a%20b");
        assert_eq!(encode_path("ns/a b"), "ns/a%20b");
    }
}
