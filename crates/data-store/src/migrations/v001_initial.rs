// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Migration v001: Initial Schema
//
// Creates all core tables for the TRRUSTT internal database.
// Tables: organizations, users, org_members, workspaces,
//   workspace_members, workspace_invitations, projects,
//   measures, dashboards, themes, ai_usage, ai_cache,
//   audit_log, config_history, system_settings.
// ═══════════════════════════════════════════════════════════════════════

use sqlx::SqlitePool;
use tracing::info;

use shared::Result;

/// Apply the initial schema migration.
pub fn up(pool: &SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
    let pool = pool.clone();
    Box::pin(async move {
        info!("Creating initial database schema");

        // ── Organizations ─────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                plan_tier TEXT NOT NULL DEFAULT 'free',
                license_expires_at TEXT,
                max_users INTEGER,
                max_workspaces INTEGER DEFAULT 1,
                settings_json TEXT NOT NULL DEFAULT '{}',
                branding_json TEXT,
                sso_domain TEXT,
                sso_provider TEXT,
                sso_config_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Users ─────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL,
                avatar_url TEXT,
                auth_provider TEXT NOT NULL DEFAULT 'local',
                auth_provider_user_id TEXT,
                password_hash TEXT,
                sso_tenant_id TEXT,
                sso_last_login_at TEXT,
                pbi_desktop_user_principal TEXT,
                pbi_desktop_last_seen_at TEXT,
                role TEXT NOT NULL DEFAULT 'viewer',
                is_active INTEGER NOT NULL DEFAULT 1,
                preferences_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Organization Members ──────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS organization_members (
                org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                org_role TEXT NOT NULL DEFAULT 'member',
                joined_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (org_id, user_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Workspaces ────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                settings_json TEXT NOT NULL DEFAULT '{}',
                is_default INTEGER NOT NULL DEFAULT 0,
                created_by TEXT NOT NULL REFERENCES users(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Workspace Members ─────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_members (
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                workspace_role TEXT NOT NULL DEFAULT 'viewer',
                permissions_json TEXT,
                joined_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (workspace_id, user_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Workspace Invitations ─────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_invitations (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                email TEXT NOT NULL,
                invited_role TEXT NOT NULL DEFAULT 'viewer',
                invited_by TEXT NOT NULL REFERENCES users(id),
                token TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Projects ─────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                pbix_path TEXT,
                ssas_port INTEGER,
                ssas_database TEXT,
                schema_hash TEXT,
                schema_snapshot_json TEXT,
                schema_discovered_at TEXT,
                dax_complexity_default TEXT DEFAULT 'intermediate',
                dax_naming_convention TEXT,
                pbi_desktop_version TEXT,
                pbi_desktop_culture TEXT,
                measure_count INTEGER DEFAULT 0,
                dashboard_count INTEGER DEFAULT 0,
                last_accessed_at TEXT,
                created_by TEXT NOT NULL REFERENCES users(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Measures ─────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS measures (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                table_name TEXT NOT NULL,
                expression TEXT NOT NULL,
                format_string TEXT,
                description TEXT,
                display_folder TEXT,
                data_type TEXT DEFAULT 'decimal',
                complexity TEXT NOT NULL,
                is_applied INTEGER NOT NULL DEFAULT 0,
                is_ai_generated INTEGER NOT NULL DEFAULT 0,
                ai_prompt_used TEXT,
                ai_provider TEXT,
                ai_model TEXT,
                validation_status TEXT,
                validation_errors_json TEXT,
                parent_measure_id TEXT REFERENCES measures(id),
                version INTEGER NOT NULL DEFAULT 1,
                created_by TEXT REFERENCES users(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Dashboards ───────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dashboards (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                pages_json TEXT NOT NULL,
                layout_config_json TEXT,
                user_intent TEXT,
                image_source_path TEXT,
                theme_id TEXT REFERENCES themes(id),
                page_count INTEGER DEFAULT 1,
                visual_count INTEGER DEFAULT 0,
                measure_count INTEGER DEFAULT 0,
                version INTEGER NOT NULL DEFAULT 1,
                parent_dashboard_id TEXT REFERENCES dashboards(id),
                created_by TEXT REFERENCES users(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Themes ───────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS themes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                colors_json TEXT NOT NULL,
                typography_json TEXT NOT NULL,
                visual_defaults_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── AI Usage ─────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ai_usage (
                id TEXT PRIMARY KEY,
                project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
                user_id TEXT NOT NULL REFERENCES users(id),
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                operation TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                cost_usd REAL NOT NULL DEFAULT 0.0,
                cache_hit INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── AI Cache ─────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ai_cache (
                cache_key TEXT PRIMARY KEY,
                prompt_hash TEXT NOT NULL,
                response TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                operation TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Audit Log ────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                user_id TEXT REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                workspace_id TEXT REFERENCES workspaces(id),
                project_id TEXT REFERENCES projects(id),
                action TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                details_json TEXT,
                ip_address TEXT,
                user_agent TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Config History ───────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config_history (
                id TEXT PRIMARY KEY,
                config_key TEXT NOT NULL,
                previous_value TEXT,
                new_value TEXT NOT NULL,
                scope TEXT NOT NULL,
                changed_by TEXT REFERENCES users(id),
                changed_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── System Settings ──────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS system_settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                description TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // ── Indexes ──────────────────────────────────────────────────
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);",
            "CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);",
            "CREATE INDEX IF NOT EXISTS idx_orgs_slug ON organizations(slug);",
            "CREATE INDEX IF NOT EXISTS idx_workspaces_org ON workspaces(org_id);",
            "CREATE INDEX IF NOT EXISTS idx_projects_workspace ON projects(workspace_id);",
            "CREATE INDEX IF NOT EXISTS idx_measures_project ON measures(project_id);",
            "CREATE INDEX IF NOT EXISTS idx_measures_name ON measures(project_id, table_name, name);",
            "CREATE INDEX IF NOT EXISTS idx_measures_validated ON measures(validation_status);",
            "CREATE INDEX IF NOT EXISTS idx_dashboards_project ON dashboards(project_id);",
            "CREATE INDEX IF NOT EXISTS idx_ai_usage_user ON ai_usage(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_ai_usage_project ON ai_usage(project_id);",
            "CREATE INDEX IF NOT EXISTS idx_ai_usage_created ON ai_usage(created_at);",
            "CREATE INDEX IF NOT EXISTS idx_ai_cache_hash ON ai_cache(prompt_hash);",
            "CREATE INDEX IF NOT EXISTS idx_ai_cache_expires ON ai_cache(expires_at);",
            "CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(action);",
            "CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_log(created_at);",
        ];

        for idx_sql in indexes {
            sqlx::query(idx_sql).execute(&pool).await?;
        }

        info!("Initial schema migration complete (16 tables, 17 indexes)");
        Ok(())
    })
}

/// Rollback the initial schema migration.
pub fn down(pool: &SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
    let pool = pool.clone();
    Box::pin(async move {
        info!("Rolling back initial schema migration");
        // Drop in reverse dependency order
        let tables = vec![
            "config_history",
            "system_settings",
            "audit_log",
            "ai_cache",
            "ai_usage",
            "dashboards",
            "measures",
            "projects",
            "workspace_invitations",
            "workspace_members",
            "workspaces",
            "organization_members",
            "users",
            "organizations",
            "themes",
        ];

        for table in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {};", table))
                .execute(&pool)
                .await?;
        }

        info!("Initial schema rollback complete");
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_pool() -> (TempDir, SqlitePool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_v001.db");
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();
        (dir, pool)
    }

    #[tokio::test]
    async fn test_up_creates_all_tables() {
        let (_dir, pool) = setup_test_pool().await;
        up(&pool).await.unwrap();

        // Verify all tables exist
        let tables = vec![
            "organizations", "users", "organization_members",
            "workspaces", "workspace_members", "workspace_invitations",
            "projects", "measures", "dashboards", "themes",
            "ai_usage", "ai_cache", "audit_log",
            "config_history", "system_settings",
        ];

        for table in tables {
            let count: i64 = sqlx::query_scalar(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'", table)
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(count, 1, "Table '{}' was not created", table);
        }
    }

    #[tokio::test]
    async fn test_up_idempotent() {
        let (_dir, pool) = setup_test_pool().await;
        up(&pool).await.unwrap();
        up(&pool).await.unwrap(); // Should not error on re-run
    }

    #[tokio::test]
    async fn test_down_drops_all_tables() {
        let (_dir, pool) = setup_test_pool().await;
        up(&pool).await.unwrap();
        down(&pool).await.unwrap();

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 0, "Not all tables were dropped");
    }
}
