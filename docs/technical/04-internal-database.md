# Internal Database & Data Management — Technical Specification v2

## 1. Overview

TRRUSTT's internal data management system is a **first-class architectural component**, not an afterthought. It manages ALL application state: users, organizations, workspaces, projects, DAX measures, dashboards, AI usage, audit trails, RBAC, caching, and configuration metadata.

**Design Principles:**
- **Embedded-first** — No external servers. Everything runs in-process.
- **Single-file** — One SQLite database per installation (`~/.trrustt/data/trrustt.db`)
- **Type-safe** — `sqlx` compile-time checked queries. Zero ORM, zero runtime SQL errors.
- **ACID** — Transactions for every multi-step operation.
- **Migratable** — Versioned schema migrations. Forward-compatible. Rollback-capable.
- **Backup-friendly** — One file to copy. Built-in export/import.
- **Privacy-respecting** — All data stays local. No cloud sync unless explicitly configured.

## 2. Crate: `data-store`

```
crates/data-store/
├── Cargo.toml          (deps: sqlx, serde, serde_json, tokio, uuid, chrono, thiserror)
└── src/
    ├── lib.rs           # Public API: DataStore struct
    ├── pool.rs           # SQLite connection pool management
    ├── migrations/       # Schema migrations (versioned)
    │   ├── mod.rs
    │   ├── v001_initial.rs
    │   ├── v002_add_workspaces.rs
    │   └── ...
    ├── models/           # Domain models (FromRow structs)
    │   ├── mod.rs
    │   ├── user.rs
    │   ├── organization.rs
    │   ├── workspace.rs
    │   ├── project.rs
    │   ├── measure.rs
    │   ├── dashboard.rs
    │   ├── theme.rs
    │   ├── ai_usage.rs
    │   ├── audit.rs
    │   └── cache.rs
    ├── repos/            # Repository pattern (data access)
    │   ├── mod.rs
    │   ├── user_repo.rs
    │   ├── org_repo.rs
    │   ├── workspace_repo.rs
    │   ├── project_repo.rs
    │   ├── measure_repo.rs
    │   ├── dashboard_repo.rs
    │   ├── ai_usage_repo.rs
    │   ├── audit_repo.rs
    │   └── cache_repo.rs
    ├── backup.rs         # Export/import database
    └── error.rs          # DataStoreError enum
```

**Public API:**
```rust
// crates/data-store/src/lib.rs

pub struct DataStore {
    pool: SqlitePool,
}

impl DataStore {
    /// Open (or create) the database at the given path
    pub async fn open(path: &Path) -> Result<Self>;
    
    /// Run pending migrations
    pub async fn migrate(&self) -> Result<()>;
    
    /// Get a repository
    pub fn users(&self) -> &UserRepo;
    pub fn orgs(&self) -> &OrgRepo;
    pub fn workspaces(&self) -> &WorkspaceRepo;
    pub fn projects(&self) -> &ProjectRepo;
    pub fn measures(&self) -> &MeasureRepo;
    pub fn dashboards(&self) -> &DashboardRepo;
    pub fn themes(&self) -> &ThemeRepo;
    pub fn ai_usage(&self) -> &AiUsageRepo;
    pub fn audit(&self) -> &AuditRepo;
    pub fn cache(&self) -> &CacheRepo;
    
    /// Transaction support
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>
    where F: FnOnce(&mut Transaction) -> Future<Output = Result<T>>;
    
    /// Backup database to a file
    pub async fn backup(&self, path: &Path) -> Result<()>;
    
    /// Restore database from a backup file
    pub async fn restore(&self, path: &Path) -> Result<()>;
    
    /// Export all data as JSON (for migration/portability)
    pub async fn export_json(&self) -> Result<serde_json::Value>;
    
    /// Vacuum + optimize (periodic maintenance)
    pub async fn optimize(&self) -> Result<()>;
}
```

## 3. Complete Database Schema

