use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Organization information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Organization {
    /// Organization ID
    pub id: i64,
    /// Organization name
    pub name: String,
    /// Whether organization is a user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user: Option<bool>,
}

/// Organization permissions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrgPermissions {
    /// Whether user is member
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<bool>,
    /// Whether user is admin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}
