use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::vault::*;
use crate::repositories::vault_repository::VaultRepository;
use crate::security::dpapi::Dpapi;
use sqlx::SqlitePool;

pub struct VaultService {
    pub vault_dir: PathBuf,
}

impl VaultService {
    pub fn new(vault_dir: PathBuf) -> Self {
        VaultService { vault_dir }
    }

    /// 新建资料
    pub async fn create(
        &self,
        pool: &SqlitePool,
        request: VaultCreateRequest,
    ) -> AppResult<VaultEntry> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::new(
                ErrorCode::VaultNameRequired,
                "资料名称不能为空",
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // 加密内容
        let plaintext = request.content.as_bytes();
        let ciphertext = Dpapi::encrypt(plaintext)?;

        // 写入密文文件
        let encrypted_rel_path = format!("{}.vault", id);
        let encrypted_full_path = self.vault_dir.join(&encrypted_rel_path);

        // 原子写入：先写临时文件，验证可解密，再替换
        self.write_ciphertext_atomic(&encrypted_full_path, &ciphertext)?;

        // 验证可解密
        let read_back = std::fs::read(&encrypted_full_path)?;
        let decrypted = Dpapi::decrypt(&read_back)?;
        if decrypted != plaintext {
            // 解密内容不匹配，删除密文文件
            let _ = std::fs::remove_file(&encrypted_full_path);
            return Err(AppError::new(
                ErrorCode::VaultEncryptFailed,
                "加密验证失败：解密内容不匹配",
            ));
        }

        // 清理解密缓冲区
        let mut decrypted_vec = decrypted;
        Dpapi::zero_memory(&mut decrypted_vec);

        let tags_json = serde_json::to_string(&request.tags.unwrap_or_default())?;

        let entry = VaultEntry {
            id: id.clone(),
            name: name.to_string(),
            source_filename: request.source_filename,
            encrypted_path: encrypted_rel_path,
            tags_json,
            created_at: now.clone(),
            updated_at: now,
            removed_at: None,
        };

        VaultRepository::insert(pool, &entry).await?;

        Ok(entry)
    }

