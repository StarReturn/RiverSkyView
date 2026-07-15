use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::editor::{EditorPreferenceInput, ProjectEditorPreference};

pub struct EditorPreferenceRepository;

impl EditorPreferenceRepository {
    pub async fn get(
        pool: &SqlitePool,
        project_id: &str,
    ) -> AppResult<Option<ProjectEditorPreference>> {
        Ok(sqlx::query_as::<_, ProjectEditorPreference>(
            "SELECT * FROM project_editor_preferences WHERE project_id = ?1",
        )
        .bind(project_id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn set(
        pool: &SqlitePool,
        project_id: &str,
        input: &EditorPreferenceInput,
    ) -> AppResult<ProjectEditorPreference> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO project_editor_preferences
               (project_id, editor_key, target_relative_path, open_mode, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(project_id) DO UPDATE SET
                 editor_key = excluded.editor_key,
                 target_relative_path = excluded.target_relative_path,
                 open_mode = excluded.open_mode,
                 updated_at = excluded.updated_at"#,
        )
        .bind(project_id)
        .bind(&input.editor_key)
        .bind(&input.target_relative_path)
        .bind(input.open_mode.as_deref().unwrap_or("default"))
        .bind(&now)
        .execute(pool)
        .await?;
        Ok(Self::get(pool, project_id)
            .await?
            .expect("upsert must return row"))
    }

    pub async fn clear(pool: &SqlitePool, project_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM project_editor_preferences WHERE project_id = ?1")
            .bind(project_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn clear_editor(pool: &SqlitePool, editor_key: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM project_editor_preferences WHERE editor_key = ?1")
            .bind(editor_key)
            .execute(pool)
            .await?;
        Ok(())
    }
}
