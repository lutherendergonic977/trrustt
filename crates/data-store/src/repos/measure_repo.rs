// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Measure Repository
// ═══════════════════════════════════════════════════════════════════════

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::Result;
use crate::error::DataStoreError;

/// Repository for measure operations.
#[derive(Debug, Clone)]
pub struct MeasureRepo {
    pool: SqlitePool,
}

impl MeasureRepo {
    /// Create a new measure repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new measure.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        id: Uuid,
        project_id: Uuid,
        name: &str,
        table_name: &str,
        expression: &str,
        complexity: &str,
        is_ai_generated: bool,
        ai_provider: Option<&str>,
        ai_model: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO measures (id, project_id, name, table_name, expression, complexity, is_ai_generated, ai_provider, ai_model, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(project_id.to_string())
        .bind(name)
        .bind(table_name)
        .bind(expression)
        .bind(complexity)
        .bind(is_ai_generated as i64)
        .bind(ai_provider)
        .bind(ai_model)
        .bind(created_by.map(|u| u.to_string()))
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Find a measure by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MeasureRow>> {
        sqlx::query_as::<_, MeasureRow>(
            "SELECT * FROM measures WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// List all measures in a project.
    pub async fn list_by_project(&self, project_id: Uuid) -> Result<Vec<MeasureRow>> {
        sqlx::query_as::<_, MeasureRow>(
            "SELECT * FROM measures WHERE project_id = ? ORDER BY table_name, name"
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// List AI-generated measures in a project.
    pub async fn list_ai_generated(&self, project_id: Uuid) -> Result<Vec<MeasureRow>> {
        sqlx::query_as::<_, MeasureRow>(
            "SELECT * FROM measures WHERE project_id = ? AND is_ai_generated = 1 ORDER BY created_at DESC"
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())
    }

    /// Update a measure's expression.
    pub async fn update_expression(&self, id: Uuid, expression: &str, version: i32) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE measures SET expression = ?, version = ?, updated_at = ? WHERE id = ?"
        )
        .bind(expression)
        .bind(version)
        .bind(&now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Mark a measure as applied to the SSAS model.
    pub async fn mark_applied(&self, id: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE measures SET is_applied = 1, updated_at = ? WHERE id = ?"
        )
        .bind(&now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Update validation status.
    pub async fn update_validation(
        &self,
        id: Uuid,
        status: &str,
        errors_json: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE measures SET validation_status = ?, validation_errors_json = ?, updated_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(errors_json)
        .bind(&now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Delete a measure.
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM measures WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())?;

        Ok(())
    }

    /// Count measures in a project.
    pub async fn count_by_project(&self, project_id: Uuid) -> Result<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM measures WHERE project_id = ?")
            .bind(project_id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataStoreError::from(e).into())
    }
}

/// A row from the measures table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MeasureRow {
    /// Measure ID.
    pub id: String,
    /// Parent project ID.
    pub project_id: String,
    /// Measure name.
    pub name: String,
    /// Parent table name.
    pub table_name: String,
    /// DAX expression.
    pub expression: String,
    /// Format string.
    pub format_string: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Display folder.
    pub display_folder: Option<String>,
    /// Data type.
    pub data_type: String,
    /// Complexity level.
    pub complexity: String,
    /// Whether applied to model.
    pub is_applied: i64,
    /// Whether AI-generated.
    pub is_ai_generated: i64,
    /// AI prompt used.
    pub ai_prompt_used: Option<String>,
    /// AI provider.
    pub ai_provider: Option<String>,
    /// AI model.
    pub ai_model: Option<String>,
    /// Validation status.
    pub validation_status: Option<String>,
    /// Validation errors JSON.
    pub validation_errors_json: Option<String>,
    /// Parent measure ID.
    pub parent_measure_id: Option<String>,
    /// Version number.
    pub version: i64,
    /// Created by user ID.
    pub created_by: Option<String>,
    /// Created at.
    pub created_at: String,
    /// Updated at.
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, MeasureRepo, Uuid) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();
        crate::migrations::run_migrations(&pool).await.unwrap();

        // Create a test organization, user, workspace, project
        let org_id = Uuid::now_v7();
        let user_id = Uuid::now_v7();
        let ws_id = Uuid::now_v7();
        let proj_id = Uuid::now_v7();
        let now = Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO organizations (id, name, slug) VALUES (?, 'Test Org', 'test-org')")
            .bind(org_id.to_string())
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO users (id, email, display_name) VALUES (?, 'test@test.com', 'Test')")
            .bind(user_id.to_string())
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO workspaces (id, org_id, name, created_by) VALUES (?, ?, 'Test WS', ?)")
            .bind(ws_id.to_string()).bind(org_id.to_string()).bind(user_id.to_string())
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO projects (id, workspace_id, name, created_by) VALUES (?, ?, 'Test Proj', ?)")
            .bind(proj_id.to_string()).bind(ws_id.to_string()).bind(user_id.to_string())
            .execute(&pool).await.unwrap();

        (dir, MeasureRepo::new(pool), proj_id)
    }

    #[tokio::test]
    async fn test_create_and_find_measure() {
        let (_dir, repo, proj_id) = setup().await;
        let id = Uuid::now_v7();

        repo.create(
            id, proj_id, "Total Sales", "Sales",
            "SUM('Sales'[Amount])", "beginner",
            true, Some("openai"), Some("gpt-4o"), None,
        ).await.unwrap();

        let m = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(m.name, "Total Sales");
        assert_eq!(m.is_ai_generated, 1);
    }
}
