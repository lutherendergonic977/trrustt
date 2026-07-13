// TRRUSTT — MCP Hub: Tool Registry
// Dynamic MCP tool registration and discovery.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use shared::Result;
use crate::protocol::{CallToolResult, ContentBlock, Tool};

/// Handler function type for tool execution.
pub type ToolHandler = Arc<dyn Fn(Option<Value>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CallToolResult>> + Send>> + Send + Sync>;

/// Registry of MCP tools with their handler functions.
pub struct ToolRegistry {
    tools: HashMap<String, ToolEntry>,
}

struct ToolEntry {
    definition: Tool,
    handler: ToolHandler,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a tool with its handler function.
    pub fn register<F, Fut>(&mut self, name: &str, description: &str, input_schema: Value, handler: F)
    where
        F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<CallToolResult>> + Send + 'static,
    {
        let tool = Tool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        };

        let entry = ToolEntry {
            definition: tool,
            handler: Arc::new(move |args| Box::pin(handler(args))),
        };

        self.tools.insert(name.to_string(), entry);
        info!(tool = %name, "MCP tool registered");
    }

    /// List all registered tools.
    pub fn list(&self) -> Vec<Tool> {
        self.tools.values().map(|e| e.definition.clone()).collect()
    }

    /// Call a registered tool by name.
    pub async fn call(&self, name: &str, args: Option<&Value>) -> Result<CallToolResult> {
        let entry = self.tools.get(name)
            .ok_or_else(|| shared::AppError::internal(format!("Tool '{}' not found", name)))?;

        debug!(tool = %name, "Calling MCP tool");
        (entry.handler)(args.cloned()).await
    }

    /// Get the count of registered tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}

/// Helper: create a simple text result from a string.
pub fn text_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text { text: text.into() }],
        is_error: Some(false),
    }
}

/// Helper: create an error result.
pub fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text { text: message.into() }],
        is_error: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_tool_registration_and_call() {
        let mut registry = ToolRegistry::new();
        registry.register(
            "greet",
            "Greet a user by name",
            json!({"type": "object", "properties": {"name": {"type": "string"}}}),
            |args: Option<Value>| async move {
                let name = args.and_then(|a| a.get("name").and_then(|n| n.as_str()).map(String::from))
                    .unwrap_or_else(|| "world".to_string());
                Ok(text_result(format!("Hello, {}!", name)))
            },
        );

        assert_eq!(registry.count(), 1);

        let result = registry.call("greet", Some(&json!({"name": "Alice"}))).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }
}
