//! Internal credential handling.
//!
//! The API key is held in an [`ApiKey`] newtype whose [`Debug`] implementation
//! redacts the secret, so it never lands in logs or panic messages by accident.

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};

/// A bearer credential, redacted in [`Debug`] output.
#[derive(Clone)]
pub(crate) struct ApiKey(String);

impl ApiKey {
    pub(crate) fn new(key: String) -> Self {
        ApiKey(key)
    }

    /// Builds the `Authorization` header value, marking it sensitive so reqwest
    /// keeps it out of its own logging.
    pub(crate) fn header_value(&self) -> HeaderValue {
        let mut value = HeaderValue::from_str(&format!("Bearer {}", self.0))
            .unwrap_or_else(|_| HeaderValue::from_static("Bearer"));
        value.set_sensitive(true);
        value
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(***redacted***)")
    }
}

/// Applies the bearer credential last, after dropping any caller-supplied
/// `Authorization` header (case-insensitively), so it can never be overridden.
pub(crate) fn apply_authorization(headers: &mut HeaderMap, key: &ApiKey) {
    headers.remove(AUTHORIZATION);
    headers.insert(AUTHORIZATION, key.header_value());
}

/// Removes a header by name, case-insensitively.
pub(crate) fn remove_header_ci(headers: &mut HeaderMap, name: &str) {
    if let Ok(parsed) = name.parse::<HeaderName>() {
        headers.remove(parsed);
    }
}
