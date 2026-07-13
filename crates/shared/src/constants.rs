// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Shared Constants
//
// All application-wide constants. Nothing hard-coded elsewhere.
// ═══════════════════════════════════════════════════════════════════════

/// Product name as displayed in UI, docs, splash screens.
pub const PRODUCT_NAME_DISPLAY: &str = "trRUSTt";

/// Product name in code, config keys, env vars, binary names.
pub const PRODUCT_NAME_CODE: &str = "TRRUSTT";

/// Internal code name.
pub const CODE_NAME: &str = "Athena";

/// Tagline.
pub const TAGLINE: &str = "trRUSTt your data. One binary. Infinite dashboards.";

/// Binary filename (without extension).
pub const BINARY_NAME: &str = "TRRUSTT";

/// Binary filename with Windows extension.
pub const BINARY_NAME_WIN: &str = "TRRUSTT.exe";

/// Environment variable prefix for all config overrides.
pub const ENV_PREFIX: &str = "TRRUSTT_";

/// Config directory (relative to user's home).
pub const CONFIG_DIR: &str = ".trrustt";

/// External Tool JSON registration filename.
pub const EXTERNAL_TOOL_JSON: &str = "TRRUSTT.json";

/// External Tool registration directory (relative to %APPDATA%).
pub const EXTERNAL_TOOL_DIR: &str = r"Microsoft\Power BI Desktop\External Tools";

/// Supported AI providers.
pub const AI_PROVIDERS: &[&str] = &[
    "openai",
    "azure-openai",
    "anthropic",
    "google",
    "deepseek",
    "ollama",
];

/// DAX complexity levels (ascending).
pub const DAX_COMPLEXITY_LEVELS: &[&str] = &[
    "beginner",
    "intermediate",
    "advanced",
    "expert",
];

/// User roles (ascending privilege).
pub const USER_ROLES: &[&str] = &[
    "viewer",
    "analyst",
    "designer",
    "admin",
    "super_admin",
];

/// Organization plan tiers.
pub const PLAN_TIERS: &[&str] = &[
    "free",
    "pro",
    "team",
    "enterprise",
    "oem",
];

/// Supported SSO providers.
pub const SSO_PROVIDERS: &[&str] = &[
    "local",
    "azure-ad",
    "google-workspace",
    "okta",
    "generic-oidc",
    "pbi-desktop",
];

/// Auth providers for user accounts.
pub const AUTH_PROVIDERS: &[&str] = &[
    "local",
    "azure-ad",
    "google",
    "oidc",
    "pbi-desktop",
];

/// Default theme names shipped with the product.
pub const DEFAULT_THEMES: &[&str] = &[
    "default",
    "dark",
    "corporate",
    "minimal",
    "vibrant",
    "accessible",
];

/// Target binary size in bytes (35 MB).
pub const MAX_BINARY_SIZE_BYTES: u64 = 35_000_000;

/// Target binary size in bytes (strict).
pub const TARGET_BINARY_SIZE_BYTES: u64 = 30_000_000;

/// SQLite database filename.
pub const DATABASE_FILENAME: &str = "trrustt.db";

/// LanceDB directory name.
pub const LANCEDB_DIR: &str = "lancedb";

/// Default SSAS port (0 = auto-detect).
pub const DEFAULT_SSAS_PORT: u16 = 0;

/// Version of the External Tool JSON schema.
pub const EXTERNAL_TOOL_SCHEMA_VERSION: &str = "1.0";

/// Maximum workspace name length.
pub const MAX_WORKSPACE_NAME_LEN: usize = 128;

/// Maximum project name length.
pub const MAX_PROJECT_NAME_LEN: usize = 256;

/// Maximum measure name length.
pub const MAX_MEASURE_NAME_LEN: usize = 256;

/// Maximum DAX expression length.
pub const MAX_DAX_EXPRESSION_LEN: usize = 65536;

/// Maximum dashboard name length.
pub const MAX_DASHBOARD_NAME_LEN: usize = 256;

/// Repository URL.
pub const REPO_URL: &str = "https://github.com/trrustt/builder";

/// Documentation URL.
pub const DOCS_URL: &str = "https://trrustt.dev/docs";

/// License server URL (placeholder).
pub const LICENSE_SERVER_URL: &str = "https://license.trrustt.dev";
