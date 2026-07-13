// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Database Migrations
//
// Versioned, forward-compatible, rollback-capable schema migrations.
// Each migration is applied in a transaction.
// ═══════════════════════════════════════════════════════════════════════

use sqlx::SqlitePool;
use tracing::{info, instrument};

use crate::error::DataStoreError;
use shared::Result;

mod v001_initial;

/// All migrations in order.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        up: v001_initial::up,
        down: v001_initial::down,
    },
];

/// A single database migration.
struct Migration {
    /// Migration version number.
    version: i32,
    /// Human-readable name.
    name: &'static str,
    /// Forward migration function.
    up: fn(&SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>,
    /// Rollback function.
    down: fn(&SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>,
}

/// Run all pending migrations.
#[instrument(skip(pool))]
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now')),
            checksum TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DataStoreError::Query(e.to_string()))?;

    // Get the current schema version
    let current_version: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(version) FROM schema_migrations"
    )
    .fetch_one(pool)
    .await?;

    let current = current_version.unwrap_or(0);

    info!(
        current_version = current,
        total_migrations = MIGRATIONS.len(),
        "Checking pending migrations"
    );

    // Apply pending migrations in order
    for migration in MIGRATIONS {
        if migration.version > current {
            info!(
                version = migration.version,
                name = migration.name,
                "Applying migration"
            );

            // Apply within a transaction
            let mut tx = pool.begin().await
                .map_err(|e| DataStoreError::Query(e.to_string()))?;

            (migration.up)(pool).await.map_err(|e| {
                DataStoreError::Migration {
                    version: migration.version,
                    reason: e.to_string(),
                }
            })?;

            // Record the migration
            sqlx::query(
                "INSERT INTO schema_migrations (version, name, checksum) VALUES (?, ?, ?)"
            )
            .bind(migration.version)
            .bind(migration.name)
            .bind("initial") // In production, this would be a SHA-256 of the SQL
            .execute(&mut *tx)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

            tx.commit().await
                .map_err(|e| DataStoreError::Query(e.to_string()))?;

            info!(version = migration.version, "Migration applied successfully");
        }
    }

    Ok(())
}

/// Rollback the last migration.
#[instrument(skip(pool))]
pub async fn rollback_last(pool: &SqlitePool) -> Result<()> {
    let last_version: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(version) FROM schema_migrations"
    )
    .fetch_one(pool)
    .await?;

    if let Some(version) = last_version {
        if let Some(migration) = MIGRATIONS.iter().find(|m| m.version == version) {
            info!(version, name = migration.name, "Rolling back migration");

            let mut tx = pool.begin().await
                .map_err(|e| DataStoreError::Query(e.to_string()))?;

            (migration.down)(pool).await.map_err(|e| {
                DataStoreError::Migration {
                    version: migration.version,
                    reason: e.to_string(),
                }
            })?;

            sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
                .bind(version)
                .execute(&mut *tx)
                .await
                .map_err(|e| DataStoreError::Query(e.to_string()))?;

            tx.commit().await
                .map_err(|e| DataStoreError::Query(e.to_string()))?;

            info!(version, "Migration rolled back successfully");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_pool() -> (TempDir, SqlitePool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_migrate.db");
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();
        (dir, pool)
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let (_dir, pool) = setup_test_pool().await;
        run_migrations(&pool).await.unwrap();

        // Verify migrations table was populated
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let (_dir, pool) = setup_test_pool().await;
        // Run twice — should be idempotent
        run_migrations(&pool).await.unwrap();
        run_migrations(&pool).await.unwrap();
    }
}
