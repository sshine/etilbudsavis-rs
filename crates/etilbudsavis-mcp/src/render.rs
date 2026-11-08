//! Rendering API values into MCP results.

use rmcp::model::{CallToolResult, ContentBlock};
use serde::Serialize;
use serde_json::{Value, json};

/// How much one tool may return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputLimits {
    /// Rows a listing returns before it truncates.
    pub max_items: usize,
    /// Bytes a result may reach before it sheds rows.
    pub max_bytes: usize,
}

impl Default for OutputLimits {
    fn default() -> Self {
        Self {
            max_items: 100,
            max_bytes: 100_000,
        }
    }
}

/// A successful result carrying one value.
pub fn json<T: Serialize>(value: &T) -> CallToolResult {
    match serde_json::to_value(value) {
        Ok(value) => CallToolResult::structured(value),
        Err(error) => unserializable(&error),
    }
}

/// A successful result carrying a JSON value, truncated to the byte budget.
pub fn limited(value: Value, max_bytes: usize) -> CallToolResult {
    let rendered = value.to_string();
    if rendered.len() <= max_bytes {
        return CallToolResult::structured(value);
    }

    let truncated = match value {
        Value::Object(mut map) => {
            map.insert("truncated".to_owned(), json!(true));
            Value::Object(map)
        }
        other => json!({ "truncated": true, "value": other }),
    };
    CallToolResult::structured(truncated)
}

/// A listing with enough of a header to tell a short answer from a truncated one.
pub fn page<T: Serialize>(items: &[T], total: usize, limits: OutputLimits) -> CallToolResult {
    let mut shown = total.min(limits.max_items);

    loop {
        let value = json!({
            "total": total,
            "returned": shown,
            "truncated": shown < total,
            "items": &items[..shown],
        });
        if shown == 0 || value.to_string().len() <= limits.max_bytes {
            return CallToolResult::structured(value);
        }
        shown /= 2;
    }
}

/// A caller's own argument was wrong.
pub fn rejected(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(text)])
}

fn unserializable(error: &serde_json::Error) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!(
        "eTilbudsavis returned a value this server could not encode as JSON: {error}"
    ))])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn a_long_listing_reports_what_it_did_not_return() {
        let items: Vec<String> = (0..250).map(|i| format!("item-{i}")).collect();
        let result = page(
            &items,
            items.len(),
            OutputLimits {
                max_items: 10,
                ..OutputLimits::default()
            },
        );
        let body = result.structured_content.expect("structured");
        assert_eq!(body["total"], json!(250));
        assert_eq!(body["returned"], json!(10));
        assert_eq!(body["truncated"], json!(true));
    }

    #[test]
    fn an_oversized_value_is_marked_but_stays_json() {
        let result = limited(json!({ "html": "x".repeat(200) }), 50);
        let body = result.structured_content.expect("structured");
        assert_eq!(body["truncated"], json!(true));
    }
}
