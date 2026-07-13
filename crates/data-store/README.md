# data-store — Internal Database

Embedded SQLite (via sqlx) + LanceDB (vector embeddings).

## Design

- **Embedded-first** — no external servers, everything in-process
- **Single-file** — one SQLite DB per installation (`~/.trrustt/data/trrustt.db`)
- **Type-safe** — sqlx compile-time checked queries, zero ORM
- **ACID** — transactions for every multi-step operation
- **Migratable** — versioned, rollback-capable schema migrations
- **Backup-friendly** — zstd-compressed, SHA-256 verified backups

## Schema (16 tables)

organizations, users, organization_members, workspaces, workspace_members,
workspace_invitations, projects, measures, dashboards, themes,
ai_usage, ai_cache, audit_log, config_history, schema_migrations, system_settings

## Usage

```rust
use data_store::DataStore;

let store = DataStore::open("trrustt.db").await?;
store.migrate().await?;

// Repositories
let user = store.users().find_by_email("user@example.com").await?;
let measures = store.measures().list_by_project(project_id).await?;
store.ai_usage().daily_cost_for_user(user_id).await?;

// Backup
store.backup_manager().create("backups/").await?;
```
