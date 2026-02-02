use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pipeline {
    /// Pipeline ID
    pub id: i64,
    /// Pipeline number
    pub number: i64,
    /// Parent pipeline number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<i64>,
    /// Event that triggered the pipeline
    pub event: String,
    /// Pipeline status
    pub status: String,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Time when pipeline was enqueued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enqueued_at: Option<i64>,
    /// Time when pipeline was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Time when pipeline was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    /// Time when pipeline started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// Time when pipeline finished
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    /// Deployment target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy_to: Option<String>,
    /// Commit SHA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Git ref
    #[serde(rename = "ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    /// Git refspec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refspec: Option<String>,
    /// Pipeline title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Commit message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Commit author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Author avatar URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_avatar: Option<String>,
    /// Author email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
    /// Link URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    /// Pipeline workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflows: Option<Vec<Workflow>>,
    /// Whether the pipeline is reviewed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<i64>,
    /// Reviewer username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
}

/// Workflow within a pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Workflow {
    /// Workflow ID
    pub id: i64,
    /// Pipeline ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<i64>,
    /// Workflow name
    pub name: String,
    /// Workflow state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Start time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Steps in this workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Step>>,
}

/// Step within a workflow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step {
    /// Step ID
    pub id: i64,
    /// Step UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Pipeline ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<i64>,
    /// Step name
    pub name: String,
    /// Step state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Exit code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Start time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Step type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub step_type: Option<String>,
}

/// Request to create a new pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineCreate {
    /// Branch to build
    pub branch: String,
    /// Variables (defaults to empty object)
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

/// Pipeline filter parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PipelineFilter {
    /// Filter by branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Filter by event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    /// Items per page
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "perPage")]
    pub per_page: Option<i32>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    /// Log entry ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Step ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<i64>,
    /// Log line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    /// Log data (base64 encoded bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Log type (0=stdout, 1=stderr, 2=exit_code, 3=metadata, 4=progress)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub log_type: Option<i32>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineConfig {
    /// Configuration hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Configuration name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Configuration data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}
