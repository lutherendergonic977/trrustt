// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Data Store: Connection Pool & Core API
//
// Manages SQLite connection pool and provides the main DataStore API.
// ═══════════════════════════════════════════════════════════════════════

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use tracing::{debug, error, info, instrument};

use shared::Result;

use crate::backup::BackupManager;
use crate::error::DataStoreError;
use crate::migrations;
use crate::repos::{
    AiCacheRepo, AiUsageRepo, AuditRepo, ConfigHistoryRepo, DashboardRepo,
    MeasureRepo, OrganizationRepo, ProjectRepo, SystemSettingsRepo,
    ThemeRepo, UserRepo, WorkspaceRepo,
};

/// Maximum number of concurrent SQLite connections.
const DEFAULT_POOL_SIZE: u32 = 5;

/// The central data store managing all database operations.
///
/// Holds a SQLite connection pool and provides access to all repositories.
/// All operations are async and use compile-time checked SQL via sqlx.
pub struct DataStore {
    /// SQLite connection pool.
    pool: SqlitePool,

    /// Database file path.
    db_path: PathBuf,

    /// Repository instances (lazily initialized).
    user_repo: Arc<RwLock<Option<UserRepo>>>,
    org_repo: Arc<RwLock<Option<OrganizationRepo>>>,
    workspace_repo: Arc<RwLock<Option<WorkspaceRepo>>>,
    project_repo: Arc<RwLock<Option<ProjectRepo>>>,
    measure_repo: Arc<RwLock<Option<MeasureRepo>>>,
    dashboard_repo: Arc<RwLock<Option<DashboardRepo>>>,
    theme_repo: Arc<RwLock<Option<ThemeRepo>>>,
    ai_usage_repo: Arc<RwLock<Option<AiUsageRepo>>>,
    ai_cache_repo: Arc<RwLock<Option<AiCacheRepo>>>,
    audit_repo: Arc<RwLock<Option<AuditRepo>>>,
    config_history_repo: Arc<RwLock<Option<ConfigHistoryRepo>>>,
    system_settings_repo: Arc<RwLock<Option<SystemSettingsRepo>>>,
}