### 3.1 Entity-Relationship Diagram

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────┐
│   users      │────▶│ workspace_members│◀────│  workspaces  │
│              │     │                  │     │              │
│ • id         │     │ • user_id (FK)   │     │ • id         │
│ • email      │     │ • workspace_id   │     │ • name       │
│ • name       │     │ • role           │     │ • org_id     │
│ • avatar_url │     └──────────────────┘     │ • settings   │
│ • metadata   │                              └──────┬───────┘
└──────┬───────┘                                     │
       │                                             │
       │ belongs to                                  │ contains
       ▼                                             ▼
┌──────────────┐                              ┌──────────────┐
│organizations │                              │   projects   │
│              │                              │              │
│ • id         │                              │ • id         │
│ • name       │                              │ • name       │
│ • plan_tier  │                              │ • workspace  │
│ • settings   │                              │ • pbix_path  │
│ • created_at │                              │ • ssas_info  │
└──────────────┘                              └──────┬───────┘
                                                     │
                    ┌────────────────────────────────┼────────────────────┐
                    │                                │                    │
                    ▼                                ▼                    ▼
            ┌──────────────┐                ┌──────────────┐    ┌──────────────┐
            │   measures   │                │  dashboards  │    │   ai_usage   │
            │              │                │              │    │              │
            │ • id         │                │ • id         │    │ • id         │
            │ • project_id │                │ • project_id │    │ • project_id │
            │ • name       │                │ • name       │    │ • provider   │
            │ • expression │                │ • pages_json │    │ • model      │
            │ • complexity │                │ • theme_id   │    │ • tokens     │
            └──────────────┘                └──────┬───────┘    │ • cost      │
                                                   │             └──────────────┘
                                                   │ uses
                                                   ▼
                                           ┌──────────────┐
                                           │    themes    │
                                           │              │
                                           │ • id         │
                                           │ • name       │
                                           │ • colors     │
                                           │ • fonts      │
                                           └──────────────┘
                                                    │
                    ┌───────────────────────────────┘
                    │
                    ▼
            ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
            │  audit_log   │     │  ai_cache    │     │  migrations  │
            │              │     │              │     │              │
            │ • id         │     │ • cache_key  │     │ • version    │
            │ • user_id    │     │ • prompt_hash│     │ • name       │
            │ • action     │     │ • response   │     │ • applied_at │
            │ • details    │     │ • expires_at │     └──────────────┘
            └──────────────┘     └──────────────┘
```

### 3.2 Full SQL Schema

```sql
-- ═══════════════════════════════════════════════════════════════
-- TRRUSTT Internal Database Schema v1
-- Engine: SQLite 3.45+
-- Access: sqlx (compile-time checked, async, pure Rust)
-- ═══════════════════════════════════════════════════════════════

PRAGMA journal_mode = WAL;          -- Write-Ahead Logging (better concurrency)
PRAGMA foreign_keys = ON;           -- Enforce referential integrity
PRAGMA busy_timeout = 5000;         -- 5 second busy timeout

-- ═══════════════════════════════════════════════════════════════
-- 1. IDENTITY & ACCESS CONTROL
-- ═══════════════════════════════════════════════════════════════