    /// 导入 TXT 文件
    pub async fn import_txt(
        &self,
        pool: &SqlitePool,
        request: VaultImportRequest,
    ) -> AppResult<VaultEntry> {
        let source_path = PathBuf::from(&request.source_file_path);

        // 读取源文件（不修改或删除源文件）
        let content = std::fs::read_to_string(&source_path).map_err(|e| {
            AppError::with_details(
                ErrorCode::FileReadFailed,
                "读取源文件失败",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        let source_filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        let name = if request.name.trim().is_empty() {
            source_filename
                .clone()
                .unwrap_or_else(|| "未命名资料".to_string())
        } else {
            request.name.trim().to_string()
        };

        self.create(
            pool,
            VaultCreateRequest {
                name,
                content,
                tags: request.tags,
                source_filename,
            },
        )
        .await
    }

    /// 获取资料内容（解密）
    pub async fn get_content(&self, pool: &SqlitePool, id: &str) -> AppResult<VaultContent> {
        let entry = VaultRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::VaultNotFound, "资料不存在"))?;

        if entry.removed_at.is_some() {
            return Err(AppError::new(ErrorCode::VaultNotFound, "资料已移除"));
        }

        let encrypted_path = self.vault_dir.join(&entry.encrypted_path);

        if !encrypted_path.exists() {
            return Err(AppError::new(
                ErrorCode::VaultCiphertextCorrupted,
                "密文文件不存在",
            ));
        }

        let ciphertext = std::fs::read(&encrypted_path)?;

        let plaintext_bytes = Dpapi::decrypt(&ciphertext)?;
        let content = String::from_utf8_lossy(&plaintext_bytes).to_string();

        // 清理缓冲区
        let mut plaintext_bytes = plaintext_bytes;
        Dpapi::zero_memory(&mut plaintext_bytes);

        let tags: Vec<String> = serde_json::from_str(&entry.tags_json).unwrap_or_default();

        Ok(VaultContent {
            id: entry.id,
            name: entry.name,
            content,
            tags,
            updated_at: entry.updated_at,
        })
    }

    /// 更新资料
    pub async fn update(
        &self,
        pool: &SqlitePool,
        request: VaultUpdateRequest,
    ) -> AppResult<VaultEntry> {
        let entry = VaultRepository::find_by_id(pool, &request.id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::VaultNotFound, "资料不存在"))?;

        // 如果有新内容，先写新密文再替换
        if let Some(content) = request.content {
            let plaintext = content.as_bytes();
            let new_ciphertext = Dpapi::encrypt(plaintext)?;

            // 写入临时密文文件
            let temp_path = self.vault_dir.join(format!("{}.vault.tmp", entry.id));
            let new_path = self.vault_dir.join(format!("{}.vault.new", entry.id));

            std::fs::write(&new_path, &new_ciphertext).map_err(|e| {
                let _ = std::fs::remove_file(&new_path);
                let _ = std::fs::remove_file(&temp_path);
                AppError::with_details(
                    ErrorCode::FileWriteFailed,
                    "写入新密文失败",
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

            // 验证新密文可解密
            let read_back = std::fs::read(&new_path)?;
            match Dpapi::decrypt(&read_back) {
                Ok(decrypted) => {
                    if decrypted != plaintext {
                        let _ = std::fs::remove_file(&new_path);
                        return Err(AppError::new(
                            ErrorCode::VaultEncryptFailed,
                            "新密文验证失败",
                        ));
                    }
                    // 清理
                    let mut dec = decrypted;
                    Dpapi::zero_memory(&mut dec);
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&new_path);
                    return Err(e);
                }
            }

            // 原子替换旧密文
            let old_path = self.vault_dir.join(&entry.encrypted_path);
            let backup_path = self.vault_dir.join(format!("{}.vault.bak", entry.id));

            // 备份旧密文
            if old_path.exists() {
                let _ = std::fs::rename(&old_path, &backup_path);
            }

            // 重命名新密文
            if std::fs::rename(&new_path, &old_path).is_err() {
                // 恢复旧密文
                if backup_path.exists() {
                    let _ = std::fs::rename(&backup_path, &old_path);
                }
                return Err(AppError::new(
                    ErrorCode::FileWriteFailed,
                    "替换密文失败，旧密文已保留",
                ));
            }

            // 删除备份
            let _ = std::fs::remove_file(&backup_path);
        }

        // 更新名称和标签
        if let Some(name) = request.name {
            let name = name.trim();
            if name.is_empty() {
                return Err(AppError::new(
                    ErrorCode::VaultNameRequired,
                    "资料名称不能为空",
                ));
            }
            let tags_json = if let Some(tags) = request.tags {
                serde_json::to_string(&tags)?
            } else {
                entry.tags_json.clone()
            };
            VaultRepository::update_name_and_tags(pool, &request.id, name, &tags_json).await?;
        } else if let Some(tags) = request.tags {
            let tags_json = serde_json::to_string(&tags)?;
            VaultRepository::update_name_and_tags(pool, &request.id, &entry.name, &tags_json)
                .await?;
        }

        VaultRepository::find_by_id(pool, &request.id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::VaultNotFound, "资料不存在"))
    }

    /// 列出有效资料
    pub async fn list_active(pool: &SqlitePool) -> AppResult<Vec<VaultListItem>> {
        VaultRepository::list_active(pool).await
    }

    /// 列出已移除资料
    pub async fn list_removed(pool: &SqlitePool) -> AppResult<Vec<VaultListItem>> {
        VaultRepository::list_removed(pool).await
    }

    /// 搜索资料
    pub async fn search(pool: &SqlitePool, query: &str) -> AppResult<Vec<VaultListItem>> {
        if query.trim().is_empty() {
            return Self::list_active(pool).await;
        }
        VaultRepository::search(pool, query).await
    }

    /// 软移除资料
    pub async fn soft_remove(pool: &SqlitePool, id: &str) -> AppResult<()> {
        VaultRepository::soft_remove(pool, id).await
    }

    /// 恢复资料
    pub async fn restore(pool: &SqlitePool, id: &str) -> AppResult<()> {
        VaultRepository::restore(pool, id).await
    }

    /// 永久清除资料
    pub async fn permanent_delete(&self, pool: &SqlitePool, id: &str) -> AppResult<()> {
        let encrypted_path = VaultRepository::permanent_delete(pool, id).await?;

        if let Some(rel_path) = encrypted_path {
            let full_path = self.vault_dir.join(&rel_path);
            if full_path.exists() {
                std::fs::remove_file(&full_path).map_err(|e| {
                    AppError::with_details(
                        ErrorCode::FileWriteFailed,
                        "删除密文文件失败",
                        serde_json::json!({ "path": full_path.to_string_lossy(), "error": e.to_string() }),
                    )
                })?;
            }
        }

        Ok(())
    }

    /// 原子写入密文文件
    fn write_ciphertext_atomic(&self, path: &Path, ciphertext: &[u8]) -> AppResult<()> {
        let temp_path = path.with_extension("vault.tmp");

        std::fs::write(&temp_path, ciphertext).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            AppError::with_details(
                ErrorCode::FileWriteFailed,
                "写入密文临时文件失败",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        if path.exists() {
            if std::fs::rename(&temp_path, path).is_err() {
                let backup = path.with_extension("vault.bak");
                let _ = std::fs::rename(path, &backup);
                if std::fs::rename(&temp_path, path).is_err() {
                    let _ = std::fs::rename(&backup, path);
                    return Err(AppError::new(
                        ErrorCode::FileWriteFailed,
                        "原子替换密文失败",
                    ));
                }
                let _ = std::fs::remove_file(&backup);
            }
        } else {
            std::fs::rename(&temp_path, path).map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                AppError::with_details(
                    ErrorCode::FileWriteFailed,
                    "重命名密文文件失败",
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        }

        Ok(())
    }

    /// 清理应用遗留的临时文件
    pub fn cleanup_temp_files(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.vault_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".tmp") || name.ends_with(".bak") || name.ends_with(".new") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(include_str!("../../migrations/0001_init.sql"))
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_content() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "测试服务器".to_string(),
                    content: "PM_TEST_SECRET_7f31a9_DO_NOT_USE".to_string(),
                    tags: Some(vec!["production".to_string()]),
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        // 验证密文文件不含明文
        let ciphertext = std::fs::read(tmp.path().join(format!("{}.vault", entry.id))).unwrap();
        let ciphertext_str = String::from_utf8_lossy(&ciphertext);
        assert!(!ciphertext_str.contains("PM_TEST_SECRET"));

        // 验证数据库不含明文
        let db_content: String =
            sqlx::query_scalar("SELECT tags_json FROM vault_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(!db_content.contains("PM_TEST_SECRET"));

        // 验证可解密
        let content = service.get_content(&pool, &entry.id).await.unwrap();
        assert_eq!(content.content, "PM_TEST_SECRET_7f31a9_DO_NOT_USE");
        assert_eq!(content.name, "测试服务器");
    }

    #[tokio::test]
    async fn test_update_content() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "服务器".to_string(),
                    content: "old_secret".to_string(),
                    tags: None,
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        // 更新内容
        let updated = service
            .update(
                &pool,
                VaultUpdateRequest {
                    id: entry.id.clone(),
                    name: Some("新名称".to_string()),
                    content: Some("new_secret".to_string()),
                    tags: Some(vec!["tag1".to_string()]),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "新名称");

        let content = service.get_content(&pool, &entry.id).await.unwrap();
        assert_eq!(content.content, "new_secret");
        assert_eq!(content.tags, vec!["tag1"]);
    }

    #[tokio::test]
    async fn test_soft_remove_and_restore() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "测试".to_string(),
                    content: "secret".to_string(),
                    tags: None,
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        // 软移除
        VaultService::soft_remove(&pool, &entry.id).await.unwrap();

        // 密文文件应保留
        assert!(tmp.path().join(format!("{}.vault", entry.id)).exists());

        // 恢复
        VaultService::restore(&pool, &entry.id).await.unwrap();

        let content = service.get_content(&pool, &entry.id).await.unwrap();
        assert_eq!(content.content, "secret");
    }

    #[tokio::test]
    async fn test_permanent_delete() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "待删除".to_string(),
                    content: "secret".to_string(),
                    tags: None,
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        let vault_file = tmp.path().join(format!("{}.vault", entry.id));
        assert!(vault_file.exists());

        service.permanent_delete(&pool, &entry.id).await.unwrap();

        assert!(!vault_file.exists());

        let result = service.get_content(&pool, &entry.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_corrupted_ciphertext_rejected() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "测试".to_string(),
                    content: "secret".to_string(),
                    tags: None,
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        // 篡改密文
        let vault_file = tmp.path().join(format!("{}.vault", entry.id));
        let mut ciphertext = std::fs::read(&vault_file).unwrap();
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        std::fs::write(&vault_file, &ciphertext).unwrap();

        // 解密应失败
        let result = service.get_content(&pool, &entry.id).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::VaultDecryptFailed.as_str());
    }

    #[tokio::test]
    async fn test_empty_content() {
        let pool = setup_db().await;
        let tmp = TempDir::new().unwrap();
        let service = VaultService::new(tmp.path().to_path_buf());

        let entry = service
            .create(
                &pool,
                VaultCreateRequest {
                    name: "空资料".to_string(),
                    content: "".to_string(),
                    tags: None,
                    source_filename: None,
                },
            )
            .await
            .unwrap();

        let content = service.get_content(&pool, &entry.id).await.unwrap();
        assert_eq!(content.content, "");
    }
}
