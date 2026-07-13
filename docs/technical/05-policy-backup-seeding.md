# Policy Engine, Backup Strategy & Seeding — Technical Specification v2

## 1. Policy-Driven Architecture

TRRUSTT is a **policy-driven application**. Configuration defines *what values exist*. Policies define *what rules govern behavior*. Every operational concern — backups, retention, security, AI usage, access — is expressed as a policy.

### 1.1 Policy vs Config

```
CONFIG: "What is the AI temperature?"         → 0.3
POLICY: "Max AI cost per user per day?"       → $5.00
POLICY: "Require MFA for admin accounts?"     → true
POLICY: "How long to retain audit logs?"      → 365 days
POLICY: "Backup schedule?"                    → Daily at 02:00 UTC
POLICY: "How many backup copies to keep?"     → 30
POLICY: "Allowed data sources?"               → [*.pbix, localhost]
POLICY: "Max DAX complexity for Analyst role?" → intermediate
```

### 1.2 Policy Categories

```rust
// crates/config-engine/src/policy.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCategory {
    /// Backup & recovery policies
    Backup,
    /// Data retention & lifecycle
    DataRetention,
    /// Security & access control
    Security,
    /// AI usage & cost management
    AiUsage,
    /// DAX generation rules
    DaxGovernance,
    /// API & MCP rate limiting
    RateLimit,
    /// Notification & alerting
    Notification,
    /// Compliance (GDPR, SOC2, etc.)
    Compliance,
    /// Feature access (license-tier enforcement)
    FeatureAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy<T> {
    /// Unique policy identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Category
    pub category: PolicyCategory,
    
    /// The enforced value
    pub value: T,
    
    /// Default if not explicitly set
    pub default: T,
    
    /// Who can override this policy
    pub overridable_by: Vec<Role>,
    
    /// Is this policy currently enabled
    pub enabled: bool,
    
    /// Reason/documentation for this policy
    pub rationale: String,
    
    /// Audit: who set this, when
    pub set_by: Option<String>,
    pub set_at: Option<DateTime<Utc>>,
}
```

### 1.3 Complete Policy Catalog

