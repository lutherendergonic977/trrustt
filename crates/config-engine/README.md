# config-engine — Configuration Engine

Central configuration system for TRRUSTT. Nothing is hard-coded.

## 7-Layer Resolution (highest priority first)

1. Environment variables (`TRRUSTT_` prefix)
2. CLI arguments
3. User config (`~/.trrustt/config/user.toml`)
4. Project config (`.trrustt/config/project.toml`)
5. Workspace config (`~/.trrustt/config/workspace.toml`)
6. Admin-enforced policies
7. System defaults (embedded at compile time)

## Modules

- **`engine.rs`** — `ConfigEngine`: layered resolution, get/set/reset/export/import
- **`registry.rs`** — `ConfigRegistry`: all known config keys with validation metadata
- **`layers.rs`** — `ConfigScope`: Admin, Workspace, Project, User
- **`encryption.rs`** — `EncryptionEngine`: AES-256-GCM for sensitive values
- **`policy.rs`** — `PolicyEngine` + `PolicyDefinition`: 8 policy categories
- **`validation.rs`** — `ConfigValidator`: type, range, pattern checks

## Usage

```rust
use config_engine::ConfigEngine;
use std::collections::HashMap;

let engine = ConfigEngine::new(HashMap::new())?;

// Get resolved value
let provider: String = engine.get("ai.default_provider")?;

// Set user-level override
engine.set("ai.default_provider", "anthropic", ConfigScope::User).await?;

// Export all (redacted)
let all_config = engine.export()?;

// Encrypt a secret
let encrypted = engine.encryption().encrypt("sk-abc123")?;
let decrypted = engine.encryption().decrypt(&encrypted)?;
```

## Dependencies

- `shared` — types and errors
- `figment` — layered config
- `ring` + `aes-gcm` — encryption
