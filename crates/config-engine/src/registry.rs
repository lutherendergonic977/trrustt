// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Config Registry
//
// Central registry of all known config keys with metadata.
// Used for validation, UI generation, and documentation.
// ═══════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use shared::AppError;

/// A single entry in the config registry.
///
/// Every configurable value in TRRUSTT has one of these.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// Dot-notation path (e.g., "ai.default_model").
    pub path: String,

    /// Human-readable description for settings UI.
    pub description: String,

    /// Category for grouping in UI.
    pub category: ConfigCategory,

    /// Expected value type.
    pub value_type: ConfigValueType,

    /// Default value (JSON).
    pub default_value: serde_json::Value,

    /// Whether the user can override this value.
    pub user_overridable: bool,

    /// Whether this value contains sensitive data (show as *** in exports).
    pub sensitive: bool,

    /// Whether a restart is required after changing this value.
    pub requires_restart: bool,

    /// Allowed values (if enum/options). Empty = any valid value.
    pub allowed_values: Vec<String>,

    /// Minimum value (for numeric types).
    pub min_value: Option<f64>,

    /// Maximum value (for numeric types).
    pub max_value: Option<f64>,

    /// Validation regex pattern (for string types).
    pub pattern: Option<String>,

    /// Whether this is an advanced setting (hidden by default in UI).
    pub advanced: bool,

    /// Deprecation notice (if this key is deprecated).
    pub deprecated: Option<String>,
}

/// Categories for grouping config entries in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigCategory {
    /// File paths and directories.
    Paths,
    /// XMLA / SSAS connection settings.
    Xmla,
    /// DAX engine settings.
    Dax,
    /// AI / LLM settings.
    Ai,
    /// AI provider-specific settings.
    AiProvider,
    /// RAG (retrieval-augmented generation) settings.
    Rag,
    /// Dashboard generation settings.
    Dashboard,
    /// Theme engine settings.
    Theme,
    /// MCP server/client settings.
    Mcp,
    /// License settings.
    License,
    /// Logging and telemetry.
    Logging,
    /// UI settings (Tauri window, theme, etc.).
    Ui,
    /// Database settings.
    Database,
    /// Security settings.
    Security,
    /// Internationalization.
    I18n,
    /// Feature flags.
    Features,
    /// General / uncategorized.
    General,
}

/// Expected value types for config entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigValueType {
    /// A text string.
    String,
    /// An integer.
    Integer,
    /// A floating-point number.
    Float,
    /// A boolean.
    Boolean,
    /// An array of strings.
    StringArray,
    /// A nested object.
    Object,
    /// An integer value in milliseconds.
    DurationMs,
    /// A file path.
    Path,
    /// A URL.
    Url,
    /// An email address.
    Email,
}

/// Registry of all known config keys.
pub struct ConfigRegistry {
    /// All registered config entries indexed by path.
    entries: HashMap<String, ConfigEntry>,
}

