// TRRUSTT — MCP Hub
// MCP server (stdio transport) and client for external MCP servers.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod protocol;
pub mod server;
pub mod client;
pub mod tools;
pub mod resources;
pub mod transport;

use shared::Result;

pub use protocol::*;
pub use server::McpServer;
pub use client::McpClient;
pub use tools::ToolRegistry;
pub use resources::ResourceManager;

/// The MCP hub — manages both server and client roles.
/// Comes pre-configured with Microsoft Power BI MCP servers as defaults.
pub struct McpHub {
    server: McpServer,
    client: McpClient,
}

impl McpHub {
    /// Create a new MCP hub with default tool registrations.
    pub fn new() -> Self {
        let tools = ToolRegistry::new();
        let resources = ResourceManager::new();
        let server = McpServer::new(tools, resources);
        let client = McpClient::new();
        Self { server, client }
    }

    /// Get a reference to the MCP server.
    pub fn server(&self) -> &McpServer { &self.server }

    /// Get a mutable reference to the MCP server.
    pub fn server_mut(&mut self) -> &mut McpServer { &mut self.server }

    /// Get a reference to the MCP client.
    pub fn client(&self) -> &McpClient { &self.client }

    /// Run the MCP server on stdio.
    pub async fn run_server(&mut self) -> Result<()> {
        self.server.run().await
    }

    /// Connect to the Microsoft Power BI Modeling MCP Server (local).
    pub async fn connect_pbi_modeling_server(&self) -> Result<()> {
        self.client.connect_stdio(
            "powerbi-modeling",
            "powerbi-modeling-mcp-server",
            &[],
        ).await
    }

    /// Connect to the Microsoft Power BI MCP Server (remote/enterprise).
    pub async fn connect_pbi_remote_server(&self) -> Result<()> {
        self.client.connect_stdio(
            "powerbi-remote",
            "powerbi-mcp-server",
            &[],
        ).await
    }
}

impl Default for McpHub {
    fn default() -> Self { Self::new() }
}
