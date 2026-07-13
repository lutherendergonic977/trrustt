// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Theme Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct ThemeRepo { pool: SqlitePool }

impl ThemeRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn create(&self, id: Uuid, name: &str, is_default: bool, colors_json: &str, typography_json: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO themes (id, name, is_default, colors_json, typography_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(id.to_string()).bind(name).bind(is_default as i64).bind(colors_json).bind(typography_json).bind(&now).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ThemeRow>> {
        sqlx::query_as::<_, ThemeRow>("SELECT * FROM themes WHERE id = ?")
            .bind(id.to_string()).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn list_all(&self) -> Result<Vec<ThemeRow>> {
        sqlx::query_as::<_, ThemeRow>("SELECT * FROM themes ORDER BY name")
            .fetch_all(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ThemeRow {
    pub id: String, pub name: String, pub description: Option<String>,
    pub is_default: i64, pub colors_json: String,
    pub typography_json: String, pub visual_defaults_json: String,
    pub created_at: String, pub updated_at: String,
}
