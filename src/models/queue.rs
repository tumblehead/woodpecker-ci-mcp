use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Queue information from /api/queue/info
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueInfo {
    /// Whether queue is paused
    #[serde(default)]
    pub paused: bool,
    /// Pending tasks
    #[serde(default)]
    pub pending: Vec<QueueTask>,
    /// Running tasks
    #[serde(default)]
    pub running: Vec<QueueTask>,
    /// Tasks waiting on dependencies
    #[serde(default)]
    pub waiting_on_deps: Vec<QueueTask>,
    /// Queue stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<QueueStats>,
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueStats {
    /// Worker count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_count: Option<i64>,
    /// Pending count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_count: Option<i64>,
    /// Running count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_count: Option<i64>,
    /// Waiting on deps count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on_deps_count: Option<i64>,
}

/// Queued task
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueTask {
    /// Task ID. A string, not a number: Woodpecker's `model.Task.ID` is a Go
    /// string and serialises as `"id": "5362"`. Typing this as i64 made
    /// get_queue_info fail to deserialise whenever the queue was non-empty.
    pub id: String,
    /// Pipeline ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<i64>,
    /// Repository name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Step name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Task status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    /// Dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    /// Run on agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_on: Option<Vec<String>>,
    /// Agent ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: the queue endpoint returns task IDs as strings, and typing
    /// `id` as i64 made every get_queue_info call fail with "invalid type:
    /// string ... expected i64" as soon as anything was queued or running.
    /// Payload trimmed from a live 3.18.0 server.
    #[test]
    fn deserializes_string_task_id() {
        let body = r#"{
            "pending": [],
            "waiting_on_deps": [],
            "running": [
                {
                    "id": "5362",
                    "pid": 1,
                    "name": "t1",
                    "labels": {"repo": "Magzor2k/TumbleVerse", "org-id": "2"}
                }
            ]
        }"#;

        let info: QueueInfo = serde_json::from_str(body).expect("queue payload should deserialize");
        assert_eq!(info.running.len(), 1);
        assert_eq!(info.running[0].id, "5362");
        assert_eq!(info.running[0].name.as_deref(), Some("t1"));
    }
}
