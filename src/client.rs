//! The [`Skailar`] client: construction, configuration, and request dispatch.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, RETRY_AFTER};
use reqwest::{Method, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::{ApiKey, apply_authorization, remove_header_ci};
use crate::error::{Error, build_api_error};
use crate::resources::audio::Audio;
use crate::resources::chat::Chat;
use crate::resources::images::Images;
use crate::resources::models::Models;
use crate::resources::uploads::Uploads;
use crate::streaming::ChatCompletionStream;
use crate::types::ping::PingKeyResponse;

const DEFAULT_BASE_URL: &str = "https://api.skailar.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_MAX_RETRIES: u32 = 2;
const BACKOFF_BASE_MS: u64 = 500;
const BACKOFF_CAP_MS: u64 = 8_000;
const MAX_RETRY_AFTER_SECS: u64 = 60;
const USER_AGENT: &str = concat!("skailar-rust/", env!("CARGO_PKG_VERSION"));

/// Async client for the Skailar API.
///
/// Cloning is cheap: the client is an [`Arc`] around shared state, so clones
/// share one connection pool. The client is [`Send`] + [`Sync`] and intended to
/// be created once and reused.
///
/// # Examples
///
/// ```no_run
/// use skailar::Skailar;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Skailar::new()?; // reads SKAILAR_API_KEY
/// let pong = client.ping().await?;
/// println!("{}", pong.user_id);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Skailar {
    pub(crate) inner: Arc<Inner>,
}

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) api_key: ApiKey,
    pub(crate) default_headers: HeaderMap,
    pub(crate) timeout: Duration,
    pub(crate) max_retries: u32,
    jitter: AtomicU64,
}

/// Whether a request may be safely replayed after a `5xx` or transport failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Idempotency {
    /// Safe to retry on `5xx` and connection errors (e.g. `GET`).
    Idempotent,
    /// Has billable side effects; only retried on `429` (rejected pre-execution).
    SideEffect,
}

impl Skailar {
    /// Constructs a client from the environment.
    ///
    /// Reads `SKAILAR_API_KEY` (required) and `SKAILAR_BASE_URL` (optional,
    /// defaults to `https://api.skailar.com`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingApiKey`] if `SKAILAR_API_KEY` is unset or empty,
    /// or [`Error::InvalidBaseUrl`] if `SKAILAR_BASE_URL` cannot be parsed.
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Starts a [`SkailarBuilder`] for explicit configuration.
    pub fn builder() -> SkailarBuilder {
        SkailarBuilder::default()
    }

