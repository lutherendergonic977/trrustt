// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Config Scope / Layers
//
// Defines the config scopes (layers) and their file paths.
// ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

/// The scope (layer) at which a config value is set.
///
/// Higher-priority scopes override lower-priority ones.
/// Admin > Workspace > Project > User > System Defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    /// Admin-enforced policies (highest priority, non-user-overridable).
    Admin,
    /// Workspace-level config.
    Workspace,
    /// Project-level config (.trrustt/config/project.toml).
    Project,
    /// User-level config (~/.trrustt/config/user.toml).
    User,
}

impl ConfigScope {
    /// Parse from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "workspace" => Some(Self::Workspace),
            "project" => Some(Self::Project),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

impl std::fmt::Display for ConfigScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConfigScope::Admin => "admin",
            ConfigScope::Workspace => "workspace",
            ConfigScope::Project => "project",
            ConfigScope::User => "user",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_parse() {
        assert_eq!(ConfigScope::parse("admin"), Some(ConfigScope::Admin));
        assert_eq!(ConfigScope::parse("USER"), Some(ConfigScope::User));
        assert_eq!(ConfigScope::parse("invalid"), None);
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(ConfigScope::Admin.to_string(), "admin");
        assert_eq!(ConfigScope::Project.to_string(), "project");
    }
}
