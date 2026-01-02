use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// User information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct User {
    /// User ID
    pub id: i64,
    /// User login name
    pub login: String,
    /// User email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User avatar URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Whether user is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether user is admin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserCreate {
    /// User login name
    pub login: String,
    /// User email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether user is admin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}

/// User update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct UserUpdate {
    /// User email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether user is admin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}

/// User feed item (recent pipeline)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedItem {
    /// Pipeline ID
    pub id: i64,
    /// Repository ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
    /// Pipeline number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    /// Pipeline status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Commit message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Repository name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
}
