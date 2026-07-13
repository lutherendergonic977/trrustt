# Configuration Engine — Rust Implementation v2

## 1. Design Philosophy

> **"If it exists in the product, it MUST be configurable. Nothing is hard-coded. Ever."**

The Configuration Engine is the central nervous system. Every behavior, threshold, model name, prompt path, theme setting, license parameter, and operational characteristic is externalized into config — never compiled into the binary.

## 2. Architecture

### 2.1 Resolution Layers (Priority: highest first)

```
1. Environment Variables      TRRUSTT_AI_PROVIDER=openai
2. CLI Arguments              --ai-provider anthropic
3. User Config                ~/.intellidashboard/config/user.toml
4. Project Config             .intellidashboard/config/project.toml
5. Workspace Config           ~/.intellidashboard/config/workspace.toml
6. Admin-Enforced Policies    Centrally distributed, signed
7. System Defaults            Embedded at compile time (build.rs)
```

### 2.2 Crate Structure

```rust
// crates/config-engine/src/lib.rs

use figment::{
    Figment,
    providers::{Toml, Env, Format},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct ConfigEngine {
    /// Figment provider chain (layered resolution)
    figment: Figment,
    
    /// Registry of all known config keys with metadata
    registry: ConfigRegistry,
    
    /// Admin-enforced policies (non-overridable by users)
    admin_policies: HashMap<String, AdminPolicyEntry>,
    
    /// File watcher handles for hot-reload
    watchers: Vec<FileWatcher>,
}

impl ConfigEngine {
    /// Build the engine from all config sources.
    /// 
    /// Resolution order is handled by Figment's provider chain:
    /// later providers override earlier ones.
    pub fn new(cli_overrides: HashMap<String, String>) -> Result<Self, ConfigError> {
        let user_config_path = dirs::config_dir()
            .unwrap()
            .join("intellidashboard")
            .join("config")
            .join("user.toml");
        
        let project_config_path = Path::new(".intellidashboard/config/project.toml");
        let workspace_config_path = dirs::config_dir()
            .unwrap()
            .join("intellidashboard")
            .join("config")
            .join("workspace.toml");
        
        let figment = Figment::new()
            // Layer 7: Embedded defaults (lowest priority)
            .merge(Toml::string(include_str!("../../config/defaults.toml")))
            // Layer 6: Admin policies
            .merge(Toml::file(admin_policy_path))
            // Layer 5: Workspace config
            .merge(Toml::file(workspace_config_path))
            // Layer 4: Project config
            .merge(Toml::file(project_config_path))
            // Layer 3: User config
            .merge(Toml::file(user_config_path))
            // Layer 2: CLI overrides
            .merge(Serialized::defaults(cli_overrides))
            // Layer 1: Environment variables (highest priority)
            .merge(Env::prefixed("TRRUSTT_").split("_"));
        
        Ok(Self {
            figment,
            registry: ConfigRegistry::new(),
            admin_policies: HashMap::new(),
            watchers: Vec::new(),
        })
    }
    
    /// Get a fully resolved config value by its dot-notation path
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ConfigError> {
        // Check admin policy first
        if let Some(policy) = self.admin_policies.get(path) {
            if policy.enforce {
                // Return enforced value regardless of user settings
                return Ok(serde_json::from_value(policy.value.clone())?);
            }
        }
        
        self.figment.extract_inner(path)
            .map_err(|e| ConfigError::Resolution(path.into(), e.to_string()))
    }
    
    /// Set a config value at a specific scope
    pub async fn set<T: Serialize>(
        &self, 
        path: &str, 
        value: T, 
        scope: ConfigScope
    ) -> Result<(), ConfigError> {
        // Validate against registry schema
        self.registry.validate(path, &serde_json::to_value(&value)?)?;
        
        // Check if user-overridable
        if scope == ConfigScope::User {
            let entry = self.registry.get(path)
                .ok_or(ConfigError::UnknownKey(path.into()))?;
            if !entry.user_overridable {
                return Err(ConfigError::NotUserOverridable(path.into()));
            }
        }
        
        // Check admin policy block
        if let Some(policy) = self.admin_policies.get(path) {
            if policy.enforce && !policy.allow_user_override && scope != ConfigScope::Admin {
                return Err(ConfigError::AdminEnforced(path.into()));
            }
        }
        
        // Write to appropriate file
        let file_path = self.scope_path(scope);
        self.write_config(&file_path, path, value).await?;
        
        Ok(())
    }
    
    /// Reset a config key to its default at a given scope
    pub async fn reset(&self, path: &str, scope: ConfigScope) -> Result<(), ConfigError> {
        let file_path = self.scope_path(scope);
        self.remove_key(&file_path, path).await?;
        Ok(())
    }
    
    /// Export all resolved config as JSON
    pub fn export(&self) -> Result<serde_json::Value, ConfigError> {
        let mut map = serde_json::Map::new();
        for (path, _entry) in self.registry.iter() {
            // Skip sensitive values in export
            if self.registry.get(path).map(|e| e.sensitive).unwrap_or(false) {
                map.insert(path.to_string(), serde_json::Value::String("***REDACTED***".into()));
            } else {
                let value = self.get::<serde_json::Value>(path)?;
                map.insert(path.to_string(), value);
            }
        }
        Ok(serde_json::Value::Object(map))
    }
    
    /// Import config from JSON data
    pub async fn import(
        &self, 
        data: serde_json::Value, 
        scope: ConfigScope
    ) -> Result<(), ConfigError> {
        if let serde_json::Value::Object(map) = data {
            for (path, value) in map {
                self.set(&path, value, scope).await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
    Admin,
    Workspace,
    Project,
    User,
}
```

