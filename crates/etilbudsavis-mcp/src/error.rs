//! Turning an API failure into a tool result a caller can act on.

use etilbudsavis_api::Error;
use rmcp::model::{CallToolResult, ContentBlock};

/// Report a failed call.
pub fn failed(error: &Error) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message(error))])
}

fn message(error: &Error) -> String {
    if error.is_auth() {
        return "This tool needs an account token. Run `etilbudsavis auth anonymous` or \
                `etilbudsavis auth login you@example.com`, or start the server with \
                --allow-writes so it can create an anonymous account."
            .to_owned();
    }

    match error {
        Error::Decode { .. } => "eTilbudsavis answered with a body this client could not decode. \
                                 The service may be serving an error page instead of JSON."
            .to_owned(),
        Error::Transport(_) => format!("Could not reach eTilbudsavis: {error}"),
        other => other.to_string(),
    }
}
