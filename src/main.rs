use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rmcp::{transport::stdio, ServiceExt};
use woodpecker_ci_mcp::{Config, WoodpeckerClient, WoodpeckerMcpServer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (stdout is used for MCP communication)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "woodpecker_ci_mcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Woodpecker CI MCP Server");

    // Load configuration from environment
    let config = Config::from_env().map_err(|e| {
        tracing::error!("Configuration error: {}", e);
        e
    })?;

    tracing::info!("Connecting to Woodpecker server: {}", config.server_url);

    // Create the Woodpecker API client
    let client = Arc::new(WoodpeckerClient::new(&config)?);

    // Create the MCP server
    let service = WoodpeckerMcpServer::new(client);

    // Run the server with stdio transport
    tracing::info!("MCP server ready, waiting for connections...");

    let server = service.serve(stdio()).await?;

    // Wait for the server to complete
    server.waiting().await?;

    tracing::info!("MCP server shutting down");
    Ok(())
}
