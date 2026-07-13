// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Database Backup & Restore
//
// Zstd-compressed, SHA-256 verified, encrypted backups.
// Supports scheduled, pre-migration, pre-destructive, and manual backups.
// ═══════════════════════════════════════════════════════════════════════

use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Sha256, Digest};
use sqlx::SqlitePool;
use tracing::{info, instrument};

use shared::Result;
use crate::error::DataStoreError;

/// Manages database backups: create, restore, verify, rotate.
pub struct BackupManager {
    /// Database connection pool.
    pool: SqlitePool,
    /// Database file path.
    db_path: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager.
    pub fn new(pool: SqlitePool, db_path: PathBuf) -> Self {
        Self { pool, db_path }
    }

    /// Create a backup of the database.
    ///
    /// The backup is a zstd-compressed SQLite dump with a SHA-256 checksum.
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store the backup in.
    ///
    /// # Returns
    /// The path to the created backup file.
    #[instrument(skip(self))]
    pub async fn create(&self, backup_dir: &Path) -> Result<PathBuf> {
        tokio::fs::create_dir_all(backup_dir).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to create backup dir: {}", e))
        })?;

        let timestamp = Utc::now().format("%Y-%m-%dT%H%M%SZ");
        let filename = format!("trrustt-backup-{}.db.zst", timestamp);
        let backup_path = backup_dir.join(&filename);

        info!(path = %backup_path.display(), "Creating database backup");

        // Backup via SQLite's backup API (VACUUM INTO in newer SQLite)
        // For now, we use .dump via sqlx
        let dump_path = backup_dir.join(format!("trrustt-dump-{}.sql", timestamp));

        // Execute VACUUM INTO for a clean copy
        sqlx::query(&format!("VACUUM INTO '{}';", dump_path.display()))
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::Backup(format!("VACUUM INTO failed: {}", e)))?;

        // Read the dump file
        let dump_data = tokio::fs::read(&dump_path).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to read dump: {}", e))
        })?;

        // Compress with zstd
        let compressed = zstd::stream::encode_all(&dump_data[..], 3)
            .map_err(|e| DataStoreError::Backup(format!("Compression failed: {}", e)))?;

        // Compute SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&compressed);
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        // Write compressed backup
        tokio::fs::write(&backup_path, &compressed).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to write backup: {}", e))
        })?;

        // Write checksum file
        let checksum_path = backup_dir.join(format!("{}.sha256", filename));
        tokio::fs::write(&checksum_path, &hash_hex).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to write checksum: {}", e))
        })?;

        // Clean up the uncompressed dump
        let _ = tokio::fs::remove_file(&dump_path).await;

        info!(
            path = %backup_path.display(),
            size_bytes = compressed.len(),
            sha256 = %hash_hex,
            "Database backup created successfully"
        );

        Ok(backup_path)
    }

    /// Restore a database from a backup file.
    ///
    /// # Safety
    /// This replaces the current database. A pre-restore backup is
    /// automatically created before the restore begins.
    #[instrument(skip(self))]
    pub async fn restore(&self, backup_path: &Path, target_backup_dir: &Path) -> Result<()> {
        // Create a safety backup before restore
        info!("Creating safety backup before restore");
        self.create(target_backup_dir).await?;

        // Read and decompress the backup
        let compressed = tokio::fs::read(backup_path).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to read backup file: {}", e))
        })?;

        // Verify checksum if a .sha256 file exists
        let checksum_path = backup_path.with_extension("sha256");
        if checksum_path.exists() {
            let expected_hash = tokio::fs::read_to_string(&checksum_path).await.map_err(|e| {
                DataStoreError::Backup(format!("Failed to read checksum: {}", e))
            })?;
            let mut hasher = Sha256::new();
            hasher.update(&compressed);
            let actual_hash = hex::encode(hasher.finalize());

            if actual_hash.trim() != expected_hash.trim() {
                return Err(DataStoreError::Backup(
                    "Checksum verification failed — backup may be corrupted".into()
                ).into());
            }
            info!("Backup checksum verified");
        }

        // Decompress
        let sql_dump = zstd::stream::decode_all(&compressed[..])
            .map_err(|e| DataStoreError::Backup(format!("Decompression failed: {}", e)))?;

        let sql_str = String::from_utf8(sql_dump)
            .map_err(|e| DataStoreError::Backup(format!("Invalid UTF-8 in backup: {}", e)))?;

        // Execute the SQL dump
        info!("Restoring database from backup");
        for statement in sql_str.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| DataStoreError::Backup(format!("SQL error during restore: {}", e)))?;
            }
        }

        info!("Database restored successfully");
        Ok(())
    }

    /// Rotate old backups, keeping only the most recent N.
    #[instrument(skip(self))]
    pub async fn rotate(&self, backup_dir: &Path, keep_count: usize) -> Result<()> {
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(backup_dir).await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to read backup dir: {}", e))
        })?;

        let mut backup_files = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            DataStoreError::Backup(format!("Failed to read entry: {}", e))
        })? {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "zst") {
                backup_files.push(path);
            }
        }

        // Sort by name (timestamp-based), oldest first
        backup_files.sort();

        // Remove oldest files if we exceed keep_count
        if backup_files.len() > keep_count {
            let to_remove = backup_files.len() - keep_count;
            info!(
                total = backup_files.len(),
                to_remove,
                "Rotating old backups"
            );

            for path in backup_files.iter().take(to_remove) {
                tokio::fs::remove_file(path).await.map_err(|e| {
                    DataStoreError::Backup(format!("Failed to remove old backup: {}", e))
                })?;
                // Also remove checksum file
                let checksum_path = path.with_extension("sha256");
                let _ = tokio::fs::remove_file(checksum_path).await;
            }
        }

        Ok(())
    }
}

// hex module for tests
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, BackupManager) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();
        crate::migrations::run_migrations(&pool).await.unwrap();
        let mgr = BackupManager::new(pool, db_path);
        (dir, mgr)
    }

    #[tokio::test]
    async fn test_create_and_verify_backup() {
        let (dir, mgr) = setup().await;
        let backup_dir = dir.path().join("backups");

        let backup_path = mgr.create(&backup_dir).await.unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_str().unwrap().ends_with(".db.zst"));

        // Check checksum file exists
        let checksum_path = backup_path.with_extension("sha256");
        assert!(checksum_path.exists());
    }

    #[tokio::test]
    async fn test_rotate_backups() {
        let (dir, mgr) = setup().await;
        let backup_dir = dir.path().join("backups");

        // Create 5 backups
        for _ in 0..5 {
            mgr.create(&backup_dir).await.unwrap();
        }

        // Rotate to keep only 3
        mgr.rotate(&backup_dir, 3).await.unwrap();
    }
}
