// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Config Engine Core
//
// 7-layer config resolution using figment.
// Resolution order (highest priority first):
//   1. Environment Variables (TRRUSTT_ prefix)
//   2. CLI Arguments
//   3. User Config (~/.trrustt/config/user.toml)
//   4. Project Config (.trrustt/config/project.toml)
//   5. Workspace Config (~/.trrustt/config/workspace.toml)
//   6. Admin-Enforced Policies
//   7. System Defaults (embedded at compile time)
// ═══════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use figment::providers::{Env, Serialized, Toml};
use figment::Figment;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info, instrument, warn};

use shared::{AppError, Result, CONFIG_DIR, ENV_PREFIX};

use crate::encryption::EncryptionEngine;
use crate::layers::ConfigScope;
use crate::policy::{AdminPolicy, PolicyEngine};
use crate::registry::ConfigRegistry;
use crate::validation::ConfigValidator;

/// The central configuration engine.
///
/// All config values flow through this struct. It resolves values from
/// 7 layers of configuration, validates them against the registry, and
/// enforces admin policies.
pub struct ConfigEngine {
    /// Figment provider chain for layered resolution.
    figment: Figment,

    /// Registry of all known config keys with metadata/validation.
    registry: ConfigRegistry,

    /// Admin-enforced policies (non-overridable by users).
    admin_policies: DashMap<String, AdminPolicy>,

    /// Encryption engine for sensitive values.
    encryption: Arc<EncryptionEngine>,

    /// CLI overrides collected at startup.
    cli_overrides: HashMap<String, String>,

    /// Base config directory path.
    config_dir: PathBuf,

    /// Policy engine for rule evaluation.
    policy_engine: PolicyEngine,
}