```toml
# config/policies.toml — shipped with binary, overridable by admin

# ═══════════════════════════════════════════════════════════
# BACKUP POLICIES
# ═══════════════════════════════════════════════════════════

[policies.backup]
enabled = true
# Schedule: cron expression
schedule = "0 2 * * *"                          # Daily at 2 AM UTC
# Where to store backups
location = "~/.trrustt/backups/"
# How many backups to retain (rotation)
retention_count = 30                             # Keep last 30 days
# Maximum total backup size (MB) before cleanup
max_total_size_mb = 500
# Compression: none | zstd | gzip
compression = "zstd"
# Compression level (1-22 for zstd)
compression_level = 3
# Verify backup integrity after creation
verify_after_backup = true
# Encrypt backups at rest
encrypt_backups = true
# Backup before destructive operations (auto-backup)
auto_backup_before_delete = true
auto_backup_before_migrate = true
# Notification on backup failure
notify_on_failure = true
# Export format for portability
export_format = "sqlite-dump"                   # sqlite-dump | json | parquet

# ═══════════════════════════════════════════════════════════
# DATA RETENTION POLICIES
# ═══════════════════════════════════════════════════════════

[policies.retention]
# Audit log retention (days, 0 = forever)
audit_log_retention_days = 365
# AI usage log retention
ai_usage_retention_days = 90
# AI cache entry TTL by operation type
ai_cache_ttl = { chat = "1h", dax_generate = "24h", dax_validate = "1h", dashboard_plan = "24h", vision = "7d", embed = "30d" }
# Config history retention
config_history_retention_days = 90
# Auto-vacuum SQLite (reclaim space)
auto_vacuum_interval_days = 7
# Auto-analyze (update query planner stats)
auto_analyze_interval_days = 1

# ═══════════════════════════════════════════════════════════
# SECURITY POLICIES
# ═══════════════════════════════════════════════════════════

[policies.security]
# Session timeout (minutes, 0 = never)
session_timeout_minutes = 480                   # 8 hours
# Max failed login attempts before lockout
max_login_attempts = 5
# Lockout duration (minutes)
lockout_duration_minutes = 15
# Require MFA for specific roles
require_mfa_for_roles = ["super_admin", "admin"]
# Password minimum length (for local auth)
password_min_length = 12
# Password complexity requirements
password_require_uppercase = true
password_require_lowercase = true
password_require_number = true
password_require_special = true
# Encrypt config secrets at rest (AES-256-GCM)
encrypt_config_secrets = true
# Encrypt database at rest (SQLite SEE or SQLCipher extension)
encrypt_database = false                         # V2: SQLCipher support
# Allowed export formats
allowed_export_formats = ["pdf", "png", "pptx", "pbix", "json"]
# Disallowed export formats
disallowed_export_formats = []

# ═══════════════════════════════════════════════════════════
# AI USAGE POLICIES
# ═══════════════════════════════════════════════════════════

[policies.ai_usage]
# Daily token budget per user (0 = unlimited)
daily_token_limit = 100000
# Daily cost budget per user (USD, 0 = unlimited)
daily_cost_limit_usd = 5.00
# Monthly cost budget per organization (USD)
monthly_cost_limit_org_usd = 500.00
# Allowed AI providers
allowed_providers = ["openai", "azure-openai", "anthropic", "google", "deepseek", "ollama"]
# Disallowed AI providers
disallowed_providers = []
# Allowed models (empty = all allowed)
allowed_models = []
# Disallowed models
disallowed_models = []
# Rate limiting (requests per minute per provider)
rate_limit_rpm = { openai = 500, azure-openai = 500, anthropic = 50, google = 100, deepseek = 100, ollama = 0 }
# Enable AI response caching
enable_cache = true
# Maximum cache entry count
max_cache_entries = 10000
# Require cost estimation before AI call
require_cost_estimate = true
# Warn when single request exceeds this cost
cost_warning_threshold_usd = 1.00
# Block when single request would exceed this cost
cost_block_threshold_usd = 10.00

# ═══════════════════════════════════════════════════════════
# DAX GOVERNANCE POLICIES
# ═══════════════════════════════════════════════════════════

[policies.dax]
# Max complexity allowed per role
max_complexity_by_role = { super_admin = "expert", admin = "expert", designer = "advanced", analyst = "intermediate", viewer = "beginner" }
# Disallowed DAX functions (security/performance)
disallowed_functions = []
# Disallowed functions per role
disallowed_functions_by_role = { viewer = ["SUMX", "FILTER", "CALCULATE"] }
# Require validation before applying to model
require_validation_before_apply = true
# Max measures per batch generation
max_measures_per_batch = 25
# Require comments on measures above this complexity
require_comments_above_complexity = "intermediate"
# Naming convention enforcement
enforce_naming_convention = true
# Block measures matching these patterns (security: no INFO.* queries)
blocked_measure_patterns = ["EVALUATE", "CREATE", "DROP", "ALTER"]

# ═══════════════════════════════════════════════════════════
# ACCESS POLICIES
# ═══════════════════════════════════════════════════════════

[policies.access]
# Allowed IP ranges (empty = all allowed)
allowed_ips = []
# Blocked IP ranges
blocked_ips = []
# Allowed email domains for SSO
allowed_sso_domains = []                        # Empty = all domains
# Blocked email domains
blocked_sso_domains = []
# Allowed countries (ISO 3166-1 alpha-2)
allowed_countries = []
# Time-based access (UTC hours, 0-23)
allowed_hours_start = 0                         # 0 = 24/7
allowed_hours_end = 24
# Allowed days of week (0=Sun, 6=Sat)
allowed_days = [0, 1, 2, 3, 4, 5, 6]
# Max concurrent sessions per user
max_concurrent_sessions = 3

# ═══════════════════════════════════════════════════════════
# NOTIFICATION POLICIES
# ═══════════════════════════════════════════════════════════

[policies.notification]
# License expiry warning (days before)
license_expiry_warning_days = [30, 14, 7, 1]
# AI cost threshold warnings (percentage of budget)
cost_warning_thresholds_pct = [50, 75, 90, 100]
# Backup failure notifications
notify_backup_failure = true
# Schema change notifications (for workspace admins)
notify_schema_changes = true
# New user join notifications
notify_new_member = true
# Weekly usage summary
send_weekly_digest = true

# ═══════════════════════════════════════════════════════════
# COMPLIANCE POLICIES
# ═══════════════════════════════════════════════════════════

[policies.compliance]
# Enable GDPR data export (user can request all their data)
gdpr_export_enabled = true
# Enable right to deletion (user can delete their account + data)
gdpr_deletion_enabled = true
# Data processing region (for data residency)
data_region = "auto"                             # auto | eu | us | apac | custom
# Require data processing agreement acceptance
require_dpa_acceptance = false
# Log all data access (for SOC2/ISO27001)
log_all_data_access = false
# Anonymize IPs in logs
anonymize_ips = true
# Strip PII from telemetry
strip_pii_from_telemetry = true
```

