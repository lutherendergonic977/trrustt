// TRRUSTT — MCP Hub: Resource Manager
// MCP resource definitions for Power BI model assets.

use crate::protocol::Resource;

/// Manager for MCP resource definitions.
pub struct ResourceManager {
    resources: Vec<Resource>,
}

impl ResourceManager {
    /// Create a new resource manager with default Power BI resources.
    pub fn new() -> Self {
        let mut manager = Self { resources: Vec::new() };
        manager.register_defaults();
        manager
    }

    /// Register default MCP resources for Power BI model assets.
    fn register_defaults(&mut self) {
        self.resources.push(Resource {
            uri: "trrustt://schema/current".to_string(),
            name: "Current Model Schema".to_string(),
            description: Some("Full schema metadata of the connected Power BI model".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        self.resources.push(Resource {
            uri: "trrustt://config/export".to_string(),
            name: "Configuration Export".to_string(),
            description: Some("All resolved configuration values".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        self.resources.push(Resource {
            uri: "trrustt://measures/list".to_string(),
            name: "Measures List".to_string(),
            description: Some("All measures in the current model".to_string()),
            mime_type: Some("application/json".to_string()),
        });
    }

    /// Register a new resource.
    pub fn register(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    /// List all registered resources.
    pub fn list(&self) -> Vec<Resource> {
        self.resources.clone()
    }

    /// Find a resource by URI.
    pub fn get(&self, uri: &str) -> Option<&Resource> {
        self.resources.iter().find(|r| r.uri == uri)
    }
}

impl Default for ResourceManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_resources() {
        let manager = ResourceManager::new();
        let resources = manager.list();
        assert!(resources.len() >= 3);
        assert!(resources.iter().any(|r| r.uri == "trrustt://schema/current"));
    }
}
