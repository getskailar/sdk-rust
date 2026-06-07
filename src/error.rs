//! Error types returned across the SDK.

use serde_json::Value;

/// Errors returned by every fallible operation in this crate.
///
/// Transport, decoding, timeout, and cancellation failures are distinct
/// variants so callers can react to each without string matching. API-level
/// failures (any non-2xx response with a parseable body) are carried by
/// [`Error::Api`] wrapping an [`ApiError`].
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No API key was supplied and `SKAILAR_API_KEY` was unset or empty.
    #[error("missing API key (pass api_key or set SKAILAR_API_KEY)")]
    MissingApiKey,

    /// The configured base URL could not be parsed.
    #[error("invalid base URL: {0}")]
    InvalidBaseUrl(#[from] url::ParseError),

    /// The gateway returned a non-2xx status.
    #[error("API error ({}): {}", .0.status, .0.message)]
    Api(Box<ApiError>),

    /// A transport-level failure occurred (DNS, TLS, connection reset).
    #[error("network error")]
    Network(#[source] reqwest::Error),

    /// The request exceeded the configured per-attempt timeout.
    #[error("request timed out after {timeout_secs}s")]
    Timeout {
        /// The per-attempt timeout, in seconds.
        timeout_secs: u64,
    },

    /// The request was aborted by the caller before completion.
    #[error("aborted by caller")]
    Aborted,

    /// A successful response carried a body that could not be deserialized.
    #[error("malformed response body")]
    Decode(#[source] serde_json::Error),

    /// A streaming event could not be parsed.
    #[error("malformed streaming event: {0}")]
    MalformedStreamEvent(String),
}

impl Error {
    /// Returns the [`ApiError`] if this is an API-level failure.
    pub fn as_api(&self) -> Option<&ApiError> {
        match self {
            Error::Api(e) => Some(e),
            _ => None,
        }
    }

    pub(crate) fn api(error: ApiError) -> Self {
        Error::Api(Box::new(error))
    }

    pub(crate) fn from_reqwest(err: reqwest::Error, timeout_secs: u64) -> Self {
        if err.is_timeout() {
            Error::Timeout { timeout_secs }
        } else {
            Error::Network(err)
        }
    }
}

/// A structured error returned by the Skailar gateway.
///
/// Use the predicate helpers ([`ApiError::is_auth`], [`ApiError::is_rate_limit`],
/// …) for branching, or match on [`ApiError::status`] directly.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError {
    /// HTTP status code of the response.
    pub status: u16,
    /// Machine-readable error code from the response body, when present.
    pub code: Option<String>,
    /// Human-readable error message.
    pub message: String,
    /// Server-assigned request identifier, when present in the response headers.
    pub request_id: Option<String>,
    /// The raw, parsed JSON body, when the response had one.
    pub raw: Option<Value>,
    /// Seconds to wait before retrying, parsed from `Retry-After` on a 429.
    ///
    /// This is the uncapped server value; the client's own retry loop caps the
    /// delay it actually waits at 60 seconds.
    pub retry_after: Option<u64>,
}

impl ApiError {
    /// Whether the status is `401 Unauthorized`.
    pub fn is_auth(&self) -> bool {
        self.status == 401
    }

    /// Whether the status is `400 Bad Request`.
    pub fn is_bad_request(&self) -> bool {
        self.status == 400
    }

    /// Whether the status is `404 Not Found`.
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }

    /// Whether the status is `429 Too Many Requests`.
    pub fn is_rate_limit(&self) -> bool {
        self.status == 429
    }

    /// Whether the status is in the `5xx` range.
    pub fn is_upstream(&self) -> bool {
        self.status >= 500
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API error ({}): {}", self.status, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Extracts `(code, message)` from a Skailar/OpenAI error body.
///
/// Tolerant of three layouts so the SDK keeps working if the gateway shape
/// shifts:
/// - flat: `{ "error": "code", "message": "msg" }`
/// - nested object: `{ "error": { "type" | "code": "...", "message": "..." } }`
/// - OpenAI-style: `{ "error": { "code": "...", "message": "..." } }`
pub(crate) fn parse_error_fields(body: &Value) -> (Option<String>, Option<String>) {
    let Some(error) = body.get("error") else {
        let message = body
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned);
        return (None, message);
    };

    if let Some(code) = error.as_str() {
        let message = body
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned);
        return (Some(code.to_owned()), message);
    }

    let code = error
        .get("type")
        .or_else(|| error.get("code"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_owned);
    (code, message)
}

pub(crate) fn build_api_error(
    status: u16,
    request_id: Option<String>,
    retry_after: Option<u64>,
    body_text: &str,
) -> ApiError {
    let raw: Option<Value> = serde_json::from_str(body_text).ok();
    let (code, message) = raw.as_ref().map(parse_error_fields).unwrap_or((None, None));

    let message = message.unwrap_or_else(|| {
        if body_text.trim().is_empty() {
            format!("HTTP {status}")
        } else {
            body_text.trim().to_owned()
        }
    });

    ApiError {
        status,
        code,
        message,
        request_id,
        raw,
        retry_after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_nested_type_field() {
        let body = json!({ "error": { "type": "invalid_api_key", "message": "bad key" } });
        let (code, message) = parse_error_fields(&body);
        assert_eq!(code.as_deref(), Some("invalid_api_key"));
        assert_eq!(message.as_deref(), Some("bad key"));
    }

    #[test]
    fn parses_openai_code_field() {
        let body = json!({ "error": { "code": "rate_limited", "message": "slow down" } });
        let (code, message) = parse_error_fields(&body);
        assert_eq!(code.as_deref(), Some("rate_limited"));
        assert_eq!(message.as_deref(), Some("slow down"));
    }

    #[test]
    fn parses_flat_error_string() {
        let body = json!({ "error": "bad_request", "message": "nope" });
        let (code, message) = parse_error_fields(&body);
        assert_eq!(code.as_deref(), Some("bad_request"));
        assert_eq!(message.as_deref(), Some("nope"));
    }

    #[test]
    fn build_falls_back_to_status_when_empty() {
        let err = build_api_error(500, None, None, "");
        assert_eq!(err.message, "HTTP 500");
        assert!(err.is_upstream());
    }

    #[test]
    fn predicates_match_status() {
        let mk = |status| build_api_error(status, None, None, "{}");
        assert!(mk(401).is_auth());
        assert!(mk(400).is_bad_request());
        assert!(mk(404).is_not_found());
        assert!(mk(429).is_rate_limit());
        assert!(mk(503).is_upstream());
    }
}
