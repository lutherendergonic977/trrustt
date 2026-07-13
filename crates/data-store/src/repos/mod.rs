// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Repository Pattern: Public API
//
// Each repository wraps sqlx queries for a specific table.
// All queries are compile-time checked by sqlx.
// ═══════════════════════════════════════════════════════════════════════

mod ai_cache_repo;
mod ai_usage_repo;
mod audit_repo;
mod config_history_repo;
mod dashboard_repo;
mod measure_repo;
mod org_repo;
mod project_repo;
mod system_settings_repo;
mod theme_repo;
mod user_repo;
mod workspace_repo;

// ── Re-exports ────────────────────────────────────────────────────────

pub use ai_cache_repo::AiCacheRepo;
pub use ai_usage_repo::AiUsageRepo;
pub use audit_repo::AuditRepo;
pub use config_history_repo::ConfigHistoryRepo;
pub use dashboard_repo::DashboardRepo;
pub use measure_repo::MeasureRepo;
pub use org_repo::OrganizationRepo;
pub use project_repo::ProjectRepo;
pub use system_settings_repo::SystemSettingsRepo;
pub use theme_repo::ThemeRepo;
pub use user_repo::UserRepo;
pub use workspace_repo::WorkspaceRepo;
