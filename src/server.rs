use reqwest::Method;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::{WoodpeckerClient, encode_segment};
use crate::error::WoodpeckerError;
use crate::logs::{self, LogWindow};
use crate::models::{LogEntry, Pipeline};

type ToolResult = Result<CallToolResult, WoodpeckerError>;

/// Fields that only waste context for a model (image URLs).
const NOISE: &[&str] = &["author_avatar", "avatar_url"];

const INSTRUCTIONS: &str = "Woodpecker CI MCP server. Most tools take a numeric repo_id: \
get it from lookup_repo (by 'owner/name') or list_repos. Pipelines are addressed by their \
per-repo pipeline_number. To read a failing step's output, call list_pipeline_steps to find \
the step_id, then get_step_logs (which returns the tail of long logs by default).";

/// MCP Server for Woodpecker CI
#[derive(Clone)]
pub struct WoodpeckerMcpServer {
    client: Arc<WoodpeckerClient>,
    tool_router: ToolRouter<Self>,
}

fn text(s: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(s)])
}

/// Return the server's JSON to the model, minus fields listed in `drop`.
fn json_result(mut value: Value, drop: &[&str]) -> ToolResult {
    strip_keys(&mut value, drop);
    Ok(text(value.to_string()))
}

fn strip_keys(value: &mut Value, keys: &[&str]) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| !keys.contains(&k.as_str()));
            map.values_mut().for_each(|v| strip_keys(v, keys));
        }
        Value::Array(items) => items.iter_mut().for_each(|v| strip_keys(v, keys)),
        _ => {}
    }
}

/// Query string pairs from optional values, skipping the unset ones.
fn query<const N: usize>(
    pairs: [(&'static str, Option<String>); N],
) -> Vec<(&'static str, String)> {
    pairs
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect()
}

/// Endpoint prefix for secrets at global, org or repo level.
fn secrets_path(repo_id: Option<i64>, org_id: Option<i64>) -> Result<String, WoodpeckerError> {
    match (repo_id, org_id) {
        (Some(_), Some(_)) => Err(WoodpeckerError::InvalidInput(
            "Pass either repo_id or org_id, not both (omit both for global secrets)".into(),
        )),
        (Some(repo_id), None) => Ok(format!("/repos/{repo_id}/secrets")),
        (None, Some(org_id)) => Ok(format!("/orgs/{org_id}/secrets")),
        (None, None) => Ok("/secrets".into()),
    }
}

const FILTERED: &str = "Woodpecker created no pipeline: it was filtered out. Either no pipeline \
config was found on the branch, the commit message contains [skip ci], or no workflow's `when` \
conditions match this event (a manually triggered pipeline has event 'manual').";

// Request types for tools

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListReposRequest {
    #[schemars(description = "Include inactive repositories (default: only active ones)")]
    pub all: Option<bool>,
    #[schemars(description = "Only return repositories whose name contains this string")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RepoRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupRepoRequest {
    #[schemars(description = "Full repository name in format 'owner/repo'")]
    pub full_name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPipelinesRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Filter by branch name")]
    pub branch: Option<String>,
    #[schemars(
        description = "Filter by event, comma separated: push, pull_request, pull_request_closed, pull_request_metadata, tag, release, deployment, cron, manual"
    )]
    pub event: Option<String>,
    #[schemars(
        description = "Filter by status: pending, running, success, failure, killed, error, blocked, declined, skipped, canceled"
    )]
    pub status: Option<String>,
    #[schemars(
        description = "Only pipelines whose git ref contains this string (e.g. 'refs/tags/v1')"
    )]
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    #[schemars(description = "Only pipelines created before this RFC 3339 timestamp")]
    pub before: Option<String>,
    #[schemars(description = "Only pipelines created after this RFC 3339 timestamp")]
    pub after: Option<String>,
    #[schemars(description = "Page number (1-based)")]
    pub page: Option<u32>,
    #[schemars(description = "Items per page (default: 50)")]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PipelineRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Pipeline number")]
    pub pipeline_number: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePipelineRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Branch name to build")]
    pub branch: String,
    #[schemars(description = "Optional pipeline variables as key-value pairs")]
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetStepLogsRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Pipeline number")]
    pub pipeline_number: i64,
    #[schemars(description = "Step ID to get logs for (use list_pipeline_steps to find step IDs)")]
    pub step_id: i64,
    #[schemars(
        description = "Number of lines to return: from the start if 'head' is true, otherwise from the end. Ignored if offset or limit is set."
    )]
    pub lines: Option<usize>,
    #[schemars(description = "With 'lines': take lines from the start instead of the end")]
    pub head: Option<bool>,
    #[schemars(
        description = "Zero-based line offset for pagination. Setting offset or limit enables pagination mode (ignores 'lines' and 'head')."
    )]
    pub offset: Option<usize>,
    #[schemars(description = "Maximum lines per page in pagination mode (default: 100)")]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSecretsRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(
        description = "Organization ID (for org secrets). Omit both IDs for global secrets."
    )]
    pub org_id: Option<i64>,
    #[schemars(description = "Page number (1-based)")]
    pub page: Option<u32>,
    #[schemars(description = "Items per page (default: 50)")]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSecretRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(
        description = "Organization ID (for org secrets). Omit both IDs for a global secret (admin only)."
    )]
    pub org_id: Option<i64>,
    #[schemars(description = "Secret name")]
    pub name: String,
    #[schemars(description = "Secret value")]
    pub value: String,
    #[schemars(
        description = "Events whose pipelines may use this secret (at least one): push, pull_request, pull_request_closed, pull_request_metadata, tag, release, deployment, cron, manual"
    )]
    pub events: Vec<String>,
    #[schemars(description = "Restrict the secret to these plugin images")]
    pub images: Option<Vec<String>>,
    #[schemars(description = "Free-text note describing the secret")]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteSecretRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(
        description = "Organization ID (for org secrets). Omit both IDs for a global secret (admin only)."
    )]
    pub org_id: Option<i64>,
    #[schemars(description = "Secret name")]
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UserFeedRequest {
    #[schemars(description = "Only return the latest pipeline of each repository")]
    pub latest: Option<bool>,
}

