use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, ErrorCode, ErrorData as McpError, Implementation, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::WoodpeckerClient;
use crate::models::*;

/// MCP Server for Woodpecker CI
#[derive(Clone)]
pub struct WoodpeckerMcpServer {
    client: Arc<WoodpeckerClient>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

fn make_error(msg: String) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(msg),
        data: None,
    }
}

// Request types for tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListReposRequest {
    #[schemars(description = "Filter by active status (true/false)")]
    pub active: Option<bool>,
    #[schemars(description = "Page number (1-based)")]
    pub page: Option<i32>,
    #[schemars(description = "Items per page (default: 50)")]
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetRepoRequest {
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
    #[schemars(description = "Filter by event type (push, pull_request, tag, etc.)")]
    pub event: Option<String>,
    #[schemars(description = "Filter by status (success, failure, pending, running, etc.)")]
    pub status: Option<String>,
    #[schemars(description = "Page number (1-based)")]
    pub page: Option<i32>,
    #[schemars(description = "Items per page (default: 50)")]
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPipelineRequest {
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
    #[schemars(description = "Optional variables as key-value pairs")]
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PipelineActionRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Pipeline number")]
    pub pipeline_number: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetStepLogsRequest {
    #[schemars(description = "Repository ID")]
    pub repo_id: i64,
    #[schemars(description = "Pipeline number")]
    pub pipeline_number: i64,
    #[schemars(description = "Step ID to get logs for")]
    pub step_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSecretsRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(description = "Organization ID (for org secrets)")]
    pub org_id: Option<i64>,
    #[schemars(description = "Page number (1-based)")]
    pub page: Option<i32>,
    #[schemars(description = "Items per page")]
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSecretRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(description = "Organization ID (for org secrets)")]
    pub org_id: Option<i64>,
    #[schemars(description = "Secret name")]
    pub name: String,
    #[schemars(description = "Secret value")]
    pub value: String,
    #[schemars(description = "Events that can use this secret")]
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteSecretRequest {
    #[schemars(description = "Repository ID (for repo secrets)")]
    pub repo_id: Option<i64>,
    #[schemars(description = "Organization ID (for org secrets)")]
    pub org_id: Option<i64>,
    #[schemars(description = "Secret name")]
    pub name: String,
}

#[tool_router]
impl WoodpeckerMcpServer {
    pub fn new(client: Arc<WoodpeckerClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // ===== System Tools =====

    #[tool(description = "Get Woodpecker CI server version and build information")]
    async fn get_server_version(&self) -> Result<CallToolResult, McpError> {
        // /version is a root-level system endpoint (no /api prefix)
        let version: VersionInfo = self
            .client
            .get_root("/version")
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&version).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Check if the Woodpecker CI server is healthy and responding")]
    async fn health_check(&self) -> Result<CallToolResult, McpError> {
        // /healthz is a root-level system endpoint (no /api prefix)
        // It returns 204 No Content when healthy, not JSON
        self.client
            .get_root_text("/healthz")
            .await
            .map_err(|e| e.into_mcp_error())?;

        Ok(CallToolResult::success(vec![Content::text(
            r#"{"status": "healthy"}"#,
        )]))
    }

    #[tool(description = "Get information about the pipeline queue")]
    async fn get_queue_info(&self) -> Result<CallToolResult, McpError> {
        let info: QueueInfo = self
            .client
            .get("/queue/info")
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&info).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ===== Repository Tools =====

    #[tool(description = "List all repositories accessible to the authenticated user")]
    async fn list_repos(
        &self,
        Parameters(req): Parameters<ListReposRequest>,
    ) -> Result<CallToolResult, McpError> {
        let query = PaginationParams {
            page: req.page,
            per_page: req.per_page,
        };

        let endpoint = if let Some(active) = req.active {
            format!("/repos?active={}", active)
        } else {
            "/repos".to_string()
        };

        let repos: Vec<Repository> = self
            .client
            .get_with_query(&endpoint, &query)
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&repos).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get detailed information about a specific repository by its ID")]
    async fn get_repo(
        &self,
        Parameters(req): Parameters<GetRepoRequest>,
    ) -> Result<CallToolResult, McpError> {
        let repo: Repository = self
            .client
            .get(&format!("/repos/{}", req.repo_id))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&repo).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Find a repository by its full name (owner/repo format)")]
    async fn lookup_repo(
        &self,
        Parameters(req): Parameters<LookupRepoRequest>,
    ) -> Result<CallToolResult, McpError> {
        let repo: Repository = self
            .client
            .get(&format!("/repos/lookup/{}", req.full_name))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&repo).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ===== Pipeline Tools =====

    #[tool(description = "List pipelines for a repository with optional filters")]
    async fn list_pipelines(
        &self,
        Parameters(req): Parameters<ListPipelinesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let query = PipelineFilter {
            branch: req.branch,
            event: req.event,
            status: req.status,
            page: req.page,
            per_page: req.per_page,
        };

        let pipelines: Vec<Pipeline> = self
            .client
            .get_with_query(&format!("/repos/{}/pipelines", req.repo_id), &query)
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipelines).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get detailed information about a specific pipeline")]
    async fn get_pipeline(
        &self,
        Parameters(req): Parameters<GetPipelineRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline: Pipeline = self
            .client
            .get(&format!(
                "/repos/{}/pipelines/{}",
                req.repo_id, req.pipeline_number
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipeline).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Trigger a new pipeline build on a specific branch")]
    async fn create_pipeline(
        &self,
        Parameters(req): Parameters<CreatePipelineRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = PipelineCreate {
            branch: req.branch,
            variables: req.variables.unwrap_or_default(),
        };

        let pipeline: Pipeline = self
            .client
            .post(&format!("/repos/{}/pipelines", req.repo_id), &body)
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipeline).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Restart a previously completed or failed pipeline")]
    async fn restart_pipeline(
        &self,
        Parameters(req): Parameters<PipelineActionRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline: Pipeline = self
            .client
            .post_empty(&format!(
                "/repos/{}/pipelines/{}",
                req.repo_id, req.pipeline_number
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipeline).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Cancel a currently running pipeline")]
    async fn cancel_pipeline(
        &self,
        Parameters(req): Parameters<PipelineActionRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .post_no_content(&format!(
                "/repos/{}/pipelines/{}/cancel",
                req.repo_id, req.pipeline_number
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Pipeline {} cancelled successfully",
            req.pipeline_number
        ))]))
    }

    #[tool(description = "Approve a gated pipeline waiting for manual approval")]
    async fn approve_pipeline(
        &self,
        Parameters(req): Parameters<PipelineActionRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline: Pipeline = self
            .client
            .post_empty(&format!(
                "/repos/{}/pipelines/{}/approve",
                req.repo_id, req.pipeline_number
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipeline).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Decline a gated pipeline waiting for manual approval")]
    async fn decline_pipeline(
        &self,
        Parameters(req): Parameters<PipelineActionRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline: Pipeline = self
            .client
            .post_empty(&format!(
                "/repos/{}/pipelines/{}/decline",
                req.repo_id, req.pipeline_number
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json =
            serde_json::to_string_pretty(&pipeline).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ===== Log Tools =====

    #[tool(description = "Get the execution logs for a specific pipeline step")]
    async fn get_step_logs(
        &self,
        Parameters(req): Parameters<GetStepLogsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let logs: Vec<LogEntry> = self
            .client
            .get(&format!(
                "/repos/{}/logs/{}/{}",
                req.repo_id, req.pipeline_number, req.step_id
            ))
            .await
            .map_err(|e| e.into_mcp_error())?;

        let formatted: String = logs
            .iter()
            .filter_map(|entry| entry.out.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("");

        if formatted.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(
                "No logs available for this step",
            )]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(formatted)]))
        }
    }

    // ===== Secret Tools =====

    #[tool(description = "List secrets (global, org, or repo level based on parameters)")]
    async fn list_secrets(
        &self,
        Parameters(req): Parameters<ListSecretsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let query = PaginationParams {
            page: req.page,
            per_page: req.per_page,
        };

        let endpoint = if let Some(repo_id) = req.repo_id {
            format!("/repos/{}/secrets", repo_id)
        } else if let Some(org_id) = req.org_id {
            format!("/orgs/{}/secrets", org_id)
        } else {
            "/secrets".to_string()
        };

        let secrets: Vec<Secret> = self
            .client
            .get_with_query(&endpoint, &query)
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&secrets).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Create a new secret (global, org, or repo level based on parameters)")]
    async fn create_secret(
        &self,
        Parameters(req): Parameters<CreateSecretRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = SecretCreate {
            name: req.name,
            value: req.value,
            events: req.events,
            images: None,
        };

        let endpoint = if let Some(repo_id) = req.repo_id {
            format!("/repos/{}/secrets", repo_id)
        } else if let Some(org_id) = req.org_id {
            format!("/orgs/{}/secrets", org_id)
        } else {
            "/secrets".to_string()
        };

        let secret: Secret = self
            .client
            .post(&endpoint, &body)
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&secret).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Delete a secret (global, org, or repo level based on parameters)")]
    async fn delete_secret(
        &self,
        Parameters(req): Parameters<DeleteSecretRequest>,
    ) -> Result<CallToolResult, McpError> {
        let endpoint = if let Some(repo_id) = req.repo_id {
            format!("/repos/{}/secrets/{}", repo_id, req.name)
        } else if let Some(org_id) = req.org_id {
            format!("/orgs/{}/secrets/{}", org_id, req.name)
        } else {
            format!("/secrets/{}", req.name)
        };

        self.client
            .delete(&endpoint)
            .await
            .map_err(|e| e.into_mcp_error())?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Secret '{}' deleted successfully",
            req.name
        ))]))
    }

    // ===== User Tools =====

    #[tool(description = "Get information about the currently authenticated user")]
    async fn get_current_user(&self) -> Result<CallToolResult, McpError> {
        let user: User = self
            .client
            .get("/user")
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&user).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get the authenticated user's feed of recent pipelines")]
    async fn get_user_feed(&self) -> Result<CallToolResult, McpError> {
        let feed: Vec<FeedItem> = self
            .client
            .get("/user/feed")
            .await
            .map_err(|e| e.into_mcp_error())?;

        let json = serde_json::to_string_pretty(&feed).map_err(|e| make_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler]
impl ServerHandler for WoodpeckerMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Woodpecker CI MCP Server - Interact with Woodpecker CI pipelines, \
                 repositories, secrets, and more."
                    .to_string(),
            ),
        }
    }
}
