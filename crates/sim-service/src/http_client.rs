//! Standalone bounded HTTP client helpers for `sim-service` (SIM-001).
//!
//! Keeps `sim-service` self-contained without coupling to commercial product
//! adapter crates.

use std::sync::OnceLock;
use std::time::Duration;

/// Default upper bound (10 MiB) for buffered upstream HTTP responses.
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// Error returned when streaming or decoding a bounded HTTP response body fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedBodyError(pub String);

impl std::fmt::Display for BoundedBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BoundedBodyError {}

/// Shared HTTP client singleton with standard timeouts and connection pooling.
#[must_use]
pub fn shared_client() -> &'static reqwest::Client {
    static SHARED: OnceLock<reqwest::Client> = OnceLock::new();
    SHARED.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("agent-society-simulator/0.1")
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|e| panic!("failed to create simulator HTTP client: {e}"))
    })
}

/// Stream-read an HTTP response body up to `max_bytes`.
///
/// # Errors
/// Returns an error if reading a chunk fails or if the body exceeds `max_bytes`.
pub async fn read_bounded_bytes(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, BoundedBodyError> {
    if let Some(len) = response.content_length() {
        if len > max_bytes as u64 {
            return Err(BoundedBodyError(format!(
                "HTTP response Content-Length {len} exceeds maximum allowed {max_bytes} bytes"
            )));
        }
    }
    let initial_cap = response
        .content_length()
        .and_then(|l| usize::try_from(l).ok())
        .map_or(4096, |l| l.min(max_bytes));
    const MAX_RESPONSE_CHUNKS: usize = 100_000;
    let mut buf = Vec::with_capacity(initial_cap);
    for _ in 0..MAX_RESPONSE_CHUNKS {
        let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| BoundedBodyError(format!("Failed reading HTTP response chunk: {e}")))?
        else {
            return Ok(buf);
        };
        if buf.len().saturating_add(chunk.len()) > max_bytes {
            return Err(BoundedBodyError(format!(
                "HTTP response stream exceeded maximum allowed {max_bytes} bytes"
            )));
        }
        buf.extend_from_slice(&chunk);
    }
    Err(BoundedBodyError(format!(
        "HTTP response stream exceeded maximum allowed {MAX_RESPONSE_CHUNKS} chunks"
    )))
}

/// Stream-read a UTF-8 HTTP response body up to `max_bytes`.
///
/// # Errors
/// Returns an error if the body exceeds `max_bytes`, transport fails, or bytes are invalid UTF-8.
pub async fn read_bounded_text(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<String, BoundedBodyError> {
    let bytes = read_bounded_bytes(response, max_bytes).await?;
    String::from_utf8(bytes)
        .map_err(|e| BoundedBodyError(format!("HTTP response body is not valid UTF-8: {e}")))
}