    /// Chat completions: `client.chat().completions().create(...)`.
    pub fn chat(&self) -> Chat<'_> {
        Chat::new(self)
    }

    /// Model catalog: `client.models().list()` / `.retrieve(id)`.
    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    /// Image generation: `client.images().generate(...)`.
    pub fn images(&self) -> Images<'_> {
        Images::new(self)
    }

    /// Audio: `client.audio().transcriptions()` / `.speech()`.
    pub fn audio(&self) -> Audio<'_> {
        Audio::new(self)
    }

    /// Storage uploads: `client.uploads().images()` / `.files()`.
    pub fn uploads(&self) -> Uploads<'_> {
        Uploads::new(self)
    }

    /// Verifies the API key against `GET /v1/ping-key`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] with status 401 if the key is invalid, or a
    /// transport error variant on network failure.
    pub async fn ping(&self) -> Result<PingKeyResponse, Error> {
        self.get_json("v1/ping-key").await
    }

    pub(crate) fn endpoint(&self, path: &str) -> Result<Url, Error> {
        let base = self.inner.base_url.as_str();
        let joined = format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        Ok(Url::parse(&joined)?)
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        let url = self.endpoint(path)?;
        let response = self
            .execute(Method::GET, url, NoBody, Idempotency::Idempotent)
            .await?;
        decode_json(response).await
    }

    pub(crate) async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        idempotency: Idempotency,
    ) -> Result<T, Error> {
        let url = self.endpoint(path)?;
        let response = self
            .execute(Method::POST, url, JsonBody(body), idempotency)
            .await?;
        decode_json(response).await
    }

    pub(crate) async fn post_stream<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<ChatCompletionStream, Error> {
        let url = self.endpoint(path)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );
        let response = self
            .execute_with_headers(
                Method::POST,
                url,
                JsonBody(body),
                Idempotency::SideEffect,
                headers,
            )
            .await?;
        Ok(ChatCompletionStream::from_response(response))
    }

    pub(crate) async fn post_binary<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        accept: &'static str,
    ) -> Result<impl futures_util::Stream<Item = Result<Bytes, Error>> + use<B>, Error> {
        let url = self.endpoint(path)?;
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static(accept));
        let response = self
            .execute_with_headers(
                Method::POST,
                url,
                JsonBody(body),
                Idempotency::SideEffect,
                headers,
            )
            .await?;
        let timeout = self.inner.timeout.as_secs();
        Ok(futures_util::TryStreamExt::map_err(
            response.bytes_stream(),
            move |e| Error::from_reqwest(e, timeout),
        ))
    }

    async fn execute<B: RequestBody>(
        &self,
        method: Method,
        url: Url,
        body: B,
        idempotency: Idempotency,
    ) -> Result<Response, Error> {
        self.execute_with_headers(method, url, body, idempotency, HeaderMap::new())
            .await
    }

    async fn execute_with_headers<B: RequestBody>(
        &self,
        method: Method,
        url: Url,
        body: B,
        idempotency: Idempotency,
        per_call_headers: HeaderMap,
    ) -> Result<Response, Error> {
        let timeout_secs = self.inner.timeout.as_secs();
        let max_attempts = self.inner.max_retries + 1;

        let mut attempt: u32 = 0;
        loop {
            let request = self.build_request(&method, url.clone(), &body, &per_call_headers)?;

            #[cfg(feature = "tracing")]
            tracing::debug!(%method, %url, attempt, "skailar request");

            let outcome = self.inner.http.execute(request).await;

            let response = match outcome {
                Ok(response) => response,
                Err(err) => {
                    let mapped = Error::from_reqwest(err, timeout_secs);
                    let transient = matches!(mapped, Error::Network(_) | Error::Timeout { .. });
                    if transient
                        && idempotency == Idempotency::Idempotent
                        && attempt + 1 < max_attempts
                    {
                        self.sleep_backoff(attempt, None).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(mapped);
                }
            };

            let status = response.status();
            if status.is_success() {
                return Ok(response);
            }

            let retry_after = parse_retry_after(response.headers());
            if self.should_retry(status, idempotency, attempt, max_attempts) {
                self.sleep_backoff(attempt, retry_after).await;
                attempt += 1;
                continue;
            }

            return Err(self.api_error_from(response, retry_after).await);
        }
    }

    fn build_request<B: RequestBody>(
        &self,
        method: &Method,
        url: Url,
        body: &B,
        per_call_headers: &HeaderMap,
    ) -> Result<reqwest::Request, Error> {
        let mut headers = self.inner.default_headers.clone();
        for (name, value) in per_call_headers {
            headers.insert(name.clone(), value.clone());
        }
        // The bearer token is owned by the SDK; no caller header may shadow it.
        remove_header_ci(&mut headers, "authorization");
        apply_authorization(&mut headers, &self.inner.api_key);

        let mut builder = self
            .inner
            .http
            .request(method.clone(), url)
            .timeout(self.inner.timeout)
            .headers(headers);
        builder = body.apply(builder);
        builder.build().map_err(Error::Network)
    }

    fn should_retry(
        &self,
        status: StatusCode,
        idempotency: Idempotency,
        attempt: u32,
        max_attempts: u32,
    ) -> bool {
        if attempt + 1 >= max_attempts {
            return false;
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return true;
        }
        status.is_server_error() && idempotency == Idempotency::Idempotent
    }

    async fn sleep_backoff(&self, attempt: u32, retry_after: Option<u64>) {
        let delay = self.backoff_delay(attempt, retry_after);
        #[cfg(feature = "tracing")]
        tracing::debug!(
            attempt,
            delay_ms = delay.as_millis() as u64,
            "skailar retry backoff"
        );
        futures_timer::Delay::new(delay).await;
    }

    fn backoff_delay(&self, attempt: u32, retry_after: Option<u64>) -> Duration {
        if let Some(secs) = retry_after {
            return Duration::from_secs(secs.min(MAX_RETRY_AFTER_SECS));
        }
        let exponential =
            BACKOFF_CAP_MS.min(BACKOFF_BASE_MS.saturating_mul(1u64 << attempt.min(20)));
        // Full jitter in [0, exponential] without an external RNG dependency.
        let jitter = self.next_jitter();
        let millis = if exponential == 0 {
            0
        } else {
            jitter % (exponential + 1)
        };
        Duration::from_millis(millis)
    }

    fn next_jitter(&self) -> u64 {
        // SplitMix64 step over a per-client counter: cheap, lock-free, and
        // good enough to decorrelate concurrent retriers.
        let mut z = self
            .inner
            .jitter
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    async fn api_error_from(&self, response: Response, retry_after: Option<u64>) -> Error {
        let status = response.status().as_u16();
        let request_id = extract_request_id(response.headers());
        let body = response.text().await.unwrap_or_default();
        Error::api(build_api_error(status, request_id, retry_after, &body))
    }
}

/// Builder for [`Skailar`].
#[derive(Default)]
pub struct SkailarBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    default_headers: HeaderMap,
    http_client: Option<reqwest::Client>,
}

