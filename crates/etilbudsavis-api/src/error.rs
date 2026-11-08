//! Errors returned by the client.

use reqwest::{Method, StatusCode};

/// The result type used throughout this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// What went wrong.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The transport failed before a response arrived.
    #[error("could not reach eTilbudsavis: {0}")]
    Transport(#[from] reqwest::Error),

    /// The service answered with a non-success status.
    #[error("eTilbudsavis returned HTTP {status} for {method} {path}: {body}")]
    Http {
        /// The HTTP status.
        status: StatusCode,
        /// The request method.
        method: Method,
        /// The request path.
        path: String,
        /// A truncated response body.
        body: String,
    },

    /// The service answered with a structured API error.
    #[error("eTilbudsavis API error {status}: {message}")]
    Api {
        /// The status from the error body, or the HTTP status when it was absent.
        status: u16,
        /// The human-readable message.
        message: String,
    },

    /// A response body did not match the expected type.
    #[error("could not decode {expected}: {body}")]
    Decode {
        /// The expected Rust type.
        expected: &'static str,
        /// A truncated response body.
        body: String,
        /// The underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// A URL was not valid.
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// The caller supplied an invalid value.
    #[error("invalid input: {0}")]
    Invalid(String),

    /// A local file operation failed.
    #[error("local I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A local JSON file was corrupt.
    #[error("local JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Whether the failure means the account token is missing or expired.
    pub fn is_auth(&self) -> bool {
        match self {
            Self::Api { status, .. } => *status == 401 || *status == 403,
            Self::Http { status, .. } => {
                *status == StatusCode::UNAUTHORIZED || *status == StatusCode::FORBIDDEN
            }
            _ => false,
        }
    }

    pub(crate) fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }
}

pub(crate) fn truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}
