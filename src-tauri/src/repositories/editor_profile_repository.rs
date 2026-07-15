use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::editor::{EditorProfile, EditorProfileInput};

pub struct EditorProfileRepository;

impl EditorProfileRepository {
    pub async fn list(
        pool: &SqlitePool,
        project_id: Option<&str>,
    ) -> AppResult<Vec<EditorProfile>> {
        let rows = if let Some(project_id) = project_id {
            sqlx::query_as::<_, EditorProfile>(
                r#"SELECT id, project_id, name, executable, args_json, working_directory,
                          sort_order, enabled
                   FROM launch_profiles
                   WHERE tool_kind = 'editor' AND (project_id IS NULL OR project_id = ?1)
                   ORDER BY sort_order, name"#,
            )
            .bind(project_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, EditorProfile>(
                r#"SELECT id, project_id, name, executable, args_json, working_directory,
                          sort_order, enabled
                   FROM launch_profiles
                   WHERE tool_kind = 'editor' AND project_id IS NULL
                   ORDER BY sort_order, name"#,
            )
            .fetch_all(pool)
            .await?
        };
        Ok(rows)
    }

    pub async fn find(pool: &SqlitePool, id: &str) -> AppResult<Option<EditorProfile>> {
        Ok(sqlx::query_as::<_, EditorProfile>(
            r#"SELECT id, project_id, name, executable, args_json, working_directory,
                      sort_order, enabled
               FROM launch_profiles WHERE id = ?1 AND tool_kind = 'editor'"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn create(pool: &SqlitePool, input: &EditorProfileInput) -> AppResult<EditorProfile> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let args_json = serde_json::to_string(&input.args)?;
        sqlx::query(
            r#"INSERT INTO launch_profiles
               (id, project_id, name, tool_kind, action_kind, executable, args_json,
                working_directory, sort_order, enabled, created_at, updated_at)
               VALUES (?1, ?2, ?3, 'editor', 'open_project', ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        )
        .bind(&id)
        .bind(&input.project_id)
        .bind(input.name.trim())
        .bind(input.executable.trim())
        .bind(&args_json)
        .bind(&input.working_directory)
        .bind(input.sort_order.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
        Self::find(pool, &id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::Internal, "创建编辑器配置后读取失败"))
    }

    pub async fn update(
        pool: &SqlitePool,
        id: &str,
        input: &EditorProfileInput,
    ) -> AppResult<EditorProfile> {
        let now = Utc::now().to_rfc3339();
        let args_json = serde_json::to_string(&input.args)?;
        let result = sqlx::query(
            r#"UPDATE launch_profiles
               SET project_id = ?1, name = ?2, executable = ?3, args_json = ?4,
                   working_directory = ?5, sort_order = ?6, enabled = ?7, updated_at = ?8
               WHERE id = ?9 AND tool_kind = 'editor'"#,
        )
        .bind(&input.project_id)
        .bind(input.name.trim())
        .bind(input.executable.trim())
        .bind(&args_json)
        .bind(&input.working_directory)
        .bind(input.sort_order.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::EditorNotFound, "编辑器配置不存在"));
        }
        Self::find(pool, id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::EditorNotFound, "编辑器配置不存在"))
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let result =
            sqlx::query("DELETE FROM launch_profiles WHERE id = ?1 AND tool_kind = 'editor'")
                .bind(id)
                .execute(pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::EditorNotFound, "编辑器配置不存在"));
        }
        Ok(())
    }
}
