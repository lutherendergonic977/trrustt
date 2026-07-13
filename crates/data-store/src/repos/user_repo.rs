// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — User Repository
//
// Data access for the `users` table.
// ═══════════════════════════════════════════════════════════════════════

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::Result;
use crate::error::DataStoreError;

/// Repository for user operations.
#[derive(Debug, Clone)]
pub struct UserRepo {
    pool: SqlitePool,
}

impl UserRepo {
    /// Create a new user repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new user.
    pub async fn create(
        &self,
        id: Uuid,
        email: &str,
        display_name: &str,
        auth_provider: &str,
        role: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, display_name, auth_provider, role, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(email)
        .bind(display_name)
        .bind(auth_provider)
        .bind(role)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Find a user by their ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, email, display_name, avatar_url, auth_provider, auth_provider_user_id, role, is_active, preferences_json, created_at, updated_at FROM users WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// Find a user by their email address.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, email, display_name, avatar_url, auth_provider, auth_provider_user_id, role, is_active, preferences_json, created_at, updated_at FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// Find a user by their PBI Desktop UPN.
    pub async fn find_by_pbi_upn(&self, upn: &str) -> Result<Option<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, email, display_name, avatar_url, auth_provider, auth_provider_user_id, role, is_active, preferences_json, created_at, updated_at FROM users WHERE pbi_desktop_user_principal = ?"
        )
        .bind(upn)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// Update a user's display name.
    pub async fn update_display_name(&self, id: Uuid, name: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE users SET display_name = ?, updated_at = ? WHERE id = ?")
            .bind(name)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Update a user's role.
    pub async fn update_role(&self, id: Uuid, role: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
            .bind(role)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Deactivate a user (soft delete).
    pub async fn deactivate(&self, id: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE users SET is_active = 0, updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// List all users in an organization.
    pub async fn list_by_org(&self, org_id: Uuid) -> Result<Vec<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.email, u.display_name, u.avatar_url, u.auth_provider, u.auth_provider_user_id, u.role, u.is_active, u.preferences_json, u.created_at, u.updated_at
            FROM users u
            JOIN organization_members om ON u.id = om.user_id
            WHERE om.org_id = ?
            ORDER BY u.display_name
            "#,
        )
        .bind(org_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// List all users (paginated).
    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, email, display_name, avatar_url, auth_provider, auth_provider_user_id, role, is_active, preferences_json, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// Count users by role.
    pub async fn count_by_role(&self, role: &str) -> Result<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = ? AND is_active = 1")
            .bind(role)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())
    }
}

/// A row from the users table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    /// User ID.
    pub id: String,
    /// Email address.
    pub email: String,
    /// Display name.
    pub display_name: String,
    /// Avatar URL.
    pub avatar_url: Option<String>,
    /// Auth provider.
    pub auth_provider: String,
    /// External auth provider user ID.
    pub auth_provider_user_id: Option<String>,
    /// Global role.
    pub role: String,
    /// Whether active.
    pub is_active: i64,
    /// Preferences JSON.
    pub preferences_json: String,
    /// Created at.
    pub created_at: String,
    /// Updated at.
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, UserRepo) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();

        // Run migration
        crate::migrations::run_migrations(&pool).await.unwrap();

        (dir, UserRepo::new(pool))
    }

    #[tokio::test]
    async fn test_create_and_find_user() {
        let (_dir, repo) = setup().await;
        let id = Uuid::now_v7();

        repo.create(id, "test@example.com", "Test User", "local", "designer")
            .await
            .unwrap();

        let user = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
    }

    #[tokio::test]
    async fn test_find_by_email() {
        let (_dir, repo) = setup().await;
        let id = Uuid::now_v7();

        repo.create(id, "findme@example.com", "Find Me", "local", "viewer")
            .await
            .unwrap();

        let user = repo.find_by_email("findme@example.com").await.unwrap().unwrap();
        assert_eq!(user.display_name, "Find Me");
    }

    #[tokio::test]
    async fn test_update_role() {
        let (_dir, repo) = setup().await;
        let id = Uuid::now_v7();

        repo.create(id, "role@example.com", "Role User", "local", "viewer")
            .await
            .unwrap();

        repo.update_role(id, "admin").await.unwrap();

        let user = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(user.role, "admin");
    }

    #[tokio::test]
    async fn test_deactivate() {
        let (_dir, repo) = setup().await;
        let id = Uuid::now_v7();

        repo.create(id, "deactivate@example.com", "Deactivate Me", "local", "viewer")
            .await
            .unwrap();

        repo.deactivate(id).await.unwrap();

        let user = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(user.is_active, 0);
    }
}
