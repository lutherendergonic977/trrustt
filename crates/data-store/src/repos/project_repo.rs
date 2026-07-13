// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Project Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct ProjectRepo { pool: SqlitePool }

impl ProjectRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn create(&self, id: Uuid, workspace_id: Uuid, name: &str, created_by: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO projects (id, workspace_id, name, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(id.to_string()).bind(workspace_id.to_string()).bind(name).bind(created_by.to_string()).bind(&now).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ProjRow>> {
        sqlx::query_as::<_, ProjRow>("SELECT * FROM projects WHERE id = ?")
            .bind(id.to_string()).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn list_by_workspace(&self, workspace_id: Uuid) -> Result<Vec<ProjRow>> {
        sqlx::query_as::<_, ProjRow>("SELECT * FROM projects WHERE workspace_id = ? ORDER BY name")
            .bind(workspace_id.to_string()).fetch_all(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn update_schema_snapshot(&self, id: Uuid, schema_json: &str, schema_hash: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE projects SET schema_snapshot_json = ?, schema_hash = ?, schema_discovered_at = ?, updated_at = ? WHERE id = ?")
            .bind(schema_json).bind(schema_hash).bind(&now).bind(&now).bind(id.to_string())
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn touch(&self, id: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE projects SET last_accessed_at = ?, updated_at = ? WHERE id = ?")
            .bind(&now).bind(&now).bind(id.to_string())
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjRow {
    pub id: String, pub workspace_id: String, pub name: String,
    pub description: Option<String>, pub pbix_path: Option<String>,
    pub ssas_port: Option<i64>, pub ssas_database: Option<String>,
    pub schema_hash: Option<String>, pub schema_snapshot_json: Option<String>,
    pub schema_discovered_at: Option<String>,
    pub dax_complexity_default: String, pub dax_naming_convention: Option<String>,
    pub pbi_desktop_version: Option<String>, pub pbi_desktop_culture: Option<String>,
    pub measure_count: i64, pub dashboard_count: i64,
    pub last_accessed_at: Option<String>,
    pub created_by: String, pub created_at: String, pub updated_at: String,
}