impl ConfigRegistry {
    /// Create a new registry and populate with all known config keys.
    pub fn new() -> Self {
        let mut registry = Self {
            entries: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Get a config entry by its path.
    pub fn get(&self, path: &str) -> Option<&ConfigEntry> {
        self.entries.get(path)
    }

    /// Iterate over all registered entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ConfigEntry)> {
        self.entries.iter()
    }

    /// Validate a value against its registry entry schema.
    pub fn validate(&self, path: &str, value: &serde_json::Value) -> Result<(), AppError> {
        let entry = self.entries.get(path).ok_or_else(|| {
            AppError::UnknownConfigKey(path.to_string())
        })?;

        // Type checking
        match entry.value_type {
            ConfigValueType::String | ConfigValueType::Path | ConfigValueType::Url | ConfigValueType::Email => {
                if !value.is_string() {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Expected string, got {}", value_type_name(value)),
                    });
                }
            }
            ConfigValueType::Integer => {
                if !value.is_number() || !value.as_i64().is_some() {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Expected integer, got {}", value_type_name(value)),
                    });
                }
            }
            ConfigValueType::Float => {
                if !value.is_number() {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Expected number, got {}", value_type_name(value)),
                    });
                }
            }
            ConfigValueType::Boolean => {
                if !value.is_boolean() {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Expected boolean, got {}", value_type_name(value)),
                    });
                }
            }
            ConfigValueType::StringArray => {
                if !value.is_array() {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Expected array, got {}", value_type_name(value)),
                    });
                }
            }
            ConfigValueType::Object | ConfigValueType::DurationMs => {
                // Accept any valid JSON for objects
            }
        }

        // Allowed values check
        if !entry.allowed_values.is_empty() {
            if let Some(s) = value.as_str() {
                if !entry.allowed_values.iter().any(|av| av == s) {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!(
                            "Value '{}' is not allowed. Allowed: {:?}",
                            s, entry.allowed_values
                        ),
                    });
                }
            }
        }

        // Range check
        if let Some(min) = entry.min_value {
            if let Some(n) = value.as_f64() {
                if n < min {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Value {} is below minimum {}", n, min),
                    });
                }
            }
        }
        if let Some(max) = entry.max_value {
            if let Some(n) = value.as_f64() {
                if n > max {
                    return Err(AppError::ConfigValidation {
                        key: path.to_string(),
                        reason: format!("Value {} is above maximum {}", n, max),
                    });
                }
            }
        }

        Ok(())
    }

    /// Register all known config keys with their metadata.
    fn register_defaults(&mut self) {
        // This is populated from the defaults.toml at build time.
        // In production, we'd auto-generate this from the TOML schema.
        // For now, we register the most critical entries.

        self.register(ConfigEntry {
            path: "ai.default_provider".into(),
            description: "Default AI provider to use".into(),
            category: ConfigCategory::Ai,
            value_type: ConfigValueType::String,
            default_value: serde_json::json!("openai"),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec!["openai".into(), "azure-openai".into(), "anthropic".into(), "google".into(), "deepseek".into(), "ollama".into()],
            min_value: None, max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "ai.default_model".into(),
            description: "Default AI model name".into(),
            category: ConfigCategory::Ai,
            value_type: ConfigValueType::String,
            default_value: serde_json::json!("gpt-4o"),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec![],
            min_value: None, max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "ai.params.temperature".into(),
            description: "AI temperature (0.0 = deterministic, 2.0 = creative)".into(),
            category: ConfigCategory::Ai,
            value_type: ConfigValueType::Float,
            default_value: serde_json::json!(0.3),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec![],
            min_value: Some(0.0), max_value: Some(2.0), pattern: None,
            advanced: true, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "dax.default_complexity".into(),
            description: "Default DAX complexity level".into(),
            category: ConfigCategory::Dax,
            value_type: ConfigValueType::String,
            default_value: serde_json::json!("intermediate"),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec!["beginner".into(), "intermediate".into(), "advanced".into(), "expert".into()],
            min_value: None, max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "logging.level".into(),
            description: "Minimum log level".into(),
            category: ConfigCategory::Logging,
            value_type: ConfigValueType::String,
            default_value: serde_json::json!("info"),
            user_overridable: true,
            sensitive: false,
            requires_restart: true,
            allowed_values: vec!["trace".into(), "debug".into(), "info".into(), "warn".into(), "error".into()],
            min_value: None, max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "xmla.connect_timeout_secs".into(),
            description: "SSAS connection timeout in seconds".into(),
            category: ConfigCategory::Xmla,
            value_type: ConfigValueType::Integer,
            default_value: serde_json::json!(10),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec![],
            min_value: Some(1.0), max_value: Some(120.0), pattern: None,
            advanced: true, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "ai.cost.daily_limit_usd".into(),
            description: "Maximum daily AI cost per user (USD)".into(),
            category: ConfigCategory::Ai,
            value_type: ConfigValueType::Float,
            default_value: serde_json::json!(5.0),
            user_overridable: true,
            sensitive: false,
            requires_restart: false,
            allowed_values: vec![],
            min_value: Some(0.0), max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });

        self.register(ConfigEntry {
            path: "security.encrypt_config_secrets".into(),
            description: "Encrypt sensitive config values at rest".into(),
            category: ConfigCategory::Security,
            value_type: ConfigValueType::Boolean,
            default_value: serde_json::json!(true),
            user_overridable: false,
            sensitive: false,
            requires_restart: true,
            allowed_values: vec![],
            min_value: None, max_value: None, pattern: None,
            advanced: false, deprecated: None,
        });
    }

    /// Register a single config entry.
    fn register(&mut self, entry: ConfigEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Human-readable name for a JSON value type.
fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_entries() {
        let registry = ConfigRegistry::new();
        assert!(registry.get("ai.default_provider").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_validate_string() {
        let registry = ConfigRegistry::new();
        let result = registry.validate("ai.default_provider", &serde_json::json!("openai"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wrong_type() {
        let registry = ConfigRegistry::new();
        let result = registry.validate("ai.default_provider", &serde_json::json!(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_choice() {
        let registry = ConfigRegistry::new();
        let result = registry.validate("ai.default_provider", &serde_json::json!("unknown-provider"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_range() {
        let registry = ConfigRegistry::new();
        // ai.params.temperature has min=0.0, max=2.0
        let result = registry.validate("ai.params.temperature", &serde_json::json!(5.0));
        assert!(result.is_err());

        let result = registry.validate("ai.params.temperature", &serde_json::json!(0.5));
        assert!(result.is_ok());
    }
}
