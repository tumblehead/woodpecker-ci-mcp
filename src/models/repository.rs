use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Repository {
    /// Repository ID
    pub id: i64,
    /// Forge ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forge_id: Option<i64>,
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub name: String,
    /// Full repository name (owner/name)
    pub full_name: String,
    /// Avatar URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Repository link URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    /// Clone URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_url: Option<String>,
    /// Default branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// Source control type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scm: Option<String>,
    /// Pipeline timeout in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    /// Repository visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Whether repository is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// Whether repository is trusted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trusted: Option<bool>,
    /// Whether repository is gated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_gated: Option<bool>,
    /// Whether to allow pull requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_pull: Option<bool>,
    /// Pipeline configuration file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,
    /// Whether repository is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Repository update request
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryUpdate {
    /// Pipeline timeout in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    /// Repository visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Whether repository is trusted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trusted: Option<bool>,
    /// Whether repository is gated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_gated: Option<bool>,
    /// Whether to allow pull requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_pull: Option<bool>,
    /// Pipeline configuration file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,
    /// Cancel previous pipelines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_previous_pipeline_events: Option<Vec<String>>,
    /// Netrc only trusted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub netrc_only_trusted: Option<bool>,
}

/// Repository activation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryActivate {
    /// Forge remote ID of the repository
    pub forge_remote_id: String,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Branch {
    /// Branch name
    pub name: String,
}

/// Pull request information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PullRequest {
    /// Pull request number
    pub number: i64,
    /// Pull request title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Source branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,
    /// Target branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_branch: Option<String>,
}
