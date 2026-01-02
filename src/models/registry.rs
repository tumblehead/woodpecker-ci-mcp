use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Registry information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Registry {
    /// Registry ID
    pub id: i64,
    /// Registry address
    pub address: String,
    /// Registry username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Registry email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Registry creation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegistryCreate {
    /// Registry address
    pub address: String,
    /// Registry username
    pub username: String,
    /// Registry password
    pub password: String,
    /// Registry email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Registry update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RegistryUpdate {
    /// Registry username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Registry password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Registry email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
