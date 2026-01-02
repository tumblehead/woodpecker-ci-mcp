use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Secret information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Secret {
    /// Secret ID
    pub id: i64,
    /// Secret name
    pub name: String,
    /// Events that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Images that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

/// Secret creation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecretCreate {
    /// Secret name
    pub name: String,
    /// Secret value
    pub value: String,
    /// Events that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Images that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

/// Secret update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SecretUpdate {
    /// Secret value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Events that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Images that can use this secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}
