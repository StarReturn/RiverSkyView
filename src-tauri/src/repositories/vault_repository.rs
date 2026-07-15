use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::vault::{VaultEntry, VaultListItem};

pub struct VaultRepository;

impl VaultRepository {
    pub async fn insert(pool: &SqlitePool, entry: &VaultEntry) -> AppResult<()> {
        sqlx::query(
            r#"INSERT INTO vault_entries
               (id, name, source_filename, encrypted_path, tags_json, created_at, updated_at, removed_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
        )
        .bind(&entry.id)
        .bind(&entry.name)
        .bind(&entry.source_filename)
        .bind(&entry.encrypted_path)
        .bind(&entry.tags_json)
        .bind(&entry.created_at)
        .bind(&entry.updated_at)
        .bind(&entry.removed_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<VaultEntry>> {
        let row = sqlx::query_as::<_, VaultEntry>("SELECT * FROM vault_entries WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    pub async fn list_active(pool: &SqlitePool) -> AppResult<Vec<VaultListItem>> {
        let entries = sqlx::query_as::<_, VaultEntry>(
            "SELECT * FROM vault_entries WHERE removed_at IS NULL ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(entries.into_iter().map(Self::to_list_item).collect())
    }

    pub async fn list_removed(pool: &SqlitePool) -> AppResult<Vec<VaultListItem>> {
        let entries = sqlx::query_as::<_, VaultEntry>(
            "SELECT * FROM vault_entries WHERE removed_at IS NOT NULL ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(entries.into_iter().map(Self::to_list_item).collect())
    }

    pub async fn search(pool: &SqlitePool, query: &str) -> AppResult<Vec<VaultListItem>> {
        let pattern = format!("%{}%", query);
        let entries = sqlx::query_as::<_, VaultEntry>(
            "SELECT * FROM vault_entries WHERE removed_at IS NULL AND (name LIKE ?1 OR tags_json LIKE ?2) ORDER BY updated_at DESC",
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await?;

        Ok(entries.into_iter().map(Self::to_list_item).collect())
    }

    pub async fn update_name_and_tags(
        pool: &SqlitePool,
        id: &str,
        name: &str,
        tags_json: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE vault_entries SET name = ?1, tags_json = ?2, updated_at = ?3 WHERE id = ?4 AND removed_at IS NULL",
        )
        .bind(name)
        .bind(tags_json)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::VaultNotFound, "资料不存在"));
        }
        Ok(())
    }

    pub async fn update_encrypted_path(
        pool: &SqlitePool,
        id: &str,
        encrypted_path: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE vault_entries SET encrypted_path = ?1, updated_at = ?2 WHERE id = ?3",
        )
        .bind(encrypted_path)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(ErrorCode::VaultNotFound, "资料不存在"));
        }
        Ok(())
    }

    pub async fn soft_remove(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE vault_entries SET removed_at = ?1 WHERE id = ?2 AND removed_at IS NULL",
        )
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(
                ErrorCode::VaultNotFound,
                "资料不存在或已移除",
            ));
        }
        Ok(())
    }

    pub async fn restore(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE vault_entries SET removed_at = NULL, updated_at = ?1 WHERE id = ?2 AND removed_at IS NOT NULL",
        )
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::new(
                ErrorCode::VaultNotFound,
                "资料不存在或未移除",
            ));
        }
        Ok(())
    }

    pub async fn permanent_delete(pool: &SqlitePool, id: &str) -> AppResult<Option<String>> {
        let entry = Self::find_by_id(pool, id).await?;
        match entry {
            Some(entry) => {
                sqlx::query("DELETE FROM vault_entries WHERE id = ?1")
                    .bind(id)
                    .execute(pool)
                    .await?;
                Ok(Some(entry.encrypted_path))
            }
            None => Ok(None),
        }
    }

    fn to_list_item(entry: VaultEntry) -> VaultListItem {
        let tags: Vec<String> = serde_json::from_str(&entry.tags_json).unwrap_or_default();
        VaultListItem {
            id: entry.id,
            name: entry.name,
            source_filename: entry.source_filename,
            tags,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
            removed_at: entry.removed_at,
        }
    }
}
