# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] - 2026-06-07

Initial pre-release. Async-only client for the Skailar API, built on `reqwest`
and `tokio`, with an OpenAI-compatible wire surface.

### Added

- `Skailar` client with a builder (`api_key`, `base_url`, `timeout`,
  `max_retries`, `default_header`, `http_client`) and `Skailar::new()` reading
  `SKAILAR_API_KEY` / `SKAILAR_BASE_URL` from the environment.
- Chat completions (`client.chat().completions().create(...)`), JSON and SSE
  streaming via `ChatCompletionStream`.
- Model discovery (`client.models().list()`, `client.models().retrieve(id)`).
- Image generation (`client.images().generate(...)`).
- Audio transcription (`client.audio().transcriptions().create(...)`) and speech
  synthesis (`client.audio().speech().create(...)`, returns an MP3 byte stream).
- Storage uploads (`client.uploads().images().create(...)`,
  `client.uploads().files().create(...)`).
- Key verification (`client.ping()`).
- Manual builders for every request type and convenience constructors on
  `ChatMessage` (`user`, `system`, `assistant`, `tool`).
- `models` module of known model-ID constants.
- Error hierarchy (`Error` + `ApiError`) with status helper predicates.

### Fixed

Shipped already-corrected for bugs fixed in the TypeScript SDK across 0.0.1–0.0.5:

- Retries do not leak background tasks or abort registrations when a caller
  cancels a request (cancellation is cooperative via `Drop`, not a detached
  listener).
- Internal timeouts are reported as `Error::Timeout { timeout_secs }`, distinct
  from `Error::Network` for other transport failures.
- Early exit from a stream (dropping `ChatCompletionStream`) cancels the
  in-flight HTTP body, closing the connection instead of leaking it.
- The `Authorization` header cannot be overridden by `default_header` or
  per-call headers; conflicting keys are dropped case-insensitively before the
  bearer token is applied.
- Side-effecting `POST` requests (chat completions, image generation, speech,
  transcription, uploads) are never retried on `5xx` to avoid double billing.
  Only idempotent `GET` requests are retried on `5xx`.
- `Retry-After` is capped at 60 seconds; the uncapped server value is still
  exposed on `ApiError::retry_after`.
- The SSE parser accepts all three line terminators (`\n`, `\r\n`, `\r`).

[Unreleased]: https://github.com/getskailar/sdk-rust/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/getskailar/sdk-rust/releases/tag/v0.0.1