#[tool_router]
impl WoodpeckerMcpServer {
    pub fn new(client: Arc<WoodpeckerClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    async fn get_json(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value, WoodpeckerError> {
        self.client
            .send(self.client.api(Method::GET, path).query(query))
            .await?
            .json()
    }

    /// POST a pipeline action that returns the resulting pipeline (or 204 when filtered).
    async fn pipeline_action(&self, path: String) -> ToolResult {
        let reply = self
            .client
            .send(self.client.api(Method::POST, &path))
            .await?;
        match reply.json_opt()? {
            Some(pipeline) => json_result(pipeline, NOISE),
            None => Ok(text(FILTERED)),
        }
    }

    // ===== System Tools =====

    #[tool(
        description = "Get Woodpecker CI server version and build information",
        annotations(read_only_hint = true)
    )]
    async fn get_server_version(&self) -> ToolResult {
        let reply = self
            .client
            .send(self.client.root(Method::GET, "/version"))
            .await?;
        json_result(reply.json()?, &[])
    }

    #[tool(
        description = "Check if the Woodpecker CI server is healthy and responding",
        annotations(read_only_hint = true)
    )]
    async fn health_check(&self) -> ToolResult {
        self.client
            .send(self.client.root(Method::GET, "/healthz"))
            .await?;
        Ok(text(r#"{"status":"healthy"}"#))
    }

    #[tool(
        description = "Get pending, running and waiting tasks in the pipeline queue and agent worker counts (admin only)",
        annotations(read_only_hint = true)
    )]
    async fn get_queue_info(&self) -> ToolResult {
        json_result(self.get_json("/queue/info", &[]).await?, &[])
    }

    // ===== Repository Tools =====

    #[tool(
        description = "List repositories the authenticated user can access",
        annotations(read_only_hint = true)
    )]
    async fn list_repos(&self, Parameters(req): Parameters<ListReposRequest>) -> ToolResult {
        let q = query([("all", req.all.map(|b| b.to_string())), ("name", req.name)]);
        json_result(self.get_json("/user/repos", &q).await?, NOISE)
    }

    #[tool(
        description = "Get detailed information about a specific repository by its ID",
        annotations(read_only_hint = true)
    )]
    async fn get_repo(&self, Parameters(req): Parameters<RepoRequest>) -> ToolResult {
        let path = format!("/repos/{}", req.repo_id);
        json_result(self.get_json(&path, &[]).await?, NOISE)
    }

    #[tool(
        description = "Find a repository by its full name (owner/repo format); returns its numeric ID among other details",
        annotations(read_only_hint = true)
    )]
    async fn lookup_repo(&self, Parameters(req): Parameters<LookupRepoRequest>) -> ToolResult {
        let full_name = req
            .full_name
            .trim()
            .trim_matches('/')
            .split('/')
            .map(encode_segment)
            .collect::<Vec<_>>()
            .join("/");
        let path = format!("/repos/lookup/{full_name}");
        json_result(self.get_json(&path, &[]).await?, NOISE)
    }

    // ===== Pipeline Tools =====

    #[tool(
        description = "List pipelines for a repository, newest first, with optional filters",
        annotations(read_only_hint = true)
    )]
    async fn list_pipelines(
        &self,
        Parameters(req): Parameters<ListPipelinesRequest>,
    ) -> ToolResult {
        let q = query([
            ("branch", req.branch),
            ("event", req.event),
            ("status", req.status),
            ("ref", req.git_ref),
            ("before", req.before),
            ("after", req.after),
            ("page", req.page.map(|n| n.to_string())),
            ("perPage", req.per_page.map(|n| n.to_string())),
        ]);
        let path = format!("/repos/{}/pipelines", req.repo_id);
        let drop: Vec<&str> = NOISE.iter().copied().chain(["changed_files"]).collect();
        json_result(self.get_json(&path, &q).await?, &drop)
    }

    #[tool(
        description = "Get detailed information about a specific pipeline, including its workflows, steps and errors",
        annotations(read_only_hint = true)
    )]
    async fn get_pipeline(&self, Parameters(req): Parameters<PipelineRequest>) -> ToolResult {
        let path = format!("/repos/{}/pipelines/{}", req.repo_id, req.pipeline_number);
        json_result(self.get_json(&path, &[]).await?, NOISE)
    }

    #[tool(
        description = "Trigger a new pipeline (event 'manual') on the head of a branch",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false
        )
    )]
    async fn create_pipeline(
        &self,
        Parameters(req): Parameters<CreatePipelineRequest>,
    ) -> ToolResult {
        let body = json!({
            "branch": req.branch,
            "variables": req.variables.unwrap_or_default(),
        });
        let path = format!("/repos/{}/pipelines", req.repo_id);
        let result = self
            .client
            .send(self.client.api(Method::POST, &path).json(&body))
            .await;

        let reply = match result {
            // Woodpecker answers 500 without a body when it cannot resolve the branch head
            Err(WoodpeckerError::Api {
                request,
                status: 500,
                message,
            }) => {
                return Err(WoodpeckerError::Api {
                    request,
                    status: 500,
                    message: format!("{message}. Check that branch '{}' exists.", req.branch),
                });
            }
            other => other?,
        };
        match reply.json_opt()? {
            Some(pipeline) => json_result(pipeline, NOISE),
            None => Ok(text(FILTERED)),
        }
    }

    #[tool(
        description = "Restart a finished pipeline; creates a new pipeline with the same commit and config",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false
        )
    )]
    async fn restart_pipeline(&self, Parameters(req): Parameters<PipelineRequest>) -> ToolResult {
        self.pipeline_action(format!(
            "/repos/{}/pipelines/{}",
            req.repo_id, req.pipeline_number
        ))
        .await
    }

    #[tool(
        description = "Cancel a pending or running pipeline",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true
        )
    )]
    async fn cancel_pipeline(&self, Parameters(req): Parameters<PipelineRequest>) -> ToolResult {
        let path = format!(
            "/repos/{}/pipelines/{}/cancel",
            req.repo_id, req.pipeline_number
        );
        self.client
            .send(self.client.api(Method::POST, &path))
            .await?;
        Ok(text(format!("Pipeline {} cancelled", req.pipeline_number)))
    }

    #[tool(
        description = "Approve a pipeline that is blocked waiting for manual approval",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn approve_pipeline(&self, Parameters(req): Parameters<PipelineRequest>) -> ToolResult {
        self.pipeline_action(format!(
            "/repos/{}/pipelines/{}/approve",
            req.repo_id, req.pipeline_number
        ))
        .await
    }

    #[tool(
        description = "Decline a pipeline that is blocked waiting for manual approval",
        annotations(read_only_hint = false, destructive_hint = true)
    )]
    async fn decline_pipeline(&self, Parameters(req): Parameters<PipelineRequest>) -> ToolResult {
        self.pipeline_action(format!(
            "/repos/{}/pipelines/{}/decline",
            req.repo_id, req.pipeline_number
        ))
        .await
    }

    // ===== Log Tools =====

    #[tool(
        description = "List all steps in a pipeline with their IDs, states, exit codes, durations and errors - use this to find the step_id for get_step_logs",
        annotations(read_only_hint = true)
    )]
    async fn list_pipeline_steps(
        &self,
        Parameters(req): Parameters<PipelineRequest>,
    ) -> ToolResult {
        let path = format!("/repos/{}/pipelines/{}", req.repo_id, req.pipeline_number);
        let reply = self
            .client
            .send(self.client.api(Method::GET, &path))
            .await?;
        Ok(text(render_steps(&reply.json()?)))
    }

    #[tool(
        description = "Get the output of a pipeline step, with line numbers. Returns the whole log, or the last 1000 lines when it is longer. Use lines+head for head/tail, or offset+limit to page through.",
        annotations(read_only_hint = true)
    )]
    async fn get_step_logs(&self, Parameters(req): Parameters<GetStepLogsRequest>) -> ToolResult {
        let path = format!(
            "/repos/{}/logs/{}/{}",
            req.repo_id, req.pipeline_number, req.step_id
        );
        let reply = self
            .client
            .send(self.client.api(Method::GET, &path))
            .await?;
        let entries: Vec<LogEntry> = reply.json_opt()?.unwrap_or_default();
        let lines = logs::decode(&entries);

        if lines.is_empty() {
            return Ok(text(
                "Step has no log output. It may have been skipped, not started yet, or its logs were deleted.",
            ));
        }

        let window = LogWindow {
            lines: req.lines,
            head: req.head.unwrap_or(false),
            offset: req.offset,
            limit: req.limit,
        };
        Ok(text(logs::render(&lines, window)))
    }

    // ===== Secret Tools =====

    #[tool(
        description = "List secrets (names and settings, never values) at global, org, or repo level",
        annotations(read_only_hint = true)
    )]
    async fn list_secrets(&self, Parameters(req): Parameters<ListSecretsRequest>) -> ToolResult {
        let q = query([
            ("page", req.page.map(|n| n.to_string())),
            ("perPage", req.per_page.map(|n| n.to_string())),
        ]);
        let path = secrets_path(req.repo_id, req.org_id)?;
        json_result(self.get_json(&path, &q).await?, &[])
    }

    #[tool(
        description = "Create a new secret at global, org, or repo level",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false
        )
    )]
    async fn create_secret(&self, Parameters(req): Parameters<CreateSecretRequest>) -> ToolResult {
        let path = secrets_path(req.repo_id, req.org_id)?;
        let body = json!({
            "name": req.name,
            "value": req.value,
            "events": req.events,
            "images": req.images.unwrap_or_default(),
            "note": req.note.unwrap_or_default(),
        });
        let reply = self
            .client
            .send(self.client.api(Method::POST, &path).json(&body))
            .await?;
        json_result(reply.json()?, &["value"])
    }

    #[tool(
        description = "Delete a secret at global, org, or repo level",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true
        )
    )]
    async fn delete_secret(&self, Parameters(req): Parameters<DeleteSecretRequest>) -> ToolResult {
        let path = format!(
            "{}/{}",
            secrets_path(req.repo_id, req.org_id)?,
            encode_segment(&req.name)
        );
        self.client
            .send(self.client.api(Method::DELETE, &path))
            .await?;
        Ok(text(format!("Secret '{}' deleted", req.name)))
    }

    // ===== User Tools =====

    #[tool(
        description = "Get information about the currently authenticated user",
        annotations(read_only_hint = true)
    )]
    async fn get_current_user(&self) -> ToolResult {
        json_result(self.get_json("/user", &[]).await?, NOISE)
    }

    #[tool(
        description = "Get recent pipelines across all repositories of the authenticated user",
        annotations(read_only_hint = true)
    )]
    async fn get_user_feed(&self, Parameters(req): Parameters<UserFeedRequest>) -> ToolResult {
        let q = query([("latest", req.latest.map(|b| b.to_string()))]);
        json_result(self.get_json("/user/feed", &q).await?, NOISE)
    }
}

