// TRRUSTT — MCP Hub: Client
// MCP client for connecting to external MCP servers.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use shared::Result;
use crate::protocol::*;
use crate::transport::Transport;

/// Connected MCP server session.
struct McpSession {
    transport: Transport,
    capabilities: ServerCapabilities,
}

/// MCP client managing connections to multiple MCP servers.
pub struct McpClient {
    sessions: Arc<RwLock<HashMap<String, McpSession>>>,
}

impl McpClient {
    /// Create a new MCP client.
    pub fn new() -> Self {
        Self { sessions: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Connect to an MCP server via stdio (spawns a process).
    #[instrument]
    pub async fn connect_stdio(&self, name: &str, command: &str, args: &[&str]) -> Result<()> {
        info!(server = %name, command = %command, "Connecting to MCP server via stdio");

        let transport = Transport::stdio(command, args).await?;

        // Send initialize
        let init_result = Self::do_initialize(&transport).await?;

        let session = McpSession {
            transport,
            capabilities: init_result.capabilities,
        };

        self.sessions.write().await.insert(name.to_string(), session);

        info!(server = %name, "Connected to MCP server: {}", init_result.server_info.name);
        Ok(())
    }

    /// List tools from all connected servers.
    pub async fn list_tools(&self, server: &str) -> Result<Vec<Tool>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(server)
            .ok_or_else(|| shared::AppError::entity_not_found("MCP server", server))?;

        let resp = session.transport.send_request("tools/list", None).await?;
        let result: ListToolsResult = serde_json::from_value(resp)
            .map_err(|e| shared::AppError::internal(format!("Failed to parse tools/list response: {}", e)))?;

        Ok(result.tools)
    }

    /// Call a tool on a specific MCP server.
    pub async fn call_tool(&self, server: &str, params: CallToolParams) -> Result<CallToolResult> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(server)
            .ok_or_else(|| shared::AppError::entity_not_found("MCP server", server))?;

        let request_params = serde_json::to_value(&params)
            .map_err(|e| shared::AppError::internal(e.to_string()))?;

        let resp = session.transport.send_request("tools/call", Some(request_params)).await?;
        let result: CallToolResult = serde_json::from_value(resp)
            .map_err(|e| shared::AppError::internal(format!("Failed to parse tools/call response: {}", e)))?;

        Ok(result)
    }

    /// Disconnect from a server.
    pub async fn disconnect(&self, name: &str) {
        self.sessions.write().await.remove(name);
        info!(server = %name, "Disconnected MCP server");
    }

    /// List connected servers.
    pub async fn list_servers(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }

    /// Perform the MCP initialize handshake with a server.
    async fn do_initialize(transport: &Transport) -> Result<InitializeResult> {
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({
                "roots": {},
                "sampling": {}
            }),
            client_info: ImplementationInfo {
                name: "TRRUSTT".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let request_params = serde_json::to_value(&init_params)
            .map_err(|e| shared::AppError::internal(e.to_string()))?;

        let resp = transport.send_request("initialize", Some(request_params)).await?;
        let result: InitializeResult = serde_json::from_value(resp)
            .map_err(|e| shared::AppError::internal(format!("Failed to parse initialize response: {}", e)))?;

        Ok(result)
    }
}

impl Default for McpClient {
    fn default() -> Self { Self::new() }
}
