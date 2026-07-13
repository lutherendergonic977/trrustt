// TRRUSTT — MCP Hub: Server
// MCP stdio server implementation.

use std::sync::Arc;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{error, info, instrument};

use shared::Result;
use crate::protocol::*;
use crate::tools::ToolRegistry;
use crate::resources::ResourceManager;

/// MCP server running on stdio transport.
pub struct McpServer {
    tools: Arc<RwLock<ToolRegistry>>,
    resources: Arc<RwLock<ResourceManager>>,
    server_info: ImplementationInfo,
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server with the given tool registry and resource manager.
    pub fn new(tools: ToolRegistry, resources: ResourceManager) -> Self {
        Self {
            tools: Arc::new(RwLock::new(tools)),
            resources: Arc::new(RwLock::new(resources)),
            server_info: ImplementationInfo {
                name: "TRRUSTT".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            initialized: false,
        }
    }

    /// Run the MCP server on stdio.
    /// Reads JSON-RPC requests from stdin, processes them, writes responses to stdout.
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        info!("MCP server started on stdio");

        while let Some(line) = lines.next_line().await.map_err(|e| {
            shared::AppError::internal(format!("Stdin read error: {}", e))
        })? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let response = self.handle_message(&line).await;
            if let Some(resp_json) = response {
                let mut out = stdout;
                out.write_all(resp_json.as_bytes()).await.map_err(|e| {
                    shared::AppError::internal(format!("Stdout write error: {}", e))
                })?;
                out.write_all(b"\n").await.map_err(|e| {
                    shared::AppError::internal(format!("Stdout write error: {}", e))
                })?;
                out.flush().await.map_err(|e| {
                    shared::AppError::internal(format!("Stdout flush error: {}", e))
                })?;
            }
        }

        info!("MCP server stopped");
        Ok(())
    }

    /// Handle a single JSON-RPC message.
    async fn handle_message(&mut self, line: &str) -> Option<String> {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, JsonRpcError::PARSE_ERROR, &format!("Parse error: {}", e));
                return Some(serde_json::to_string(&resp).unwrap());
            }
        };

        // Notifications (no id) — process but don't respond
        if request.id.is_none() {
            self.handle_notification(&request.method, &request.params).await;
            return None;
        }

        let id = request.id.clone();
        let result = self.dispatch(&request.method, &request.params).await;

        let response = match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::INTERNAL_ERROR, &e.to_string()),
        };

        Some(serde_json::to_string(&response).unwrap())
    }

    /// Dispatch a method call to the appropriate handler.
    async fn dispatch(&mut self, method: &str, params: &Option<Value>) -> Result<Value> {
        match method {
            "initialize" => self.handle_initialize(params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(params).await,
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => self.handle_read_resource(params).await,
            _ => Err(shared::AppError::internal(format!("Unknown method: {}", method))),
        }
    }

    /// Handle the initialize handshake.
    async fn handle_initialize(&mut self, params: &Option<Value>) -> Result<Value> {
        let _init_params: InitializeParams = params
            .as_ref()
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .unwrap_or(InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: json!({}),
                client_info: ImplementationInfo { name: "unknown".to_string(), version: "0".to_string() },
            });

        self.initialized = true;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: Some(false) }),
                resources: Some(ResourcesCapability { subscribe: Some(false), list_changed: Some(false) }),
            },
            server_info: self.server_info.clone(),
        };

        Ok(serde_json::to_value(result).map_err(|e| shared::AppError::internal(e.to_string()))?)
    }

    /// Handle tools/list — return all registered tools.
    async fn handle_list_tools(&self) -> Result<Value> {
        let tools = self.tools.read().await.list();
        let result = ListToolsResult { tools };
        Ok(serde_json::to_value(result).map_err(|e| shared::AppError::internal(e.to_string()))?)
    }

    /// Handle tools/call — execute a tool.
    async fn handle_call_tool(&self, params: &Option<Value>) -> Result<Value> {
        let call_params: CallToolParams = params
            .as_ref()
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .ok_or_else(|| shared::AppError::internal("Invalid call_tool params"))?;

        let result = self.tools.read().await.call(&call_params.name, call_params.arguments.as_ref()).await?;
        Ok(serde_json::to_value(result).map_err(|e| shared::AppError::internal(e.to_string()))?)
    }

    /// Handle resources/list.
    async fn handle_list_resources(&self) -> Result<Value> {
        let resources = self.resources.read().await.list();
        let result = ListResourcesResult { resources };
        Ok(serde_json::to_value(result).map_err(|e| shared::AppError::internal(e.to_string()))?)
    }

    /// Handle resources/read.
    async fn handle_read_resource(&self, _params: &Option<Value>) -> Result<Value> {
        Ok(json!({"contents": []}))
    }

    /// Handle notifications (fire-and-forget).
    async fn handle_notification(&self, method: &str, _params: &Option<Value>) {
        info!(method = %method, "MCP notification received");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let tools = ToolRegistry::new();
        let resources = ResourceManager::new();
        let server = McpServer::new(tools, resources);
        assert!(!server.initialized);
        assert_eq!(server.server_info.name, "TRRUSTT");
    }
}