### 1.4 Policy Engine Implementation

```rust
// crates/config-engine/src/policy_engine.rs

pub struct PolicyEngine {
    policies: HashMap<String, PolicyValue>,
    overrides: HashMap<String, PolicyOverride>,
}

impl PolicyEngine {
    /// Load policies from TOML + admin overrides
    pub fn load(config: &ConfigEngine) -> Result<Self>;
    
    /// Evaluate a policy for a given user/role
    pub fn evaluate<T: DeserializeOwned>(
        &self, 
        policy_id: &str, 
        user: &User
    ) -> Result<T>;
    
    /// Check if an action is allowed under current policies
    pub fn is_allowed(
        &self, 
        action: &PolicyAction, 
        user: &User
    ) -> Result<bool>;
    
    /// Get all violations of a user's request against policies
    pub fn check_violations(
        &self, 
        request: &PolicyCheckRequest, 
        user: &User
    ) -> Result<Vec<PolicyViolation>>;
    
    /// Override a policy (admin only)
    pub async fn set_override(
        &self, 
        policy_id: &str, 
        value: serde_json::Value, 
        set_by: &User
    ) -> Result<()>;
    
    /// Reset a policy to default
    pub async fn reset(&self, policy_id: &str) -> Result<()>;
}

// Usage example: AI cost check before making an API call
let violations = policy_engine.check_violations(
    &PolicyCheckRequest::AiRequest {
        provider: "openai",
        model: "gpt-4o",
        estimated_tokens: 4000,
        estimated_cost: 0.12,
    },
    &current_user,
)?;

if !violations.is_empty() {
    for v in &violations {
        tracing::warn!(
            policy = %v.policy_id,
            message = %v.message,
            "AI request blocked by policy"
        );
    }
    return Err(AppError::PolicyViolation(violations));
}
```

---

## 2. Backup Strategy

### 2.1 Backup Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    BACKUP STRATEGY                                │
│                                                                  │
│  TRIGGERS:                                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ • Scheduled (cron: daily at 2 AM)                           │ │
│  │ • Pre-migration (automatic before schema changes)           │ │
│  │ • Pre-destructive (automatic before deletes)                │ │
│  │ • Manual (user-initiated via CLI/GUI)                       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  BACKUP CONTENTS:                                                │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ • SQLite DB (full .dump or VACUUM INTO)                    │ │
│  │ • Config files (TOML, JSON)                                │ │
│  │ • Themes (JSON files)                                       │ │
│  │ • Prompts (text files)                                      │ │
│  │ • LanceDB vectors (parquet export)                          │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  STORAGE:                                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ • Local: ~/.trrustt/backups/YYYY-MM-DD/                    │ │
│  │ • Optional: cloud sync (S3, Azure Blob, GCS) — V2          │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ROTATION:                                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ • Keep last 30 daily backups                                │ │
│  │ • Keep last 12 monthly backups (first of month)             │ │
│  │ • Auto-delete expired backups                               │ │
│  │ • Max total backup size: 500 MB                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  VERIFICATION:                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ • SHA-256 checksum on every backup                          │ │
│  │ • Verify integrity after creation                           │ │
│  │ • Periodic restore test (monthly, to temp location)         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Backup Rust Implementation

