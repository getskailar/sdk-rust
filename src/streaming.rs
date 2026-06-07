//! Server-Sent Events parsing and the [`ChatCompletionStream`] type.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::Stream;

use crate::error::{Error, parse_error_fields};
use crate::types::chat::ChatCompletionChunk;

const DONE_SENTINEL: &str = "[DONE]";

/// Splits a byte stream into SSE lines, tolerating `\n`, `\r\n`, and `\r`.
///
/// A trailing lone `\r` is held back across [`push`](Self::push) calls so a
/// `\r\n` straddling a chunk boundary is treated as one terminator rather than
/// an empty line.
#[derive(Default)]
struct LineBuffer {
    buf: String,
    pending_cr: bool,
}

impl LineBuffer {
    fn push(&mut self, text: &str, out: &mut Vec<String>) {
        for ch in text.chars() {
            match ch {
                '\r' => {
                    self.flush_line(out);
                    self.pending_cr = true;
                }
                '\n' => {
                    if self.pending_cr {
                        // Part of a `\r\n` pair; the line was already flushed.
                        self.pending_cr = false;
                    } else {
                        self.flush_line(out);
                    }
                }
                _ => {
                    self.pending_cr = false;
                    self.buf.push(ch);
                }
            }
        }
    }

    fn flush_line(&mut self, out: &mut Vec<String>) {
        out.push(std::mem::take(&mut self.buf));
    }

    fn take_remainder(&mut self) -> Option<String> {
        if self.buf.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buf))
        }
    }
}

/// Extracts the payload of a `data:` line, or `None` for blanks, comments, and
/// other SSE fields.
fn data_payload(line: &str) -> Option<&str> {
    let trimmed = line.trim_end_matches([' ', '\t']);
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return None;
    }
    let rest = trimmed.strip_prefix("data:")?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
}

/// What a single SSE data line decoded to.
#[derive(Debug)]
enum Event {
    Chunk(Box<ChatCompletionChunk>),
    Done,
}

fn decode_event(payload: &str) -> Result<Option<Event>, Error> {
    if payload == DONE_SENTINEL {
        return Ok(Some(Event::Done));
    }
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|_| Error::MalformedStreamEvent(payload.to_owned()))?;

    // An in-band error frame: surface it as an API error rather than a chunk.
    if value.get("error").is_some() {
        let (code, message) = parse_error_fields(&value);
        return Err(Error::api(crate::error::ApiError {
            status: 500,
            code,
            message: message.unwrap_or_else(|| "streaming error".to_owned()),
            request_id: None,
            raw: Some(value),
            retry_after: None,
        }));
    }

    let chunk: ChatCompletionChunk = serde_json::from_value(value)
        .map_err(|_| Error::MalformedStreamEvent(payload.to_owned()))?;
    Ok(Some(Event::Chunk(Box::new(chunk))))
}

type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// An asynchronous stream of [`ChatCompletionChunk`]s.
///
/// Implements [`Stream`], so consume it with
/// [`StreamExt::next`](futures_util::StreamExt::next):
///
/// ```no_run
/// use futures_util::StreamExt;
/// # use skailar::{Skailar, ChatCompletionRequest, ChatMessage};
/// # async fn run(client: Skailar) -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = client.chat().completions().create_stream(
///     ChatCompletionRequest::builder()
///         .model("claude-sonnet-4-6")
///         .message(ChatMessage::user("Count to 5"))
///         .build()?,
/// ).await?;
///
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     if let Some(piece) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
///         print!("{piece}");
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// To cancel early, drop the stream: the underlying HTTP body is dropped with
/// it, which closes the connection. [`close`](Self::close) makes that explicit.
pub struct ChatCompletionStream {
    bytes: Option<ByteStream>,
    lines: LineBuffer,
    queued: std::collections::VecDeque<String>,
    timeout_secs: u64,
    done: bool,
}

impl ChatCompletionStream {
    pub(crate) fn from_response(response: reqwest::Response) -> Self {
        ChatCompletionStream {
            bytes: Some(Box::pin(response.bytes_stream())),
            lines: LineBuffer::default(),
            queued: std::collections::VecDeque::new(),
            timeout_secs: 0,
            done: false,
        }
    }

    /// Explicitly closes the stream, cancelling the in-flight HTTP body.
    ///
    /// Equivalent to dropping the stream; provided for call sites that want the
    /// intent to read clearly.
    pub fn close(mut self) {
        self.bytes = None;
        self.done = true;
    }

