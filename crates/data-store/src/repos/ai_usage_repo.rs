// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — AI Usage Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use shared::Result;
use crate::error::DataStoreError;

#[derive(Debug, Clone)]
pub struct AiUsageRepo { pool: SqlitePool }

impl AiUsageRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    pub async fn record(
        &self, id: Uuid, user_id: Uuid, project_id: Option<Uuid>,
        provider: &str, model: &str, operation: &str,
        input_tokens: i64, output_tokens: i64, cost_usd: f64,
        cache_hit: bool, duration_ms: i64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let total = input_tokens + output_tokens;
        sqlx::query(
            "INSERT INTO ai_usage (id, project_id, user_id, provider, model, operation, input_tokens, output_tokens, total_tokens, cost_usd, cache_hit, duration_ms, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string()).bind(project_id.map(|u| u.to_string()))
        .bind(user_id.to_string()).bind(provider).bind(model).bind(operation)
        .bind(input_tokens).bind(output_tokens).bind(total)
        .bind(cost_usd).bind(cache_hit as i64).bind(duration_ms).bind(&now)
        .execute(&self.pool).await.map_err(|e| DataStoreError::from(e).into())?;
        Ok(())
    }

    pub async fn daily_cost_for_user(&self, user_id: Uuid) -> Result<f64> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        sqlx::query_scalar::<_, f64>(
            "SELECT COALESCE(SUM(cost_usd), 0.0) FROM ai_usage WHERE user_id = ? AND date(created_at) = ?"
        )
        .bind(user_id.to_string()).bind(&today)
        .fetch_one(&self.pool).await
        .map_err(|e| DataStoreError::from(e).into())
    }

    pub async fn monthly_cost_for_org(&self, org_id: Uuid) -> Result<f64> {
        sqlx::query_scalar::<_, f64>(
            r#"SELECT COALESCE(SUM(au.cost_usd), 0.0) FROM ai_usage au
               JOIN users u ON au.user_id = u.id
               JOIN organization_members om ON u.id = om.user_id
               WHERE om.org_id = ? AND strftime('%Y-%m', au.created_at) = strftime('%Y-%m', 'now')"#
        )
        .bind(org_id.to_string()).fetch_one(&self.pool).await
        .map_err(|e| DataStoreError::from(e).into())
    }
}

/// Stub for the AI Cache repo, Audit repo, ConfigHistory repo, SystemSettings repo.

#[derive(Debug, Clone)]
pub struct AiCacheRepo { pool: SqlitePool }

impl AiCacheRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
    pub async fn get(&self, _cache_key: &str) -> Result<Option<String>> { Ok(None) }
    pub async fn set(&self, _cache_key: &str, _prompt_hash: &str, _response: &str, _provider: &str, _model: &str, _operation: &str, _ttl_secs: i64) -> Result<()> { Ok(()) }
    pub async fn cleanup_expired(&self) -> Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct AuditRepo { pool: SqlitePool }

impl AuditRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
    pub async fn log(&self, _id: Uuid, _user_id: Option<Uuid>, _action: &str, _entity_type: Option<&str>, _entity_id: Option<Uuid>, _details: Option<&str>) -> Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct ConfigHistoryRepo { pool: SqlitePool }

impl ConfigHistoryRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
    pub async fn record(&self, _id: Uuid, _key: &str, _prev: Option<&str>, _new: &str, _scope: &str, _changed_by: Option<Uuid>) -> Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct SystemSettingsRepo { pool: SqlitePool }

impl SystemSettingsRepo {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
    pub async fn get(&self, _key: &str) -> Result<Option<String>> { Ok(None) }
    pub async fn set(&self, _key: &str, _value: &str, _desc: Option<&str>) -> Result<()> { Ok(()) }
}