## 3. Config Registry

### 3.1 Entry Definition

```rust
// crates/config-engine/src/registry.rs

pub struct ConfigEntry {
    /// Dot-notation path: "ai.default_model"
    pub path: &'static str,
    
    /// Human-readable description for settings UI
    pub description: &'static str,
    
    /// Category for grouping in UI
    pub category: ConfigCategory,
    
    /// JSON Schema for validation
    pub schema: serde_json::Value,
    
    /// Default value
    pub default: serde_json::Value,
    
    /// Is this value encrypted at rest?
    pub sensitive: bool,
    
    /// Can users override this?
    pub user_overridable: bool,
    
    /// Is this enforced by admin policies?
    pub admin_enforceable: bool,
    
    /// Depends on other config keys?
    pub depends_on: &'static [&'static str],
    
    /// UI hints for settings editor generation
    pub ui_hint: Option<UiHint>,
}

pub enum ConfigCategory {
    Ai,
    Dax,
    Dashboard,
    Theme,
    Authentication,
    Licensing,
    Mcp,
    Ui,
    Export,
    Telemetry,
    Performance,
    Security,
    Branding,
    Internationalization,
}

pub struct UiHint {
    pub widget: UiWidget,
    pub options: Option<Vec<UiOption>>,
    pub placeholder: Option<&'static str>,
    pub advanced: bool,
}

pub enum UiWidget {
    Text,
    Number,
    Toggle,
    Select,
    Color,
    File,
    Code,
    Slider,
}
```

### 3.2 Registry Construction

The registry is built at compile time via a `build.rs` script to ensure zero runtime overhead:

```rust
// crates/config-engine/build.rs
fn main() {
    // Reads config/registry.toml
    // Generates config_registry.rs with all entries as const data
    // This gives us compile-time guarantees + zero allocation at startup
}
```

Or declaratively:

