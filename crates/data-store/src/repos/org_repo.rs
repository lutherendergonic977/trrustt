// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Organization Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct OrganizationRepo { pool: SqlitePool }

impl OrganizationRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn create(&self, id: Uuid, name: &str, slug: &str, plan_tier: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO organizations (id, name, slug, plan_tier, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(id.to_string()).bind(name).bind(slug).bind(plan_tier).bind(&now).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OrgRow>> {
        sqlx::query_as::<_, OrgRow>("SELECT * FROM organizations WHERE id = ?")
            .bind(id.to_string()).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<OrgRow>> {
        sqlx::query_as::<_, OrgRow>("SELECT * FROM organizations WHERE slug = ?")
            .bind(slug).fetch_optional(&self.pool).await
            .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn add_member(&self, org_id: Uuid, user_id: Uuid, role: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT OR IGNORE INTO organization_members (org_id, user_id, org_role, joined_at) VALUES (?, ?, ?, ?)")
            .bind(org_id.to_string()).bind(user_id.to_string()).bind(role).bind(&now)
            .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OrgRow {
    pub id: String, pub name: String, pub slug: String,
    pub plan_tier: String, pub license_expires_at: Option<String>,
    pub max_users: Option<i64>, pub max_workspaces: i64,
    pub settings_json: String, pub branding_json: Option<String>,
    pub sso_domain: Option<String>, pub sso_provider: Option<String>,
    pub sso_config_json: Option<String>,
    pub created_at: String, pub updated_at: String,
}
