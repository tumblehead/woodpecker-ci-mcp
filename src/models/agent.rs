use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Agent {
    /// Agent ID
    pub id: i64,
    /// Agent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Agent backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Agent platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Agent capacity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i32>,
    /// Agent version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Whether agent is paused
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_schedule: Option<bool>,
    /// Custom labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_labels: Option<String>,
    /// Last contact time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_contact: Option<i64>,
    /// Created timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    /// Updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<i64>,
}

/// Agent creation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentCreate {
    /// Agent name
    pub name: String,
    /// Whether agent should not be scheduled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_schedule: Option<bool>,
}

/// Agent update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AgentUpdate {
    /// Agent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether agent should not be scheduled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_schedule: Option<bool>,
}

/// Agent task information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentTask {
    /// Task ID
    pub id: i64,
    /// Repository name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Pipeline number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_number: Option<i64>,
    /// Step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Task state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}
