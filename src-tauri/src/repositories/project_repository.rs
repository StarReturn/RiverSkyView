use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::project::Project;

pub struct ProjectRepository;

impl ProjectRepository {
    pub async fn insert(pool: &SqlitePool, project: &Project) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO projects
               (id, name, path, canonical_path, is_favorite, created_at, updated_at, last_opened_at, last_activity_at, removed_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        )
        .bind(&project.id)
        .bind(&project.name)
        .bind(&project.path)
        .bind(&project.canonical_path)
        .bind(project.is_favorite)
        .bind(&project.created_at)
        .bind(&project.updated_at)
        .bind(&project.last_opened_at)
        .bind(&project.last_activity_at)
        .bind(&project.removed_at)
        .execute(pool)
        .await?;

        let _ = &now;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Project>> {
        let row = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    pub async fn find_by_canonical_path(
        pool: &SqlitePool,
        canonical_path: &str,
    ) -> AppResult<Option<Project>> {
        let row = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE canonical_path = ?1 COLLATE NOCASE",
        )
        .bind(canonical_path.to_lowercase())
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn find_active_by_canonical_path(
        pool: &SqlitePool,
        canonical_path: &str,
    ) -> AppResult<Option<Project>> {
        let row = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE canonical_path = ?1 COLLATE NOCASE AND removed_at IS NULL",
        )
        .bind(canonical_path.to_lowercase())
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn find_removed_by_canonical_path(
        pool: &SqlitePool,
        canonical_path: &str,
    ) -> AppResult<Option<Project>> {
        let row = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE canonical_path = ?1 COLLATE NOCASE AND removed_at IS NOT NULL",
        )
        .bind(canonical_path.to_lowercase())
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn list_active(pool: &SqlitePool) -> AppResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE removed_at IS NULL ORDER BY is_favorite DESC, COALESCE(last_activity_at, last_opened_at, created_at) DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_removed(pool: &SqlitePool) -> AppResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE removed_at IS NOT NULL ORDER BY removed_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_favorites(pool: &SqlitePool) -> AppResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE removed_at IS NULL AND is_favorite = 1 ORDER BY COALESCE(last_activity_at, last_opened_at, created_at) DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_recent(pool: &SqlitePool, limit: u64) -> AppResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE removed_at IS NULL AND last_opened_at IS NOT NULL ORDER BY last_opened_at DESC LIMIT ?1",
        )
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn search(pool: &SqlitePool, query: &str) -> AppResult<Vec<Project>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE removed_at IS NULL AND (name LIKE ?1 OR path LIKE ?2) ORDER BY is_favorite DESC, COALESCE(last_activity_at, last_opened_at, created_at) DESC",
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn update_name(pool: &SqlitePool, id: &str, name: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query("UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(name)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::ProjectNotFound, "项目不存在"));
        }
        Ok(())
    }

    pub async fn set_favorite(pool: &SqlitePool, id: &str, favorite: bool) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result =
            sqlx::query("UPDATE projects SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3")
                .bind(favorite)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::ProjectNotFound, "项目不存在"));
        }
        Ok(())
    }

    pub async fn soft_remove(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result =
            sqlx::query("UPDATE projects SET removed_at = ?1 WHERE id = ?2 AND removed_at IS NULL")
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(
                ErrorCode::ProjectNotFound,
                "项目不存在或已移除",
            ));
        }
        Ok(())
    }

    pub async fn restore(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query("UPDATE projects SET removed_at = NULL, updated_at = ?1 WHERE id = ?2 AND removed_at IS NOT NULL")
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(
                ErrorCode::ProjectNotFound,
                "项目不存在或未移除",
            ));
        }
        Ok(())
    }

    pub async fn update_last_opened(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE projects SET last_opened_at = ?1 WHERE id = ?2")
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_last_activity(
        pool: &SqlitePool,
        id: &str,
        activity_at: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE projects SET last_activity_at = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(activity_at)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn count_active(pool: &SqlitePool) -> AppResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM projects WHERE removed_at IS NULL")
            .fetch_one(pool)
            .await?;
        Ok(row.get::<i64, _>("count"))
    }
}
