// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Dashboard Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct DashboardRepo { pool: SqlitePool }

impl DashboardRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn create(
        &self, id: Uuid, project_id: Uuid, name: &str,
        pages_json: &str, created_by: Option<Uuid>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO dashboards (id, project_id, name, pages_json, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(id.to_string()).bind(project_id.to_string()).bind(name)
            .bind(pages_json).bind(created_by.map(|u| u.to_string())).bind(&now).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DashRow>> {
        sqlx::query_as::<_, DashRow>("SELECT * FROM dashboards WHERE id = ?")
            .bind(id.to_string()).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn list_by_project(&self, project_id: Uuid) -> Result<Vec<DashRow>> {
        sqlx::query_as::<_, DashRow>("SELECT * FROM dashboards WHERE project_id = ? ORDER BY created_at DESC")
            .bind(project_id.to_string()).fetch_all(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM dashboards WHERE id = ?")
            .bind(id.to_string()).execute(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DashRow {
    pub id: String, pub project_id: String, pub name: String,
    pub description: Option<String>, pub pages_json: String,
    pub layout_config_json: Option<String>,
    pub user_intent: Option<String>, pub image_source_path: Option<String>,
    pub theme_id: Option<String>,
    pub page_count: i64, pub visual_count: i64, pub measure_count: i64,
    pub version: i64, pub parent_dashboard_id: Option<String>,
    pub created_by: Option<String>, pub created_at: String, pub updated_at: String,
}
