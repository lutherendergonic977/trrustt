// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Workspace Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct WorkspaceRepo { pool: SqlitePool }

impl WorkspaceRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn create(&self, id: Uuid, org_id: Uuid, name: &str, created_by: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO workspaces (id, org_id, name, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(id.to_string()).bind(org_id.to_string()).bind(name).bind(created_by.to_string()).bind(&now).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WsRow>> {
        sqlx::query_as::<_, WsRow>("SELECT * FROM workspaces WHERE id = ?")
            .bind(id.to_string()).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn list_by_org(&self, org_id: Uuid) -> Result<Vec<WsRow>> {
        sqlx::query_as::<_, WsRow>("SELECT * FROM workspaces WHERE org_id = ? ORDER BY name")
            .bind(org_id.to_string()).fetch_all(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn add_member(&self, workspace_id: Uuid, user_id: Uuid, role: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT OR IGNORE INTO workspace_members (workspace_id, user_id, workspace_role, joined_at) VALUES (?, ?, ?, ?)")
            .bind(workspace_id.to_string()).bind(user_id.to_string()).bind(role).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WsRow {
    pub id: String, pub org_id: String, pub name: String,
    pub description: Option<String>, pub settings_json: String,
    pub is_default: i64, pub created_by: String,
    pub created_at: String, pub updated_at: String,
}
