// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Data Store Error Types
// ═══════════════════════════════════════════════════════════════════════

use thiserror::Error;

/// Data store specific errors.
#[derive(Debug, Error)]
pub enum DataStoreError {
    /// A database query failed.
    #[error("Database query failed: {0}")]
    Query(String),

    /// A migration failed.
    #[error("Migration v{version} failed: {reason}")]
    Migration {
        /// Migration version.
        version: i32,
        /// Failure reason.
        reason: String,
    },

    /// Entity not found.
    #[error("{entity} not found (id: {id})")]
    NotFound {
        /// Entity type.
        entity: String,
        /// Entity ID.
        id: String,
    },

    /// A constraint was violated.
    #[error("Constraint violation on {entity}: {reason}")]
    Constraint {
        /// Entity type.
        entity: String,
        /// Violation reason.
        reason: String,
    },

    /// Backup/restore failed.
    #[error("Backup/restore failed: {0}")]
    Backup(String),

    /// Seeding failed.
    #[error("Seeding failed: {0}")]
    Seed(String),

    /// Pool connection error.
    #[error("Connection pool error: {0}")]
    Pool(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<sqlx::Error> for DataStoreError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => DataStoreError::NotFound {
                entity: "record".into(),
                id: "unknown".into(),
            },
            _ => DataStoreError::Query(e.to_string()),
        }
    }
}

impl From<DataStoreError> for shared::AppError {
    fn from(e: DataStoreError) -> Self {
        match e {
            DataStoreError::Query(msg) => shared::AppError::Database(msg),
            DataStoreError::Migration { version, reason } => {
                shared::AppError::DatabaseMigration { version, reason }
            }
            DataStoreError::NotFound { entity, id } => {
                shared::AppError::EntityNotFound { entity, id }
            }
            DataStoreError::Constraint { entity, reason } => {
                shared::AppError::DatabaseConstraint { entity, reason }
            }
            DataStoreError::Backup(msg) => shared::AppError::DatabaseBackup(msg),
            DataStoreError::Seed(msg) => shared::AppError::Database(msg),
            DataStoreError::Pool(msg) => shared::AppError::Database(msg),
            DataStoreError::Serialization(msg) => shared::AppError::Database(msg),
        }
    }
}