```rust
// crates/config-engine/src/registry.rs

config_entries! {
    // ── AI ────────────────────────────────────
    "ai.provider" => {
        description: "Primary AI provider for LLM operations",
        category: Ai,
        default: "openai",
        schema: json!({"type": "string", "enum": ["openai", "azure-openai", "anthropic", "ollama", "candle"]}),
        user_overridable: true,
        ui_hint: UiHint {
            widget: Select,
            options: Some(vec![
                ui_option!("openai", "OpenAI"),
                ui_option!("azure-openai", "Azure OpenAI"),
                ui_option!("anthropic", "Anthropic Claude"),
                ui_option!("ollama", "Ollama (Local)"),
                ui_option!("candle", "Candle (Embedded)"),
            ]),
            ..default()
        },
    },
    
    "ai.default_model" => {
        description: "Default AI model for generation tasks",
        category: Ai,
        default: "gpt-4o",
        schema: json!({"type": "string", "minLength": 1}),
        user_overridable: true,
        depends_on: ["ai.provider"],
    },
    
    "ai.temperature" => {
        description: "LLM temperature (0 = deterministic, 2 = creative)",
        category: Ai,
        default: 0.3,
        schema: json!({"type": "number", "minimum": 0.0, "maximum": 2.0}),
        user_overridable: true,
        ui_hint: UiHint { widget: Slider, ..default() },
    },
    
    // ── DAX ────────────────────────────────────
    "dax.complexity_level" => {
        description: "Default complexity level for generated DAX",
        category: Dax,
        default: "intermediate",
        schema: json!({"type": "string", "enum": ["beginner", "intermediate", "advanced", "expert"]}),
        user_overridable: true,
    },
    
    "dax.naming_convention" => {
        description: "Naming convention for auto-generated measures",
        category: Dax,
        default: "Pascal Case",
        schema: json!({"type": "string", "enum": ["Pascal Case", "camelCase", "snake_case", "pascalCase"]}),
        user_overridable: true,
    },
    
    "dax.comment_style" => {
        description: "Amount of comments in generated DAX",
        category: Dax,
        default: "brief",
        schema: json!({"type": "string", "enum": ["none", "brief", "detailed"]}),
        user_overridable: true,
    },
    
    "dax.max_nesting_depth" => {
        description: "Maximum nested function call depth",
        category: Dax,
        default: 5,
        schema: json!({"type": "integer", "minimum": 1, "maximum": 20}),
        user_overridable: true,
    },
    
    // ── Validation ─────────────────────────────
    "dax.validation.require_table_prefix" => {
        description: "Require table prefix on column references",
        category: Dax,
        default: false,
        schema: json!({"type": "boolean"}),
        user_overridable: true,
    },
    
    "dax.validation.disallow_volatile_functions" => {
        description: "Disallow volatile functions (NOW, TODAY, etc.)",
        category: Dax,
        default: true,
        schema: json!({"type": "boolean"}),
        user_overridable: true,
        admin_enforceable: true,
    },
    
    // ── Dashboard ──────────────────────────────
    "dashboard.default_page_size" => {
        description: "Default page aspect ratio",
        category: Dashboard,
        default: "16:9",
        schema: json!({"type": "string", "enum": ["16:9", "4:3", "letter", "custom"]}),
        user_overridable: true,
    },
    
    "dashboard.grid_density" => {
        description: "How tightly visuals are packed",
        category: Dashboard,
        default: "normal",
        schema: json!({"type": "string", "enum": ["sparse", "normal", "dense"]}),
        user_overridable: true,
    },
    
    // ── Security ───────────────────────────────
    "security.encrypt_config" => {
        description: "Encrypt sensitive config values at rest",
        category: Security,
        default: true,
        schema: json!({"type": "boolean"}),
        user_overridable: false,
        admin_enforceable: true,
    },
    
    // ── Licensing ──────────────────────────────
    "licensing.offline_grace_days" => {
        description: "Days license remains valid without phoning home",
        category: Licensing,
        default: 30,
        schema: json!({"type": "integer", "minimum": 0, "maximum": 365}),
        user_overridable: false,
        admin_enforceable: true,
    },
    
    // ── Observability ──────────────────────────
    "observability.log_level" => {
        description: "Minimum log level",
        category: Telemetry,
        default: "info",
        schema: json!({"type": "string", "enum": ["trace", "debug", "info", "warn", "error"]}),
        user_overridable: true,
    },
    
    // ── Branding ───────────────────────────────
    "branding.app_name" => {
        description: "Application display name",
        category: Branding,
        default: "IntelliDashboard Builder",
        schema: json!({"type": "string", "minLength": 1, "maxLength": 100}),
        user_overridable: false,
        admin_enforceable: true,
    },
    
    "branding.primary_color" => {
        description: "Primary brand color (hex)",
        category: Branding,
        default: "#2563EB",
        schema: json!({"type": "string", "pattern": "^#[0-9A-Fa-f]{6}$"}),
        user_overridable: false,
        admin_enforceable: true,
        ui_hint: UiHint { widget: Color, ..default() },
    },
    
    // ── MCP ────────────────────────────────────
    "mcp.default_timeout_ms" => {
        description: "Default timeout for MCP server connections",
        category: Mcp,
        default: 30000,
        schema: json!({"type": "integer", "minimum": 1000, "maximum": 300000}),
        user_overridable: true,
    },
}
```

