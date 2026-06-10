//! Error types for ai-lib.

use thiserror::Error;

/// All errors that can come out of an `ai-lib` call.
#[derive(Debug, Error)]
pub enum Error {
    /// The HTTP client failed (network error, DNS, timeout, etc).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The provider returned a non-2xx status. The body is included for
    /// debugging; do not show it to end users as it may contain sensitive
    /// details.
    #[error("provider returned {status}: {body}")]
    Provider { status: u16, body: String },

    /// The provider's response could not be parsed.
    #[error("failed to parse provider response: {0}")]
    Parse(String),

    /// The caller passed an invalid request (e.g. empty messages).
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}