impl std::fmt::Debug for SkailarBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkailarBuilder")
            .field("api_key", &self.api_key.as_ref().map(|_| "***redacted***"))
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .field("default_headers", &self.default_headers.len())
            .field("http_client", &self.http_client.is_some())
            .finish()
    }
}

impl SkailarBuilder {
    /// Sets the API key, overriding `SKAILAR_API_KEY`.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the base URL, overriding `SKAILAR_BASE_URL` and the default.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the per-attempt request timeout (default 60s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the maximum number of retries (default 2).
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Adds a default header sent on every request.
    ///
    /// An `Authorization` header set here is ignored; the SDK always applies its
    /// own bearer token.
    pub fn default_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            name.as_ref().parse::<HeaderName>(),
            HeaderValue::from_str(value.as_ref()),
        ) {
            self.default_headers.insert(name, value);
        }
        self
    }

    /// Supplies an existing `reqwest::Client` to reuse its connection pool.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingApiKey`] if no key was provided and
    /// `SKAILAR_API_KEY` is unset/empty, or [`Error::InvalidBaseUrl`] if the
    /// base URL cannot be parsed.
    pub fn build(self) -> Result<Skailar, Error> {
        let api_key = self
            .api_key
            .or_else(|| std::env::var("SKAILAR_API_KEY").ok())
            .filter(|k| !k.is_empty())
            .ok_or(Error::MissingApiKey)?;

        let base_url = self
            .base_url
            .or_else(|| std::env::var("SKAILAR_BASE_URL").ok())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
        let base_url = Url::parse(&base_url)?;

        let http = match self.http_client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .map_err(Error::Network)?,
        };

        Ok(Skailar {
            inner: Arc::new(Inner {
                http,
                base_url,
                api_key: ApiKey::new(api_key),
                default_headers: self.default_headers,
                timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
                max_retries: self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
                jitter: AtomicU64::new(0x2545_F491_4F6C_DD1D),
            }),
        })
    }
}