impl ConfigEngine {
    /// Build the engine from all config sources.
    ///
    /// # Arguments
    /// * `cli_overrides` - Key-value pairs from CLI arguments (Layer 2).
    ///
    /// # Returns
    /// A fully initialized `ConfigEngine` ready for config resolution.
    ///
    /// # Errors
    /// Returns `ConfigResolution` if any config source fails to parse.
    #[instrument(name = "config_engine_init", skip(cli_overrides))]
    pub fn new(cli_overrides: HashMap<String, String>) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_DIR);

        let user_config_path = config_dir.join("config").join("user.toml");
        let workspace_config_path = config_dir.join("config").join("workspace.toml");
        let project_config_path = PathBuf::from(".trrustt").join("config").join("project.toml");
        let admin_policy_path = config_dir.join("config").join("admin-policies.toml");

        info!(
            config_dir = %config_dir.display(),
            "Initializing configuration engine"
        );

        // ── Layer 7: System defaults (embedded at compile time) ────────
        let defaults_str = include_str!("../../../config/defaults.toml");
        let mut figment = Figment::new()
            .merge(Toml::string(defaults_str));

        // ── Layer 6: Admin policies (if file exists) ──────────────────
        if admin_policy_path.exists() {
            debug!(path = %admin_policy_path.display(), "Loading admin policies");
            figment = figment.merge(Toml::file(&admin_policy_path));
        }

        // ── Layer 5: Workspace config ─────────────────────────────────
        if workspace_config_path.exists() {
            debug!(path = %workspace_config_path.display(), "Loading workspace config");
            figment = figment.merge(Toml::file(&workspace_config_path));
        }

        // ── Layer 4: Project config ───────────────────────────────────
        if project_config_path.exists() {
            debug!(path = %project_config_path.display(), "Loading project config");
            figment = figment.merge(Toml::file(&project_config_path));
        }

        // ── Layer 3: User config ──────────────────────────────────────
        if user_config_path.exists() {
            debug!(path = %user_config_path.display(), "Loading user config");
            figment = figment.merge(Toml::file(&user_config_path));
        }

        // ── Layer 2: CLI overrides ────────────────────────────────────
        if !cli_overrides.is_empty() {
            debug!(count = cli_overrides.len(), "Applying CLI overrides");
            figment = figment.merge(Serialized::defaults(&cli_overrides));
        }

        // ── Layer 1: Environment variables ────────────────────────────
        figment = figment.merge(
            Env::prefixed(ENV_PREFIX)
                .split('_'),
        );

        // ── Load admin policies into memory ───────────────────────────
        let admin_policies = DashMap::new();
        // Admin policies are loaded lazily via load_policies()

        // ── Initialize encryption engine ──────────────────────────────
        let encryption = Arc::new(EncryptionEngine::new(&config_dir)?);

        // ── Build config registry ─────────────────────────────────────
        let registry = ConfigRegistry::new();

        // ── Build policy engine ───────────────────────────────────────
        let policy_engine = PolicyEngine::new();

        info!("Configuration engine initialized successfully");

        Ok(Self {
            figment,
            registry,
            admin_policies,
            encryption,
            cli_overrides,
            config_dir,
            policy_engine,
        })
    }

    /// Get a fully resolved config value by its dot-notation path.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into (must implement `DeserializeOwned`).
    ///
    /// # Arguments
    /// * `path` - Dot-notation config path (e.g., "ai.default_model").
    ///
    /// # Returns
    /// The resolved value, accounting for all 7 layers and admin policy enforcement.
    ///
    /// # Errors
    /// Returns `ConfigNotFound` if the key is not in the registry.
    /// Returns `ConfigResolution` if the value fails to deserialize.
    #[instrument(skip(self), fields(config_key = %path))]
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        // Check if the key is registered
        let _entry = self.registry.get(path)
            .ok_or_else(|| AppError::ConfigNotFound(path.to_string()))?;

        // Check admin policy enforcement
        if let Some(policy) = self.admin_policies.get(path) {
            if policy.enforce {
                debug!(key = %path, "Admin policy enforced — returning admin value");
                let value: T = serde_json::from_value(policy.value.clone())
                    .map_err(|e| AppError::ConfigResolution {
                        key: path.to_string(),
                        reason: e.to_string(),
                    })?;
                return Ok(value);
            }
        }

        // Resolve through figment's layered chain
        self.figment
            .extract_inner::<T>(path)
            .map_err(|e| AppError::ConfigResolution {
                key: path.to_string(),
                reason: e.to_string(),
            })
    }

    /// Get a config value with a default fallback if not found.
    pub fn get_or<T: DeserializeOwned>(&self, path: &str, default: T) -> T {
        self.get(path).unwrap_or(default)
    }

    /// Set a config value at a specific scope.
    ///
    /// # Arguments
    /// * `path` - Dot-notation config path.
    /// * `value` - The value to set.
    /// * `scope` - Which layer to write to.
    ///
    /// # Errors
    /// Returns validation errors if the value doesn't match the registry schema.
    /// Returns `NotUserOverridable` if the key cannot be changed at this scope.
    /// Returns `AdminEnforced` if an admin policy blocks the change.
    #[instrument(skip(self, value), fields(config_key = %path, scope = ?scope))]
    pub async fn set<T: Serialize + std::fmt::Debug>(
        &self,
        path: &str,
        value: T,
        scope: ConfigScope,
    ) -> Result<()> {
        // Validate against registry schema
        let entry = self.registry.get(path)
            .ok_or_else(|| AppError::UnknownConfigKey(path.to_string()))?;

        let json_value = serde_json::to_value(&value)
            .map_err(|e| AppError::Parse {
                path: PathBuf::from(path),
                reason: e.to_string(),
            })?;

        self.registry.validate(path, &json_value)?;

        // Check user-overridable
        if scope == ConfigScope::User && !entry.user_overridable {
            return Err(AppError::NotUserOverridable(path.to_string()));
        }

        // Check admin policy block
        if let Some(policy) = self.admin_policies.get(path) {
            if policy.enforce && !policy.allow_user_override && scope != ConfigScope::Admin {
                return Err(AppError::AdminEnforced(path.to_string()));
            }
        }

        // Write to appropriate file
        let file_path = self.scope_path(scope);
        self.write_config(&file_path, path, &json_value).await?;

        info!(key = %path, scope = ?scope, "Configuration value set");
        Ok(())
    }

    /// Reset a config key to its default at a given scope.
    #[instrument(skip(self), fields(config_key = %path, scope = ?scope))]
    pub async fn reset(&self, path: &str, scope: ConfigScope) -> Result<()> {
        let file_path = self.scope_path(scope);
        self.remove_key(&file_path, path).await?;
        info!(key = %path, scope = ?scope, "Configuration key reset to default");
        Ok(())
    }

    /// Export all resolved config as JSON.
    ///
    /// Sensitive values are redacted.
    #[instrument(skip(self))]
    pub fn export(&self) -> Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (path, entry) in self.registry.iter() {
            if entry.sensitive {
                map.insert(path.to_string(), serde_json::Value::String("***REDACTED***".into()));
            } else {
                match self.get::<serde_json::Value>(path) {
                    Ok(value) => { map.insert(path.to_string(), value); }
                    Err(_) => {
                        map.insert(path.to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
        Ok(serde_json::Value::Object(map))
    }

    /// Import config from JSON data at a given scope.
    #[instrument(skip(self, data))]
    pub async fn import(
        &self,
        data: serde_json::Value,
        scope: ConfigScope,
    ) -> Result<()> {
        if let serde_json::Value::Object(map) = data {
            for (path, value) in map {
                self.set(&path, value, scope).await?;
            }
        }
        info!(scope = ?scope, "Configuration imported");
        Ok(())
    }

    /// Get the encryption engine for sensitive value handling.
    pub fn encryption(&self) -> &Arc<EncryptionEngine> {
        &self.encryption
    }

    /// Get the config registry for introspection.
    pub fn registry(&self) -> &ConfigRegistry {
        &self.registry
    }

    /// Get the policy engine.
    pub fn policies(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    /// Get the config directory path.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Load admin policies from a URL or file.
    #[instrument(skip(self))]
    pub async fn load_admin_policies(&self, _source: &str) -> Result<()> {
        // In production, this would:
        // 1. Fetch signed policy file from admin server
        // 2. Verify Ed25519 signature
        // 3. Parse and load into admin_policies DashMap
        warn!("Admin policy loading not yet implemented");
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────

    /// Get the filesystem path for a given config scope.
    fn scope_path(&self, scope: ConfigScope) -> PathBuf {
        match scope {
            ConfigScope::Admin => self.config_dir.join("config").join("admin-policies.toml"),
            ConfigScope::Workspace => self.config_dir.join("config").join("workspace.toml"),
            ConfigScope::Project => PathBuf::from(".trrustt").join("config").join("project.toml"),
            ConfigScope::User => self.config_dir.join("config").join("user.toml"),
        }
    }

    /// Write a config value to a TOML file.
    async fn write_config(&self, file_path: &Path, key: &str, value: &serde_json::Value) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                AppError::io(file_path.to_path_buf(), e)
            })?;
        }

        // Read existing TOML or start fresh
        let mut toml_value: toml::Value = if file_path.exists() {
            let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
                AppError::io(file_path.to_path_buf(), e)
            })?;
            toml::from_str(&content).unwrap_or(toml::Value::Table(toml::map::Map::new()))
        } else {
            toml::Value::Table(toml::map::Map::new())
        };

        // Navigate and set the nested key
        set_nested_toml_key(&mut toml_value, key, value);

        // Write back
        let content = toml::to_string_pretty(&toml_value)
            .map_err(|e| AppError::Parse {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            })?;

        tokio::fs::write(file_path, content).await.map_err(|e| {
            AppError::io(file_path.to_path_buf(), e)
        })?;

        Ok(())
    }

    /// Remove a config key from a TOML file.
    async fn remove_key(&self, file_path: &Path, key: &str) -> Result<()> {
        if !file_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            AppError::io(file_path.to_path_buf(), e)
        })?;

        let mut toml_value: toml::Value = toml::from_str(&content).unwrap_or_default();
        remove_nested_toml_key(&mut toml_value, key);

        let content = toml::to_string_pretty(&toml_value)
            .map_err(|e| AppError::Parse {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            })?;

        tokio::fs::write(file_path, content).await.map_err(|e| {
            AppError::io(file_path.to_path_buf(), e)
        })?;

        Ok(())
    }
}

