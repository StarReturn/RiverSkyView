use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::editor::EditorInstallationRecord;

pub struct EditorInstallationRepository;

impl EditorInstallationRepository {
    pub async fn list(pool: &SqlitePool) -> AppResult<Vec<EditorInstallationRecord>> {
        Ok(sqlx::query_as::<_, EditorInstallationRecord>(
            r#"SELECT editor_key, manual_executable, detected_executable, active_source,
                      enabled, verification_status, detected_source, version,
                      last_detected_at, last_verified_at, last_error, updated_at
               FROM editor_installations ORDER BY editor_key"#,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn get(
        pool: &SqlitePool,
        editor_key: &str,
    ) -> AppResult<Option<EditorInstallationRecord>> {
        Ok(sqlx::query_as::<_, EditorInstallationRecord>(
            r#"SELECT editor_key, manual_executable, detected_executable, active_source,
                      enabled, verification_status, detected_source, version,
                      last_detected_at, last_verified_at, last_error, updated_at
               FROM editor_installations WHERE editor_key = ?1"#,
        )
        .bind(editor_key)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn upsert_detection(
        pool: &SqlitePool,
        editor_key: &str,
        executable: Option<&str>,
        source: Option<&str>,
        version: Option<&str>,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let status = if executable.is_some() {
            "valid"
        } else {
            "missing"
        };
        sqlx::query(
            r#"INSERT INTO editor_installations
               (editor_key, detected_executable, active_source, enabled, verification_status,
                detected_source, version, last_detected_at, last_verified_at, updated_at)
               VALUES (?1, ?2, 'auto', 1, ?3, ?4, ?5, ?6, ?6, ?6)
               ON CONFLICT(editor_key) DO UPDATE SET
                 detected_executable = excluded.detected_executable,
                 detected_source = excluded.detected_source,
                 version = excluded.version,
                 last_detected_at = excluded.last_detected_at,
                 verification_status = CASE
                   WHEN editor_installations.active_source = 'auto' THEN excluded.verification_status
                   ELSE editor_installations.verification_status
                 END,
                 last_verified_at = CASE
                   WHEN editor_installations.active_source = 'auto' THEN excluded.last_verified_at
                   ELSE editor_installations.last_verified_at
                 END,
                 last_error = CASE
                   WHEN editor_installations.active_source = 'auto' THEN NULL
                   ELSE editor_installations.last_error
                 END,
                 updated_at = excluded.updated_at"#,
        )
        .bind(editor_key)
        .bind(executable)
        .bind(status)
        .bind(source)
        .bind(version)
        .bind(&now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_manual(
        pool: &SqlitePool,
        editor_key: &str,
        executable: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO editor_installations
               (editor_key, manual_executable, active_source, enabled, verification_status,
                last_verified_at, updated_at)
               VALUES (?1, ?2, 'manual', 1, 'valid', ?3, ?3)
               ON CONFLICT(editor_key) DO UPDATE SET
                 manual_executable = excluded.manual_executable,
                 active_source = 'manual', enabled = 1,
                 verification_status = 'valid', last_verified_at = excluded.last_verified_at,
                 last_error = NULL, updated_at = excluded.updated_at"#,
        )
        .bind(editor_key)
        .bind(executable)
        .bind(&now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn clear_manual(pool: &SqlitePool, editor_key: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"UPDATE editor_installations
               SET manual_executable = NULL, active_source = 'auto',
                   verification_status = CASE WHEN detected_executable IS NULL THEN 'missing' ELSE 'valid' END,
                   last_verified_at = ?2, last_error = NULL, updated_at = ?2
               WHERE editor_key = ?1"#,
        )
        .bind(editor_key)
        .bind(&now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_enabled(pool: &SqlitePool, editor_key: &str, enabled: bool) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE editor_installations SET enabled = ?2, updated_at = ?3 WHERE editor_key = ?1",
        )
        .bind(editor_key)
        .bind(enabled)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_verification(
        pool: &SqlitePool,
        editor_key: &str,
        status: &str,
        error: Option<&str>,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE editor_installations SET verification_status = ?2, last_error = ?3, last_verified_at = ?4, updated_at = ?4 WHERE editor_key = ?1",
        )
        .bind(editor_key)
        .bind(status)
        .bind(error)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }
}