-- Organizations (tenants, companies, teams)
CREATE TABLE organizations (
    id TEXT PRIMARY KEY,                              -- UUID
    name TEXT NOT NULL,                               -- Display name
    slug TEXT UNIQUE NOT NULL,                        -- URL-friendly identifier
    plan_tier TEXT NOT NULL DEFAULT 'free',            -- free|pro|team|enterprise|oem
    license_expires_at TEXT,                          -- NULL = perpetual
    max_users INTEGER,                                -- NULL = unlimited
    max_workspaces INTEGER DEFAULT 1,                 -- Free: 1, Pro: 5, Team: unlimited
    settings_json TEXT NOT NULL DEFAULT '{}',          -- Org-wide settings (JSON)
    branding_json TEXT,                                -- White-label branding (JSON, OEM only)
    sso_domain TEXT,                                   -- Enterprise SSO domain (e.g., "company.com")
    sso_provider TEXT,                                 -- azure-ad|google-workspace|okta|generic-oidc
    sso_config_json TEXT,                              -- SSO configuration (encrypted fields)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Users
CREATE TABLE users (
    id TEXT PRIMARY KEY,                              -- UUID
    email TEXT UNIQUE NOT NULL,                       -- Primary identifier
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    
    -- Auth: multiple methods supported
    auth_provider TEXT NOT NULL DEFAULT 'local',       -- local|azure-ad|google|oidc|pbi-desktop
    auth_provider_user_id TEXT,                       -- External ID from SSO provider
    password_hash TEXT,                                -- bcrypt hash (only for local auth)
    
    -- SSO
    sso_tenant_id TEXT,                                -- Entra ID tenant / Google Workspace domain
    sso_last_login_at TEXT,
    
    -- PBI Desktop bridge (automatically detected)
    pbi_desktop_user_principal TEXT,                   -- UPN from PBI Desktop (e.g., user@company.com)
    pbi_desktop_last_seen_at TEXT,
    
    -- Profile
    role TEXT NOT NULL DEFAULT 'viewer',               -- super_admin|admin|designer|analyst|viewer
    is_active INTEGER NOT NULL DEFAULT 1,
    preferences_json TEXT NOT NULL DEFAULT '{}',       -- User-level preferences
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Organization membership (users belong to orgs)
CREATE TABLE organization_members (
    org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_role TEXT NOT NULL DEFAULT 'member',           -- owner|admin|member
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (org_id, user_id)
);

-- ═══════════════════════════════════════════════════════════════
-- 2. WORKSPACES & COLLABORATION
-- ═══════════════════════════════════════════════════════════════

-- Workspaces (group projects + settings)
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,                              -- UUID
    org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    settings_json TEXT NOT NULL DEFAULT '{}',          -- Workspace-level overrides
    is_default INTEGER NOT NULL DEFAULT 0,             -- Auto-created per org
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Workspace membership with RBAC
CREATE TABLE workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_role TEXT NOT NULL DEFAULT 'viewer',     -- admin|designer|analyst|viewer
    permissions_json TEXT,                             -- Fine-grained permissions override
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (workspace_id, user_id)
);

-- Workspace invitations (pending)
CREATE TABLE workspace_invitations (
    id TEXT PRIMARY KEY,                              -- UUID
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    invited_role TEXT NOT NULL DEFAULT 'viewer',
    invited_by TEXT NOT NULL REFERENCES users(id),
    token TEXT UNIQUE NOT NULL,                        -- Invitation token sent via email
    status TEXT NOT NULL DEFAULT 'pending',            -- pending|accepted|expired|revoked
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ═══════════════════════════════════════════════════════════════
-- 3. PROJECTS & POWER BI CONTEXT
-- ═══════════════════════════════════════════════════════════════

-- Projects (one per .pbix / dataset)
CREATE TABLE projects (
    id TEXT PRIMARY KEY,                              -- UUID
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    
    -- PBI Desktop connection info
    pbix_path TEXT,                                   -- Path to .pbix file
    ssas_port INTEGER,                                -- SSAS instance port
    ssas_database TEXT,                               -- Database name from SSAS
    
    -- Schema cache
    schema_hash TEXT,                                 -- SHA-256 of last discovered schema
    schema_snapshot_json TEXT,                        -- Full schema metadata (JSON)
    schema_discovered_at TEXT,
    
    -- DAX profile
    dax_complexity_default TEXT DEFAULT 'intermediate', -- Project-level DAX complexity
    dax_naming_convention TEXT,                         -- Project-level naming convention
    
    -- PBI Desktop metadata (from External Tool launch)
    pbi_desktop_version TEXT,                          -- e.g., "2.138.1004.0"
    pbi_desktop_culture TEXT,                          -- e.g., "en-US"
    
    -- Stats
    measure_count INTEGER DEFAULT 0,
    dashboard_count INTEGER DEFAULT 0,
    last_accessed_at TEXT,
    
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ═══════════════════════════════════════════════════════════════
-- 4. DAX MEASURES
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE measures (
    id TEXT PRIMARY KEY,                              -- UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    
    -- Measure identity
    name TEXT NOT NULL,                               -- Measure name
    table_name TEXT NOT NULL,                         -- Parent table in SSAS
    expression TEXT NOT NULL,                         -- DAX expression
    
    -- Metadata
    format_string TEXT,                               -- e.g., "0.00%", "#,##0", "$#,##0.00"
    description TEXT,                                 -- Human-readable description
    display_folder TEXT,                              -- Display folder in PBI field list
    data_type TEXT DEFAULT 'decimal',                 -- decimal|integer|string|datetime|boolean
    
    -- Generation metadata
    complexity TEXT NOT NULL,                         -- beginner|intermediate|advanced|expert
    is_applied INTEGER NOT NULL DEFAULT 0,            -- 1 = pushed to SSAS model
    is_ai_generated INTEGER NOT NULL DEFAULT 0,       -- 1 = generated by AI
    ai_prompt_used TEXT,                              -- The prompt that generated this
    ai_provider TEXT,                                 -- Which AI provider generated it
    ai_model TEXT,                                    -- Which model
    
    -- Validation history
    validation_status TEXT,                           -- valid|warning|error|corrected
    validation_errors_json TEXT,                       -- Last validation error details
    
    -- Version history (corrections, iterations)
    parent_measure_id TEXT REFERENCES measures(id),   -- Previous version
    version INTEGER NOT NULL DEFAULT 1,                -- Version number
    
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_measures_project ON measures(project_id);
CREATE INDEX idx_measures_name ON measures(project_id, table_name, name);
CREATE INDEX idx_measures_validated ON measures(validation_status);

-- ═══════════════════════════════════════════════════════════════
-- 5. DASHBOARDS
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE dashboards (
    id TEXT PRIMARY KEY,                              -- UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    
    -- Content
    pages_json TEXT NOT NULL,                         -- JSON: all pages with visuals
    layout_config_json TEXT,                          -- Grid density, padding, etc.
    
    -- Origin
    user_intent TEXT,                                 -- Original natural language request
    image_source_path TEXT,                           -- If created from image, the source
    
    -- Theme
    theme_id TEXT REFERENCES themes(id),              -- Which theme was applied
    
    -- Stats
    page_count INTEGER DEFAULT 1,
    visual_count INTEGER DEFAULT 0,
    measure_count INTEGER DEFAULT 0,
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_dashboard_id TEXT REFERENCES dashboards(id), -- Previous version
    
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_dashboards_project ON dashboards(project_id);

-- ═══════════════════════════════════════════════════════════════
-- 6. THEMES
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE themes (
    id TEXT PRIMARY KEY,                              -- UUID
    name TEXT NOT NULL,
    description TEXT,
    
    -- Theme data (Power BI JSON theme format)
    theme_json TEXT NOT NULL,                         -- Full theme JSON
    preview_image_path TEXT,                          -- Screenshot for theme picker
    
    -- Classification
    category TEXT DEFAULT 'custom',                   -- built-in|custom|marketplace|shared
    tags_json TEXT,                                   -- ["dark", "corporate", "minimal"]
    
    -- Sharing
    is_public INTEGER NOT NULL DEFAULT 0,             -- Available to all workspace members
    workspace_id TEXT REFERENCES workspaces(id),      -- NULL = personal theme
    
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ═══════════════════════════════════════════════════════════════
-- 7. AI USAGE & COST TRACKING
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE ai_usage (
    id TEXT PRIMARY KEY,                              -- UUID
    project_id TEXT REFERENCES projects(id),
    user_id TEXT REFERENCES users(id),
    
    -- Provider details
    provider TEXT NOT NULL,                           -- openai|azure-openai|anthropic|google|deepseek|ollama
    model TEXT NOT NULL,                              -- gpt-4o|claude-3.5-sonnet|gemini-2.5-pro|deepseek-v3
    
    -- Operation
    operation TEXT NOT NULL,                          -- dax_generate|dax_validate|dax_explain|dash_plan|vision|chat|embed
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    
    -- Cost (in USD, calculated)
    input_cost_per_1k REAL NOT NULL,
    output_cost_per_1k REAL NOT NULL,
    total_cost_usd REAL NOT NULL,
    
    -- Performance
    latency_ms INTEGER NOT NULL,
    is_cached INTEGER NOT NULL DEFAULT 0,            -- 1 = served from cache
    
    -- Tracing
    correlation_id TEXT NOT NULL,                     -- Links to tracing spans
    
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_ai_usage_project ON ai_usage(project_id);
CREATE INDEX idx_ai_usage_user ON ai_usage(user_id);
CREATE INDEX idx_ai_usage_created ON ai_usage(created_at);
CREATE INDEX idx_ai_usage_provider ON ai_usage(provider);

-- Aggregated usage view (for dashboards/analytics)
CREATE VIEW ai_usage_daily AS
SELECT 
    DATE(created_at) as date,
    user_id,
    provider,
    model,
    operation,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    SUM(total_cost_usd) as total_cost_usd,
    AVG(latency_ms) as avg_latency_ms,
    SUM(CASE WHEN is_cached THEN 1 ELSE 0 END) as cached_count
FROM ai_usage
GROUP BY DATE(created_at), user_id, provider, model, operation;

-- ═══════════════════════════════════════════════════════════════
-- 8. AI RESPONSE CACHE
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE ai_cache (
    cache_key TEXT PRIMARY KEY,                       -- SHA-256(provider + model + system_prompt + user_prompt + temperature)
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,                        -- SHA-256 of prompt only
    system_prompt_hash TEXT,
    temperature REAL NOT NULL,
    
    -- Response
    response_json TEXT NOT NULL,                      -- Full LLM response (JSON)
    token_count INTEGER NOT NULL,
    estimated_cost_usd REAL NOT NULL,
    
    -- Cache metadata
    hit_count INTEGER NOT NULL DEFAULT 1,            -- Times served from cache
    last_hit_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- TTL
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,                         -- Configurable: 1h for chat, 24h for DAX, 7d for embeddings
    is_permanent INTEGER NOT NULL DEFAULT 0           -- Never expire (e.g., embeddings)
);

CREATE INDEX idx_ai_cache_expires ON ai_cache(expires_at);
CREATE INDEX idx_ai_cache_lookup ON ai_cache(cache_key, expires_at);

-- ═══════════════════════════════════════════════════════════════
-- 9. AUDIT LOG
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,                              -- UUID
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Who
    user_id TEXT REFERENCES users(id),
    user_email TEXT,                                  -- Denormalized for deleted users
    
    -- What
    action TEXT NOT NULL,                             -- see Action enum below
    resource_type TEXT NOT NULL,                      -- measure|dashboard|project|workspace|user|org|config|theme|license
    resource_id TEXT,                                 -- UUID of affected resource
    resource_name TEXT,                               -- Denormalized for deleted resources
    
    -- Details
    change_type TEXT NOT NULL,                        -- created|updated|deleted|applied|exported|shared|revoked
    details_json TEXT,                                -- Full change details (before/after for updates)
    
    -- Context
    correlation_id TEXT NOT NULL,                     -- Links to tracing spans
    ip_address TEXT,                                  -- Client IP (if applicable)
    session_id TEXT,                                  -- Browser/CLI session
    
    -- Result
    status TEXT NOT NULL DEFAULT 'success',            -- success|failure
    error_message TEXT                                 -- Only if failure
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_action ON audit_log(action);

-- Audit log actions
-- dax.create, dax.update, dax.delete, dax.apply, dax.validate, dax.correct
-- dash.create, dash.update, dash.delete, dash.export
-- project.create, project.update, project.delete
-- workspace.create, workspace.update, workspace.delete
-- user.invite, user.join, user.leave, user.role_change
-- org.create, org.update, org.delete, org.plan_change
-- config.update, config.reset
-- theme.create, theme.update, theme.delete, theme.apply
-- license.activate, license.expire, license.upgrade
-- sso.login, sso.connect
-- backup.create, backup.restore
-- ai.request, ai.cache_hit

-- ═══════════════════════════════════════════════════════════════
-- 10. CONFIGURATION HISTORY
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE config_history (
    id TEXT PRIMARY KEY,                              -- UUID
    config_path TEXT NOT NULL,                        -- Dot-notation: "ai.default_model"
    scope TEXT NOT NULL,                              -- user|project|workspace|admin|system
    old_value TEXT,                                   -- JSON
    new_value TEXT NOT NULL,                          -- JSON
    changed_by TEXT REFERENCES users(id),
    changed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_config_history_path ON config_history(config_path, changed_at);

-- ═══════════════════════════════════════════════════════════════
-- 11. MIGRATIONS TRACKING
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,                      -- Sequential migration number
    name TEXT NOT NULL,                               -- Human-readable name
    checksum TEXT NOT NULL,                           -- SHA-256 of migration SQL
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER
);

-- ═══════════════════════════════════════════════════════════════
-- 12. SYSTEM SETTINGS (Singleton)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE system_settings (
    key TEXT PRIMARY KEY,                             -- Setting key
    value TEXT NOT NULL,                              -- JSON value
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Pre-populated with defaults:
-- installation_id: UUID (unique per installation, for telemetry)
-- database_version: current schema version
-- last_backup_at: timestamp
-- last_vacuum_at: timestamp
-- first_launch_at: timestamp
-- data_retention_days: how long to keep audit logs (default 365)
```

## 4. PBI Desktop → SSO Bridge

### 4.1 How It Works

When TRRUSTT is launched as a Power BI External Tool, PBI Desktop provides the user's identity context implicitly:

```
PBI Desktop (logged in as user@company.com via Entra ID)
        │
        │ Launches: TRRUSTT.exe --port 54321
        │
        ▼
TRRUSTT queries SSAS for user identity:
  XMLA DISCOVER → <CONNECTION_USER_NAME>DOMAIN\user</CONNECTION_USER_NAME>
        │
        ▼
TRRUSTT resolves to Entra ID UPN: user@company.com
        │
        ▼
Auto-creates/links user record:
  - auth_provider = "pbi-desktop"
  - sso_tenant_id detected from domain
  - org auto-detected from domain
        │
        ▼
RBAC applied based on org role + workspace membership
```

### 4.2 Enterprise SSO Configuration

```toml
# config/sso.toml
[auth.sso]
enabled = true
providers = ["azure-ad", "google-workspace"]

[auth.sso.azure-ad]
tenant_id = "${AZURE_TENANT_ID}"          # Or "common" for multi-tenant
client_id = "${AZURE_CLIENT_ID}"
authority = "https://login.microsoftonline.com"
scopes = ["User.Read", "offline_access"]

[auth.sso.google-workspace]
client_id = "${GOOGLE_CLIENT_ID}"
hosted_domain = "company.com"             # Restrict to specific domain

[auth.pbi-desktop-bridge]
enabled = true
auto_detect_org = true                    # Auto-create org from domain
auto_detect_role = true                   # Map PBI Desktop user to TRRUSTT role
default_role = "designer"                 # Default role for PBI Desktop users
```

## 5. Repository Pattern

Each repository provides type-safe CRUD with compile-time verified SQL:

```rust
// crates/data-store/src/repos/measure_repo.rs

impl MeasureRepo {
    /// Find all measures for a project
    pub async fn find_by_project(&self, project_id: &str) -> Result<Vec<Measure>> {
        sqlx::query_as::<_, Measure>(
            "SELECT * FROM measures WHERE project_id = ? ORDER BY table_name, name"
        )
        .bind(project_id)
        .fetch_all(&*self.pool)
        .await
    }
    
    /// Find measures by validation status
    pub async fn find_by_status(
        &self, project_id: &str, status: &str
    ) -> Result<Vec<Measure>> {
        sqlx::query_as::<_, Measure>(
            "SELECT * FROM measures WHERE project_id = ? AND validation_status = ?"
        )
        .bind(project_id)
        .bind(status)
        .fetch_all(&*self.pool)
        .await
    }
    
    /// Get measure version history
    pub async fn version_history(&self, measure_id: &str) -> Result<Vec<Measure>> {
        sqlx::query_as::<_, Measure>(
            "WITH RECURSIVE history AS (
                SELECT * FROM measures WHERE id = ?
                UNION ALL
                SELECT m.* FROM measures m
                JOIN history h ON m.parent_measure_id = h.id
            ) SELECT * FROM history ORDER BY version DESC"
        )
        .bind(measure_id)
        .fetch_all(&*self.pool)
        .await
    }
    
    /// Bulk insert from AI generation
    pub async fn insert_batch(&self, measures: &[NewMeasure]) -> Result<Vec<Measure>> {
        let mut tx = self.pool.begin().await?;
        let mut results = Vec::new();
        
        for m in measures {
            let measure = sqlx::query_as::<_, Measure>(
                "INSERT INTO measures (id, project_id, name, table_name, expression, 
                 format_string, description, complexity, is_ai_generated, ai_provider, 
                 ai_model, validation_status, created_by)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, 'pending', ?)
                 RETURNING *"
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&m.project_id)
            .bind(&m.name)
            .bind(&m.table_name)
            .bind(&m.expression)
            .bind(&m.format_string)
            .bind(&m.description)
            .bind(&m.complexity)
            .bind(&m.ai_provider)
            .bind(&m.ai_model)
            .bind(&m.created_by)
            .fetch_one(&mut *tx)
            .await?;
            results.push(measure);
        }
        
        tx.commit().await?;
        Ok(results)
    }
    
    /// Dashboard: count measures by complexity per project
    pub async fn complexity_summary(
        &self, project_id: &str
    ) -> Result<Vec<ComplexityCount>> {
        sqlx::query_as::<_, ComplexityCount>(
            "SELECT complexity, COUNT(*) as count 
             FROM measures 
             WHERE project_id = ? 
             GROUP BY complexity"
        )
        .bind(project_id)
        .fetch_all(&*self.pool)
        .await
    }
}
```

## 6. Migration Strategy

```rust
// crates/data-store/src/migrations/mod.rs

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub up: &'static str,
    pub down: Option<&'static str>,  // Optional rollback
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        up: include_str!("v001_initial.sql"),
        down: None,
    },
    Migration {
        version: 2,
        name: "add_workspaces",
        up: include_str!("v002_add_workspaces.sql"),
        down: Some(include_str!("v002_rollback.sql")),
    },
    // ... more migrations
];

impl DataStore {
    pub async fn migrate(&self) -> Result<()> {
        // Create migrations table if not exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now')),
                duration_ms INTEGER
            )"
        ).execute(&self.pool).await?;
        
        // Get applied migrations
        let applied: Vec<i32> = sqlx::query_scalar(
            "SELECT version FROM schema_migrations ORDER BY version"
        ).fetch_all(&self.pool).await?;
        
        // Apply pending migrations in transaction
        for migration in MIGRATIONS {
            if !applied.contains(&migration.version) {
                let start = std::time::Instant::now();
                let mut tx = self.pool.begin().await?;
                
                sqlx::query(migration.up).execute(&mut *tx).await?;
                
                let checksum = sha256::digest(migration.up);
                sqlx::query(
                    "INSERT INTO schema_migrations (version, name, checksum, duration_ms)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(migration.version)
                .bind(migration.name)
                .bind(&checksum)
                .bind(start.elapsed().as_millis() as i64)
                .execute(&mut *tx)
                .await?;
                
                tx.commit().await?;
                
                tracing::info!(
                    version = migration.version,
                    name = migration.name,
                    duration_ms = start.elapsed().as_millis(),
                    "Database migration applied"
                );
            }
        }
        
        Ok(())
    }
}
```

## 7. Backup & Restore

```rust
impl DataStore {
    /// Creates a consistent backup of the entire database
    pub async fn backup(&self, backup_path: &Path) -> Result<()> {
        // SQLite backup API: online, consistent, doesn't block writes
        let mut dst = SqliteConnectOptions::new()
            .filename(backup_path)
            .create_if_missing(true)
            .connect()
            .await?;
        
        sqlx::query("VACUUM INTO ?")
            .bind(backup_path.to_str().unwrap())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Export all data as portable JSON (for cross-version migration)
    pub async fn export_json(&self) -> Result<serde_json::Value> {
        let users = self.users().find_all().await?;
        let orgs = self.orgs().find_all().await?;
        let workspaces = self.workspaces().find_all().await?;
        let projects = self.projects().find_all().await?;
        // ... all tables
        
        Ok(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "users": users,
            "organizations": orgs,
            "workspaces": workspaces,
            "projects": projects,
            // ...
        }))
    }
}
```

---

> **Document Version:** 1.0  
> **Part of:** TRRUSTT Technical Docs  
> **Crate:** `crates/data-store/`