```rust
// crates/data-store/src/backup.rs

pub struct BackupManager {
    store: Arc<DataStore>,
    config: BackupConfig,
    scheduler: BackupScheduler,
}

impl BackupManager {
    /// Create a full backup (DB + config + themes + prompts)
    pub async fn create_backup(&self, label: &str) -> Result<BackupMetadata> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H%M%S");
        let backup_dir = self.config.location.join(format!("{}-{}", timestamp, label));
        tokio::fs::create_dir_all(&backup_dir).await?;
        
        // 1. Backup SQLite (consistent, online, non-blocking)
        let db_backup = backup_dir.join("trrustt.db");
        self.store.backup(&db_backup).await?;
        
        // 2. Backup config files
        let config_backup = backup_dir.join("config");
        copy_dir(&self.config.config_dir, &config_backup).await?;
        
        // 3. Backup themes
        let themes_backup = backup_dir.join("themes");
        copy_dir(&self.config.themes_dir, &themes_backup).await?;
        
        // 4. Backup prompts
        let prompts_backup = backup_dir.join("prompts");
        copy_dir(&self.config.prompts_dir, &prompts_backup).await?;
        
        // 5. Export LanceDB vectors
        let vectors_backup = backup_dir.join("vectors.parquet");
        self.export_vectors(&vectors_backup).await?;
        
        // 6. Create archive (zstd compressed)
        let archive_path = backup_dir.with_extension("tar.zst");
        create_zstd_archive(&backup_dir, &archive_path).await?;
        
        // 7. Generate checksum
        let checksum = sha256_digest_file(&archive_path).await?;
        
        // 8. Verify integrity
        if self.config.verify_after_backup {
            self.verify_backup(&archive_path, &checksum).await?;
        }
        
        // 9. Cleanup old backups (rotation)
        self.rotate_backups().await?;
        
        // 10. Record metadata
        let metadata = BackupMetadata {
            id: Uuid::new_v4(),
            timestamp,
            label: label.to_string(),
            archive_path,
            size_bytes: tokio::fs::metadata(&archive_path).await?.len(),
            checksum,
            db_version: env!("CARGO_PKG_VERSION").to_string(),
            verified: self.config.verify_after_backup,
        };
        
        self.store.create_backup_record(&metadata).await?;
        
        Ok(metadata)
    }
    
    /// Restore from a backup
    pub async fn restore(&self, backup_id: &str) -> Result<()> {
        let metadata = self.store.get_backup_record(backup_id).await?;
        
        // 1. Verify checksum before restore
        let current_checksum = sha256_digest_file(&metadata.archive_path).await?;
        if current_checksum != metadata.checksum {
            return Err(BackupError::ChecksumMismatch);
        }
        
        // 2. Create pre-restore safety backup
        self.create_backup("pre-restore-safety").await?;
        
        // 3. Extract archive to temp location
        let temp_dir = std::env::temp_dir().join("trrustt-restore");
        extract_zstd_archive(&metadata.archive_path, &temp_dir).await?;
        
        // 4. Restore SQLite
        self.store.restore(&temp_dir.join("trrustt.db")).await?;
        
        // 5. Restore config, themes, prompts
        // ... (copy back)
        
        // 6. Re-index LanceDB vectors
        self.reindex_vectors().await?;
        
        Ok(())
    }
    
    /// Start the backup scheduler
    pub async fn start_scheduler(&self) -> Result<()> {
        let schedule = self.config.schedule.clone();
        let manager = Arc::new(self.clone());
        
        tokio::spawn(async move {
            loop {
                let next = schedule.next_occurrence();
                tokio::time::sleep_until(next).await;
                
                match manager.create_backup("scheduled").await {
                    Ok(meta) => tracing::info!(
                        backup_id = %meta.id,
                        size_mb = meta.size_bytes / 1_048_576,
                        "Scheduled backup completed"
                    ),
                    Err(e) => {
                        tracing::error!(error = %e, "Scheduled backup failed");
                        if manager.config.notify_on_failure {
                            manager.send_notification("Backup failed", &e).await;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}
```