// ── TOML key manipulation helpers ─────────────────────────────────────

/// Set a nested key in a TOML value (e.g., "ai.providers.openai.enabled").
fn set_nested_toml_key(root: &mut toml::Value, key: &str, value: &serde_json::Value) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part: set the value
            let toml_val = json_to_toml(value);
            if let toml::Value::Table(table) = current {
                table.insert(part.to_string(), toml_val);
            }
        } else {
            // Intermediate part: traverse or create table
            if let toml::Value::Table(table) = current {
                if !table.contains_key(*part) {
                    table.insert(part.to_string(), toml::Value::Table(toml::map::Map::new()));
                }
                // Safe: key guaranteed to exist after insert above
                current = table.get_mut(*part).expect("Key just inserted into TOML table");
            }
        }
    }
}

/// Remove a nested key from a TOML value.
fn remove_nested_toml_key(root: &mut toml::Value, key: &str) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let toml::Value::Table(table) = current {
                table.remove(*part);
            }
        } else if let toml::Value::Table(table) = current {
            if let Some(next) = table.get_mut(*part) {
                current = next;
            } else {
                return;
            }
        }
    }
}

/// Convert a `serde_json::Value` to a `toml::Value`.
fn json_to_toml(json: &serde_json::Value) -> toml::Value {
    match json {
        serde_json::Value::String(s) => toml::Value::String(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                toml::Value::Float(f)
            } else {
                toml::Value::String(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
        serde_json::Value::Array(arr) => {
            let toml_arr: toml::value::Array = arr.iter().map(json_to_toml).collect();
            toml::Value::Array(toml_arr)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v));
            }
            toml::Value::Table(table)
        }
        serde_json::Value::Null => toml::Value::String("null".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_toml_string() {
        let json = serde_json::json!("hello");
        let toml_val = json_to_toml(&json);
        assert_eq!(toml_val, toml::Value::String("hello".to_string()));
    }

    #[test]
    fn test_json_to_toml_integer() {
        let json = serde_json::json!(42);
        let toml_val = json_to_toml(&json);
        assert_eq!(toml_val, toml::Value::Integer(42));
    }

    #[test]
    fn test_json_to_toml_bool() {
        let json = serde_json::json!(true);
        let toml_val = json_to_toml(&json);
        assert_eq!(toml_val, toml::Value::Boolean(true));
    }

    #[test]
    fn test_set_nested_toml_key() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        set_nested_toml_key(&mut root, "ai.default_model", &serde_json::json!("gpt-4o"));
        set_nested_toml_key(&mut root, "ai.params.temperature", &serde_json::json!(0.5));

        let table = root.as_table().expect("root should be a table");
        assert!(table.contains_key("ai"));
        let ai = table["ai"].as_table().expect("ai should be a table");
        assert_eq!(ai["default_model"].as_str(), Some("gpt-4o"));
        let params = ai["params"].as_table().expect("params should be a table");
        assert_eq!(params["temperature"].as_float(), Some(0.5));
    }
}

/// Convert a JSON value to a TOML value.
fn json_to_toml(value: &serde_json::Value) -> toml::Value {
    match value {
        serde_json::Value::Null => toml::Value::String("".into()),
        serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                toml::Value::Float(f)
            } else {
                toml::Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => toml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            toml::Value::Array(arr.iter().map(json_to_toml).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = toml::map::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_toml(v));
            }
            toml::Value::Table(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_engine_new() {
        let engine = ConfigEngine::new(HashMap::new());
        assert!(engine.is_ok());
    }

    #[test]
    fn test_get_default_value() {
        let engine = ConfigEngine::new(HashMap::new()).unwrap();
        let level: String = engine.get("logging.level").unwrap();
        assert_eq!(level, "info");
    }

    #[test]
    fn test_get_unknown_key() {
        let engine = ConfigEngine::new(HashMap::new()).unwrap();
        let result: Result<String> = engine.get("nonexistent.key");
        assert!(result.is_err());
    }
}
