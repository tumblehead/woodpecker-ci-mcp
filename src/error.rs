use rmcp::model::{ErrorCode, ErrorData as McpError};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WoodpeckerError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: Invalid or expired token")]
    Unauthorized,

    #[error("Forbidden: Insufficient permissions")]
    Forbidden,

    #[error("{0}")]
    Other(String),
}

impl WoodpeckerError {
    /// Convert to MCP error for tool responses
    pub fn into_mcp_error(self) -> McpError {
        let code = match &self {
            WoodpeckerError::NotFound(_) => ErrorCode::INVALID_PARAMS,
            WoodpeckerError::Unauthorized | WoodpeckerError::Forbidden => ErrorCode::INVALID_REQUEST,
            _ => ErrorCode::INTERNAL_ERROR,
        };

        McpError {
            code,
            message: Cow::from(self.to_string()),
            data: None,
        }
    }

    /// Create from HTTP status code and response body
    pub fn from_response(status: u16, body: String) -> Self {
        match status {
            401 => WoodpeckerError::Unauthorized,
            403 => WoodpeckerError::Forbidden,
            404 => WoodpeckerError::NotFound(body),
            500 => WoodpeckerError::ApiError {
                status,
                message: if body.is_empty() {
                    "Server error (500): No details provided. Check server logs, ensure the branch exists, and verify the repository has a .woodpecker.yml file.".to_string()
                } else {
                    format!("Server error (500): {}", body)
                },
            },
            _ => WoodpeckerError::ApiError {
                status,
                message: body,
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, WoodpeckerError>;