trait RequestBody {
    fn apply(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder;
}

struct NoBody;
impl RequestBody for NoBody {
    fn apply(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
    }
}

struct JsonBody<'a, B: Serialize>(&'a B);
impl<B: Serialize> RequestBody for JsonBody<'_, B> {
    fn apply(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder.json(self.0)
    }
}

async fn decode_json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
    let bytes = response.bytes().await.map_err(Error::Network)?;
    serde_json::from_slice(&bytes).map_err(Error::Decode)
}

fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    for name in ["x-request-id", "x-skailar-request-id", "request-id"] {
        if let Some(value) = headers.get(name) {
            if let Ok(text) = value.to_str() {
                return Some(text.to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> Skailar {
        Skailar::builder()
            .api_key("skl_live_test")
            .base_url("http://localhost:9999")
            .build()
            .unwrap()
    }

    #[test]
    fn missing_key_errors() {
        // Only meaningful when the environment has no key; skip otherwise so the
        // test is robust under any developer/CI environment (and avoids the
        // unsafe `set_var`/`remove_var` forbidden crate-wide).
        if std::env::var("SKAILAR_API_KEY").is_ok() {
            return;
        }
        let err = Skailar::builder().build().unwrap_err();
        assert!(matches!(err, Error::MissingApiKey));
    }

    #[test]
    fn explicit_key_builds() {
        let client = Skailar::builder()
            .api_key("skl_live_explicit")
            .build()
            .unwrap();
        assert_eq!(client.inner.base_url.as_str(), "https://api.skailar.com/");
    }

    #[test]
    fn endpoint_joins_without_double_slash() {
        let client = test_client();
        let url = client.endpoint("v1/models").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9999/v1/models");
    }

    #[test]
    fn endpoint_tolerates_leading_slash() {
        let client = test_client();
        let url = client.endpoint("/v1/models").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9999/v1/models");
    }

    #[test]
    fn backoff_respects_retry_after_cap() {
        let client = test_client();
        let delay = client.backoff_delay(0, Some(120));
        assert_eq!(delay, Duration::from_secs(MAX_RETRY_AFTER_SECS));
    }

    #[test]
    fn backoff_stays_within_exponential_window() {
        let client = test_client();
        for attempt in 0..6 {
            let cap = BACKOFF_CAP_MS.min(BACKOFF_BASE_MS * (1u64 << attempt));
            for _ in 0..50 {
                let delay = client.backoff_delay(attempt, None).as_millis() as u64;
                assert!(delay <= cap, "attempt {attempt}: {delay} > {cap}");
            }
        }
    }

    #[test]
    fn side_effect_not_retried_on_5xx() {
        let client = test_client();
        assert!(!client.should_retry(
            StatusCode::INTERNAL_SERVER_ERROR,
            Idempotency::SideEffect,
            0,
            3
        ));
    }

    #[test]
    fn idempotent_retried_on_5xx() {
        let client = test_client();
        assert!(client.should_retry(StatusCode::BAD_GATEWAY, Idempotency::Idempotent, 0, 3));
    }

    #[test]
    fn rate_limit_retried_for_side_effects() {
        let client = test_client();
        assert!(client.should_retry(StatusCode::TOO_MANY_REQUESTS, Idempotency::SideEffect, 0, 3));
    }

    #[test]
    fn no_retry_when_attempts_exhausted() {
        let client = test_client();
        assert!(!client.should_retry(StatusCode::TOO_MANY_REQUESTS, Idempotency::Idempotent, 2, 3));
    }

    #[test]
    fn client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Skailar>();
    }

    #[test]
    fn parses_retry_after_header() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("30"));
        assert_eq!(parse_retry_after(&headers), Some(30));
    }

    #[test]
    fn extracts_request_id_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("request-id", HeaderValue::from_static("c"));
        headers.insert("x-request-id", HeaderValue::from_static("a"));
        assert_eq!(extract_request_id(&headers).as_deref(), Some("a"));
    }
}
