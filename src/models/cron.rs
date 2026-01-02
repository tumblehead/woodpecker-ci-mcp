use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Cron job information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronJob {
    /// Cron job ID
    pub id: i64,
    /// Cron job name
    pub name: String,
    /// Repository ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
    /// Creator ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<i64>,
    /// Next execution time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_exec: Option<i64>,
    /// Cron schedule expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    /// Branch to build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Created timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
}

/// Cron job creation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronJobCreate {
    /// Cron job name
    pub name: String,
    /// Cron schedule expression
    pub schedule: String,
    /// Branch to build
    pub branch: String,
}

/// Cron job update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CronJobUpdate {
    /// Cron job name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Cron schedule expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    /// Branch to build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}
