//! Shared HTTP helpers for LLM providers.
//!
//! The Server-Sent-Events plumbing (byte-stream buffering, line splitting,
//! `data: ` stripping, UTF-8 handling, channel sending) is identical across the
//! OpenAI- and Anthropic-style streaming endpoints. Each provider only differs in
//! how it parses one SSE payload line, so that part stays in the provider as a small
//! synchronous closure while the async plumbing lives here.

use futures_util::stream::StreamExt;
use tokio::sync::mpsc::Sender;

/// What a provider's per-line SSE parser wants the pump to do with one `data:` line.
pub(crate) enum SseAction {
    /// Forward these token strings to the receiver, in order.
    Emit(Vec<String>),
    /// Terminal event (e.g. OpenAI `[DONE]`, Anthropic `message_stop`): stop the stream.
    Stop,
    /// Nothing to emit for this line (unknown/keepalive event): keep going.
    Skip,
}

/// Drive an SSE response byte stream to completion, sending parsed tokens on `tx`.
///
/// `parse` receives each `data:` payload (already stripped of the `data: ` prefix)
/// and decides what to emit. Bare `[DONE]` lines and a dropped receiver both stop the
/// stream. Chunk read / UTF-8 / parse problems are logged and skipped, matching the
/// previous per-provider behavior.
pub(crate) async fn pump_sse<F>(response: reqwest::Response, tx: Sender<String>, mut parse: F)
where
    F: FnMut(&str) -> SseAction,
{
    let mut byte_stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = match chunk_result {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("Failed to read response chunk: {}", e);
                return;
            }
        };

        let Ok(text) = String::from_utf8(chunk.to_vec()) else {
            tracing::warn!("Failed to convert response chunk to UTF-8");
            continue;
        };
        buffer.push_str(&text);

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }
            if line == "[DONE]" {
                return;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                match parse(data) {
                    SseAction::Emit(tokens) => {
                        for token in tokens {
                            if let Err(e) = tx.send(token).await {
                                tracing::warn!("Failed to send token: {}", e);
                                return;
                            }
                        }
                    }
                    SseAction::Stop => return,
                    SseAction::Skip => {}
                }
            }
        }
    }
}
