use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Pagination parameters for list endpoints
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    /// Items per page (default: 50)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "perPage")]
    pub per_page: Option<i32>,
}

/// Server version information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionInfo {
    /// Version string
    pub version: String,
    /// Source code URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthInfo {
    /// Health status
    pub status: String,
}
