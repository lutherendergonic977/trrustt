// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Data Store: Public API
//
// Central data management. SQLite via sqlx (compile-time checked SQL,
// zero ORM overhead) + LanceDB for vector embeddings.
//
// Design principles:
// - Embedded-first — no external servers, everything in-process.
// - Single-file — one SQLite DB per installation.
// - Type-safe — sqlx compile-time checked queries.
// - ACID — transactions for every multi-step operation.
// - Migratable — versioned, rollback-capable schema migrations.
// ═══════════════════════════════════════════════════════════════════════

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod backup;
pub mod error;
pub mod migrations;
pub mod models;
pub mod pool;
pub mod repos;
pub mod seed;

// ── Re-exports ────────────────────────────────────────────────────────

pub use backup::BackupManager;
pub use error::DataStoreError;
pub use pool::DataStore;
pub use repos::*;
