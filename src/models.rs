//! Typed views of the few Woodpecker responses this server inspects itself.
//!
//! Everything else is passed through to the client as the server's own JSON,
//! so new or renamed fields in Woodpecker never get silently dropped. Fields
//! here default when absent so older or newer servers still deserialize.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub number: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub errors: Option<Vec<PipelineError>>,
    #[serde(default)]
    pub workflows: Option<Vec<Workflow>>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineError {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub is_warning: bool,
}

#[derive(Debug, Deserialize)]
pub struct Workflow {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub children: Option<Vec<Step>>,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub exit_code: i64,
    #[serde(default, rename = "type")]
    pub step_type: String,
    #[serde(default)]
    pub started: i64,
    #[serde(default)]
    pub finished: i64,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    #[serde(default)]
    pub line: Option<usize>,
    /// Base64-encoded bytes of the line
    #[serde(default)]
    pub data: Option<String>,
}