/// Human-readable step overview for list_pipeline_steps.
fn render_steps(pipeline: &Pipeline) -> String {
    let mut out = format!("Pipeline #{} ({})\n", pipeline.number, pipeline.status);

    for err in pipeline.errors.iter().flatten() {
        let kind = if err.is_warning { "Warning" } else { "Error" };
        out.push_str(&format!("{kind}: {}\n", err.message));
    }

    let workflows = pipeline.workflows.as_deref().unwrap_or_default();
    if workflows.is_empty() {
        out.push_str("\nNo workflows found in this pipeline.\n");
    }

    for workflow in workflows {
        out.push_str(&format!(
            "\nWorkflow: {} ({})\n",
            workflow.name, workflow.state
        ));
        if !workflow.error.is_empty() {
            out.push_str(&format!("  Error: {}\n", workflow.error));
        }
        for step in workflow.children.iter().flatten() {
            out.push_str(&format!(
                "  - step_id {} | {} | {}",
                step.id, step.name, step.state
            ));
            if !step.step_type.is_empty() {
                out.push_str(&format!(" | type {}", step.step_type));
            }
            if matches!(step.state.as_str(), "success" | "failure" | "killed") {
                out.push_str(&format!(" | exit {}", step.exit_code));
            }
            if step.started > 0 && step.finished >= step.started {
                out.push_str(&format!(" | {}s", step.finished - step.started));
            }
            if !step.error.is_empty() {
                out.push_str(&format!(" | error: {}", step.error));
            }
            out.push('\n');
        }
    }

    out.truncate(out.trim_end().len());
    out
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WoodpeckerMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(INSTRUCTIONS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_noise_recursively() {
        let mut v = json!([{"id": 1, "author_avatar": "x", "workflows": [{"avatar_url": "y", "name": "w"}]}]);
        strip_keys(&mut v, NOISE);
        assert_eq!(v, json!([{"id": 1, "workflows": [{"name": "w"}]}]));
    }

    #[test]
    fn secrets_path_rejects_ambiguous_scope() {
        assert_eq!(secrets_path(Some(1), None).unwrap(), "/repos/1/secrets");
        assert_eq!(secrets_path(None, Some(2)).unwrap(), "/orgs/2/secrets");
        assert_eq!(secrets_path(None, None).unwrap(), "/secrets");
        assert!(secrets_path(Some(1), Some(2)).is_err());
    }

    /// Payload shape from a Woodpecker 3.x server: timestamps are `started`/`finished`,
    /// pipeline errors are an `errors` array, and unknown fields must be tolerated.
    #[test]
    fn renders_steps_from_v3_payload() {
        let pipeline: Pipeline = serde_json::from_value(json!({
            "id": 10, "number": 42, "status": "failure", "forge_url": "https://x",
            "errors": [{"type": "linter", "message": "deprecated key", "is_warning": true, "data": null}],
            "workflows": [{
                "id": 1, "pid": 1, "name": "check", "state": "failure",
                "children": [
                    {"id": 7, "pid": 2, "ppid": 1, "name": "clone", "state": "success", "exit_code": 0,
                     "type": "clone", "started": 100, "finished": 103, "uuid": "u"},
                    {"id": 8, "pid": 3, "ppid": 1, "name": "test", "state": "failure", "exit_code": 101,
                     "type": "commands", "started": 103, "finished": 160, "error": "exit code 101"},
                    {"id": 9, "pid": 4, "ppid": 1, "name": "publish", "state": "skipped", "exit_code": 0}
                ]
            }]
        }))
        .unwrap();

        assert_eq!(
            render_steps(&pipeline),
            "Pipeline #42 (failure)\n\
             Warning: deprecated key\n\
             \n\
             Workflow: check (failure)\n  \
             - step_id 7 | clone | success | type clone | exit 0 | 3s\n  \
             - step_id 8 | test | failure | type commands | exit 101 | 57s | error: exit code 101\n  \
             - step_id 9 | publish | skipped"
        );
    }
}