## 4. Sensitive Value Encryption

```rust
// crates/config-engine/src/encryption.rs

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

const NONCE_LEN: usize = 12;

pub fn encrypt_value(plaintext: &str, key: &LessSafeKey) -> Result<String, ConfigError> {
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)?;
    
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let aad = Aad::empty();
    
    let mut in_out = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, aad, &mut in_out)?;
    
    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(in_out);
    
    Ok(format!("AES256-GCM:{}", base64::encode(&result)))
}

pub fn decrypt_value(encrypted: &str, key: &LessSafeKey) -> Result<String, ConfigError> {
    let payload = encrypted
        .strip_prefix("AES256-GCM:")
        .ok_or(ConfigError::InvalidEncryptedFormat)?;
    
    let data = base64::decode(payload)?;
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    
    let nonce = Nonce::assume_unique_for_key(
        nonce_bytes.try_into().map_err(|_| ConfigError::InvalidNonce)?
    );
    let aad = Aad::empty();
    
    let mut in_out = ciphertext.to_vec();
    key.open_in_place(nonce, aad, &mut in_out)?;
    
    Ok(String::from_utf8(in_out)?)
}
```

## 5. Admin Policy Enforcement

```rust
// crates/config-engine/src/admin.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPolicy {
    pub id: String,
    pub version: u32,
    pub settings: Vec<AdminPolicySetting>,
    pub signature: String,     // Ed25519 signature
    pub issued_at: String,
    pub expires_at: String,
    pub issued_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPolicySetting {
    pub path: String,
    pub enforced_value: serde_json::Value,
    pub reason: String,
    pub allow_user_override: bool,
}

impl ConfigEngine {
    /// Load admin policies from a signed policy file or URL
    pub async fn load_policies(&mut self, source: &str) -> Result<(), ConfigError> {
        let policy: AdminPolicy = if source.starts_with("http") {
            let resp = reqwest::get(source).await?;
            resp.json().await?
        } else {
            let data = tokio::fs::read_to_string(source).await?;
            serde_json::from_str(&data)?
        };
        
        // Verify signature
        verify_policy_signature(&policy)?;
        
        // Check expiration
        let expires = chrono::DateTime::parse_from_rfc3339(&policy.expires_at)?;
        if chrono::Utc::now() > expires {
            return Err(ConfigError::PolicyExpired(policy.id));
        }
        
        // Apply enforced settings
        for setting in &policy.settings {
            self.admin_policies.insert(setting.path.clone(), AdminPolicyEntry {
                value: setting.enforced_value.clone(),
                enforce: true,
                allow_user_override: setting.allow_user_override,
                reason: setting.reason.clone(),
            });
        }
        
        Ok(())
    }
}
```

## 6. Hot Reload

```rust
// crates/config-engine/src/hot_reload.rs

use notify::{Watcher, RecursiveMode, Event};

impl ConfigEngine {
    /// Start watching config files for changes.
    /// When a file changes, the Figment provider chain is rebuilt.
    pub fn watch(&mut self) -> Result<(), ConfigError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        
        let mut watcher = notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            }
        )?;
        
        let config_dir = dirs::config_dir()
            .unwrap()
            .join("intellidashboard")
            .join("config");
        
        watcher.watch(&config_dir, RecursiveMode::Recursive)?;
        
        // Spawn background task
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if event.kind.is_modify() {
                    tracing::info!("Config file changed, reloading...");
                    // Rebuild figment provider chain
                    // Emit config:reloaded event
                }
            }
        });
        
        Ok(())
    }
}
```

## 7. Usage Throughout the Codebase

```rust
// Anywhere in the app:
use config_engine::CONFIG;

// Type-safe access:
let provider: String = CONFIG.get("ai.provider")?;
let temperature: f64 = CONFIG.get("ai.temperature")?;
let complexity: ComplexityLevel = CONFIG.get("dax.complexity_level")?;
let app_name: String = CONFIG.get("branding.app_name")?;

// The CONFIG global is initialized once at startup.
// All config access is type-safe and validated.
// Nothing is ever hard-coded.
```

---

> **Document Version:** 2.0  
> **Part of:** IntelliDashboard Builder Technical Docs (Rust-Native)