impl DataStore {
    /// Open (or create) the database at the given path.
    ///
    /// This creates the SQLite file if it doesn't exist, sets up
    /// WAL mode, enables foreign keys, and configures the pool.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file.
    ///
    /// # Example
    /// ```rust,no_run
    /// use data_store::DataStore;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = DataStore::open(std::path::Path::new("trrustt.db")).await?;
    /// store.migrate().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(path))]
    pub async fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DataStoreError::Pool(format!("Failed to create DB directory: {}", e))
            })?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5))
            .log_statements(log::LevelFilter::Debug);

        let pool = SqlitePoolOptions::new()
            .max_connections(DEFAULT_POOL_SIZE)
            .connect_with(options)
            .await
            .map_err(|e| DataStoreError::Pool(format!("Failed to connect: {}", e)))?;

        // Run PRAGMAs for optimal performance
        sqlx::query("PRAGMA cache_size = -32000;") // 32 MB cache
            .execute(&pool)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

        sqlx::query("PRAGMA synchronous = NORMAL;")
            .execute(&pool)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

        sqlx::query("PRAGMA temp_store = MEMORY;")
            .execute(&pool)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

        info!(
            db_path = %path.display(),
            pool_size = DEFAULT_POOL_SIZE,
            "Database connection pool established"
        );

        Ok(Self {
            pool,
            db_path: path.to_path_buf(),
            user_repo: Arc::new(RwLock::new(None)),
            org_repo: Arc::new(RwLock::new(None)),
            workspace_repo: Arc::new(RwLock::new(None)),
            project_repo: Arc::new(RwLock::new(None)),
            measure_repo: Arc::new(RwLock::new(None)),
            dashboard_repo: Arc::new(RwLock::new(None)),
            theme_repo: Arc::new(RwLock::new(None)),
            ai_usage_repo: Arc::new(RwLock::new(None)),
            ai_cache_repo: Arc::new(RwLock::new(None)),
            audit_repo: Arc::new(RwLock::new(None)),
            config_history_repo: Arc::new(RwLock::new(None)),
            system_settings_repo: Arc::new(RwLock::new(None)),
        })
    }

    /// Run all pending schema migrations.
    ///
    /// Migrations are versioned and applied sequentially.
    /// Each migration is wrapped in a transaction.
    #[instrument(skip(self))]
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");
        migrations::run_migrations(&self.pool).await?;
        info!("Database migrations complete");
        Ok(())
    }

    /// Get a raw reference to the SQLite pool (for custom queries).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the database file path.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get the user repository.
    pub fn users(&self) -> UserRepo {
        let mut lock = self.user_repo.write();
        if lock.is_none() {
            *lock = Some(UserRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the organization repository.
    pub fn orgs(&self) -> OrganizationRepo {
        let mut lock = self.org_repo.write();
        if lock.is_none() {
            *lock = Some(OrganizationRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the workspace repository.
    pub fn workspaces(&self) -> WorkspaceRepo {
        let mut lock = self.workspace_repo.write();
        if lock.is_none() {
            *lock = Some(WorkspaceRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the project repository.
    pub fn projects(&self) -> ProjectRepo {
        let mut lock = self.project_repo.write();
        if lock.is_none() {
            *lock = Some(ProjectRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the measure repository.
    pub fn measures(&self) -> MeasureRepo {
        let mut lock = self.measure_repo.write();
        if lock.is_none() {
            *lock = Some(MeasureRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the dashboard repository.
    pub fn dashboards(&self) -> DashboardRepo {
        let mut lock = self.dashboard_repo.write();
        if lock.is_none() {
            *lock = Some(DashboardRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the theme repository.
    pub fn themes(&self) -> ThemeRepo {
        let mut lock = self.theme_repo.write();
        if lock.is_none() {
            *lock = Some(ThemeRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the AI usage repository.
    pub fn ai_usage(&self) -> AiUsageRepo {
        let mut lock = self.ai_usage_repo.write();
        if lock.is_none() {
            *lock = Some(AiUsageRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the AI cache repository.
    pub fn ai_cache(&self) -> AiCacheRepo {
        let mut lock = self.ai_cache_repo.write();
        if lock.is_none() {
            *lock = Some(AiCacheRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the audit log repository.
    pub fn audit(&self) -> AuditRepo {
        let mut lock = self.audit_repo.write();
        if lock.is_none() {
            *lock = Some(AuditRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the config history repository.
    pub fn config_history(&self) -> ConfigHistoryRepo {
        let mut lock = self.config_history_repo.write();
        if lock.is_none() {
            *lock = Some(ConfigHistoryRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Get the system settings repository.
    pub fn system_settings(&self) -> SystemSettingsRepo {
        let mut lock = self.system_settings_repo.write();
        if lock.is_none() {
            *lock = Some(SystemSettingsRepo::new(self.pool.clone()));
        }
        lock.as_ref().unwrap().clone()
    }

    /// Create a backup manager for this database.
    pub fn backup_manager(&self) -> BackupManager {
        BackupManager::new(self.pool.clone(), self.db_path.clone())
    }

    /// Vacuum and optimize the database (periodic maintenance).
    #[instrument(skip(self))]
    pub async fn optimize(&self) -> Result<()> {
        debug!("Running database optimization");
        sqlx::query("PRAGMA optimize;")
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

        sqlx::query("PRAGMA analysis_limit=1000; PRAGMA optimize;")
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::Query(e.to_string()))?;

        info!("Database optimization complete");
        Ok(())
    }

    /// Close the database connection pool gracefully.
    #[instrument(skip(self))]
    pub async fn close(self) {
        debug!("Closing database connection pool");
        self.pool.close().await;
        info!("Database connection pool closed");
    }
}

/// Run a function within a database transaction.
///
/// If the function returns `Ok`, the transaction is committed.
/// If it returns `Err`, the transaction is rolled back.
pub async fn transaction<F, Fut, T>(pool: &SqlitePool, f: F) -> Result<T>
where
    F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut tx = pool.begin().await.map_err(|e| DataStoreError::Query(e.to_string()))?;
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await.map_err(|e| DataStoreError::Query(e.to_string()))?;
            Ok(result)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, DataStore) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let store = DataStore::open(&db_path).await.unwrap();
        store.migrate().await.unwrap();
        (dir, store)
    }

    #[tokio::test]
    async fn test_open_and_migrate() {
        let (_dir, store) = setup_test_db().await;
        assert!(store.db_path().exists());
    }

    #[tokio::test]
    async fn test_optimize() {
        let (_dir, store) = setup_test_db().await;
        store.optimize().await.unwrap();
    }
}