---

## 3. Seeding Strategy

### 3.1 What Needs Seeding

```
┌─────────────────────────────────────────────────────────────────┐
│                    SEEDING MANIFEST                              │
│                                                                  │
│  SHIPPED WITH BINARY (embedded at compile time):                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 1. DEFAULT THEMES (6)                                       │ │
│  │    • Modern Dark       • Modern Light                       │ │
│  │    • Corporate Blue    • Minimal Clean                      │ │
│  │    • High Contrast     • Forest Green                       │ │
│  │                                                             │ │
│  │ 2. DEFAULT PROMPTS (full prompt library)                    │ │
│  │    • System prompts                                          │ │
│  │    • Chain definitions (DAX, Dashboard, Vision)              │ │
│  │    • Few-shot examples                                       │ │
│  │                                                             │ │
│  │ 3. DEFAULT CONFIG                                           │ │
│  │    • System defaults (config/defaults.toml)                  │ │
│  │    • Policy defaults (config/policies.toml)                  │ │
│  │                                                             │ │
│  │ 4. SYSTEM DATA (first launch)                                │ │
│  │    • Default roles + permissions                             │ │
│  │    • Installation record                                     │ │
│  │    • System settings                                         │ │
│  │                                                             │ │
│  │ 5. ONBOARDING (optional, opt-in)                             │ │
│  │    • Demo project (AdventureWorks schema)                    │ │
│  │    • Sample DAX measures                                     │ │
│  │    • Walkthrough data                                        │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Seeding Implementation

```rust
// crates/data-store/src/seed.rs

pub struct Seeder {
    store: Arc<DataStore>,
}

impl Seeder {
    /// Run all seeds for a fresh installation
    pub async fn seed_all(&self) -> Result<()> {
        let mut tx = self.store.begin_transaction().await?;
        
        self.seed_roles(&mut tx).await?;
        self.seed_system_settings(&mut tx).await?;
        self.seed_default_themes(&mut tx).await?;
        
        tx.commit().await?;
        
        // Prompts are file-based, no DB needed
        self.seed_prompts().await?;
        
        Ok(())
    }
    
    /// Seed default roles with permission matrix
    async fn seed_roles(&self, tx: &mut Transaction) -> Result<()> {
        let roles = [
            ("super_admin", "Super Administrator", vec!["*"]),
            ("admin", "Administrator", vec![
                "dashboard:*", "dax:*", "schema:*", 
                "config:read", "config:write", 
                "user:manage", "workspace:*", "project:*"
            ]),
            ("designer", "Dashboard Designer", vec![
                "dashboard:*", "dax:*", "schema:read",
                "project:read", "project:write"
            ]),
            ("analyst", "Data Analyst", vec![
                "dashboard:read", "dashboard:create",
                "dax:create", "dax:read",
                "schema:read", "project:read"
            ]),
            ("viewer", "Viewer", vec![
                "dashboard:read", "schema:read", "project:read"
            ]),
        ];
        
        for (name, description, permissions) in &roles {
            sqlx::query(
                "INSERT OR IGNORE INTO system_settings (key, value, description)
                 VALUES (?, ?, ?)"
            )
            .bind(format!("roles.{}", name))
            .bind(serde_json::json!({
                "name": name,
                "description": description,
                "permissions": permissions,
                "is_system": true,
            }).to_string())
            .bind(format!("System role: {}", description))
            .execute(&mut *tx)
            .await?;
        }
        
        Ok(())
    }
    
