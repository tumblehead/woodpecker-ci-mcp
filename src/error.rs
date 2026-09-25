use rmcp::handler::server::tool::IntoCallToolResult;
use rmcp::model::{CallToolResponse, CallToolResult, ContentBlock, ErrorData as McpError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WoodpeckerError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{request} failed with HTTP {status}: {message}")]
    Api {
        request: String,
        status: u16,
        message: String,
    },

    #[error("{request} returned an unexpected response: {message}")]
    Decode { request: String, message: String },

    #[error("{0}")]
    InvalidInput(String),
}

impl WoodpeckerError {
    /// Build an error from a non-success HTTP response.
    pub fn from_response(request: String, status: u16, body: &str) -> Self {
        let body = body.trim();
        let message = match status {
            401 => "unauthorized: the token is invalid or expired".to_string(),
            403 => "forbidden: the token lacks permission for this resource".to_string(),
            404 if body.is_empty() => "not found".to_string(),
            _ if body.is_empty() => "no details provided by the server".to_string(),
            _ => truncate(body, 1000),
        };
        WoodpeckerError::Api {
            request,
            status,
            message,
        }
    }
}

/// Tool failures are reported as a tool result with `isError: true` so the
/// model sees the message and can correct itself, rather than as a JSON-RPC
/// protocol error (which the MCP spec reserves for malformed requests).
impl IntoCallToolResult for WoodpeckerError {
    fn into_call_tool_result(self) -> std::result::Result<CallToolResponse, McpError> {
        Ok(CallToolResult::error(vec![ContentBlock::text(self.to_string())]).into())
    }
}

/// Truncate to at most `max` bytes without splitting a UTF-8 character.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}... ({} bytes total)", &s[..end], s.len())
}

pub type Result<T> = std::result::Result<T, WoodpeckerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_respects_char_boundaries() {
        // 'ø' is two bytes; cutting at byte 3 would split it
        let s = "abø".repeat(10);
        let out = truncate(&s, 3);
        assert!(out.starts_with("ab..."));
    }

    #[test]
    fn empty_bodies_get_a_readable_message() {
        let err = WoodpeckerError::from_response("GET /api/repos/1".into(), 404, "");
        assert_eq!(
            err.to_string(),
            "GET /api/repos/1 failed with HTTP 404: not found"
        );
    }
}