    fn next_ready(&mut self) -> Option<Result<ChatCompletionChunk, Error>> {
        while let Some(payload) = self.queued.pop_front() {
            match decode_event(&payload) {
                Ok(Some(Event::Chunk(chunk))) => return Some(Ok(*chunk)),
                Ok(Some(Event::Done)) => {
                    self.done = true;
                    self.bytes = None;
                    return None;
                }
                Ok(None) => continue,
                Err(err) => {
                    self.done = true;
                    self.bytes = None;
                    return Some(Err(err));
                }
            }
        }
        None
    }
}

impl Stream for ChatCompletionStream {
    type Item = Result<ChatCompletionChunk, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if let Some(ready) = this.next_ready() {
                return Poll::Ready(Some(ready));
            }
            if this.done {
                return Poll::Ready(None);
            }

            let Some(bytes) = this.bytes.as_mut() else {
                return Poll::Ready(None);
            };

            match bytes.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Ok(chunk))) => {
                    let text = String::from_utf8_lossy(&chunk);
                    let mut lines = Vec::new();
                    this.lines.push(&text, &mut lines);
                    for line in lines {
                        if let Some(payload) = data_payload(&line) {
                            this.queued.push_back(payload.to_owned());
                        }
                    }
                }
                Poll::Ready(Some(Err(err))) => {
                    this.done = true;
                    this.bytes = None;
                    return Poll::Ready(Some(Err(Error::from_reqwest(err, this.timeout_secs))));
                }
                Poll::Ready(None) => {
                    // Flush any final unterminated line before ending.
                    if let Some(remainder) = this.lines.take_remainder() {
                        if let Some(payload) = data_payload(&remainder) {
                            this.queued.push_back(payload.to_owned());
                        }
                    }
                    this.done = true;
                    this.bytes = None;
                    if let Some(ready) = this.next_ready() {
                        return Poll::Ready(Some(ready));
                    }
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(input: &str) -> Vec<String> {
        let mut buf = LineBuffer::default();
        let mut out = Vec::new();
        buf.push(input, &mut out);
        if let Some(rem) = buf.take_remainder() {
            out.push(rem);
        }
        out
    }

    #[test]
    fn splits_on_lf() {
        assert_eq!(split("a\nb\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn splits_on_crlf() {
        assert_eq!(split("a\r\nb\r\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn splits_on_cr() {
        assert_eq!(split("a\rb\rc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn crlf_split_across_chunks() {
        let mut buf = LineBuffer::default();
        let mut out = Vec::new();
        buf.push("a\r", &mut out);
        buf.push("\nb", &mut out);
        if let Some(rem) = buf.take_remainder() {
            out.push(rem);
        }
        assert_eq!(out, vec!["a", "b"]);
    }

    #[test]
    fn data_payload_strips_prefix_and_space() {
        assert_eq!(data_payload("data: hello"), Some("hello"));
        assert_eq!(data_payload("data:hello"), Some("hello"));
    }

    #[test]
    fn data_payload_ignores_comments_and_blanks() {
        assert_eq!(data_payload(": keep-alive"), None);
        assert_eq!(data_payload(""), None);
        assert_eq!(data_payload("event: message"), None);
    }

    #[test]
    fn detects_done_sentinel() {
        let event = decode_event("[DONE]").unwrap();
        assert!(matches!(event, Some(Event::Done)));
    }

    #[test]
    fn decodes_chunk() {
        let payload = r#"{"id":"1","object":"chat.completion.chunk","created":1,"model":"m","choices":[{"index":0,"delta":{"content":"hi"}}]}"#;
        let event = decode_event(payload).unwrap();
        match event {
            Some(Event::Chunk(chunk)) => {
                assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hi"));
            }
            _ => panic!("expected chunk"),
        }
    }

    #[test]
    fn malformed_json_is_surfaced() {
        match decode_event("{not json") {
            Err(Error::MalformedStreamEvent(_)) => {}
            other => panic!("expected malformed stream event, got {other:?}"),
        }
    }

    #[test]
    fn in_band_error_becomes_api_error() {
        let payload = r#"{"error":{"type":"upstream_error","message":"boom"}}"#;
        match decode_event(payload) {
            Err(Error::Api(api)) => {
                assert_eq!(api.code.as_deref(), Some("upstream_error"));
                assert_eq!(api.message, "boom");
            }
            other => panic!("expected API error, got {other:?}"),
        }
    }
}