    /// Seed default themes into the database
    async fn seed_default_themes(&self, tx: &mut Transaction) -> Result<()> {
        let built_in_themes = [
            ("modern-dark", include_str!("../../themes/modern-dark.json")),
            ("modern-light", include_str!("../../themes/modern-light.json")),
            ("corporate-blue", include_str!("../../themes/corporate-blue.json")),
            ("minimal-clean", include_str!("../../themes/minimal-clean.json")),
            ("high-contrast", include_str!("../../themes/high-contrast.json")),
            ("forest-green", include_str!("../../themes/forest-green.json")),
        ];
        
        for (name, theme_json) in &built_in_themes {
            sqlx::query(
                "INSERT OR IGNORE INTO themes (id, name, description, theme_json, category, is_public)
                 VALUES (?, ?, ?, ?, 'built-in', 1)"
            )
            .bind(Uuid::new_v4().to_string())
            .bind(name)
            .bind(format!("Built-in theme: {}", name))
            .bind(*theme_json)
            .execute(&mut *tx)
            .await?;
        }
        
        Ok(())
    }
}
```

---

## 4. Config Naming Convention: TRRUSTT_ Prefix

All environment variables, config keys, and CLI args use the `TRRUSTT_` prefix:

```
❌ OLD: INTELLI_AI_PROVIDER=openai
✅ NEW: TRRUSTT_AI_PROVIDER=openai

❌ OLD: ~/.intellidashboard/config/
✅ NEW: ~/.trrustt/config/

❌ OLD: IntelliDashboard.exe
✅ NEW: TRRUSTT.exe

ENV VARS:
  TRRUSTT_AI_PROVIDER=openai
  TRRUSTT_AI_DEFAULT_MODEL=gpt-4o
  TRRUSTT_DAX_COMPLEXITY_LEVEL=advanced
  TRRUSTT_LOG_LEVEL=debug
  TRRUSTT_CONFIG_DIR=/custom/path
  TRRUSTT_LICENSE_KEY=xxxx-xxxx-xxxx-xxxx

CONFIG FILES:
  ~/.trrustt/config/user.toml
  ~/.trrustt/config/workspace.toml
  ~/.trrustt/config/admin-policies.toml
  ~/.trrustt/config/policies.toml

CLI ARGS:
  trrustt --ai-provider openai --dax-complexity advanced
```

**Fix needed everywhere:** The master plan, AGENTS.md, architecture docs, and README still reference `INTELLI_` in the config engine section. Must find and replace all occurrences.

---

## 5. Why sqlx, NOT an ORM

| Concern | ORM (Diesel/SeaORM) | sqlx |
|---|---|---|
| **Schema ownership** | ORM owns schema → you follow its rules | You own the schema → full SQL control |
| **Compile-time checks** | Some (Diesel) | ✅ Every query checked against real DB |
| **Runtime overhead** | Yes (abstraction layers) | ✅ Zero (direct SQL execution) |
| **Complex queries** | Fight the ORM | ✅ Write SQL directly |
| **Migrations** | Built-in | ✅ `sqlx migrate` |
| **Async** | SeaORM yes, Diesel no | ✅ Fully async (tokio) |
| **Learning curve** | ORM-specific DSL | ✅ Just SQL |
| **Debugging** | Generated SQL hard to trace | ✅ Exact query you wrote |

**sqlx is the Rust community's answer to "we want type-safe SQL without an ORM."** It gives you Prisma-level type safety with zero runtime penalty.

---

> **Document Version:** 1.0  
> **Part of:** TRRUSTT Technical Docs
