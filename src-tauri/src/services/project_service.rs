use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::project::{
    AddProjectRequest, AddProjectResult, Project, ProjectIdentifier, ProjectListItem,
    RenameProjectRequest,
};
use crate::repositories::project_repository::ProjectRepository;
use crate::security::path_guard::PathGuard;
use crate::services::instruction_file_service;
use sqlx::SqlitePool;

const IDENTIFIER_FILE: &str = ".project-manager.json";
const PM_LOG_DIR: &str = "pm_log";

pub struct ProjectService;

impl ProjectService {
    /// 添加项目：完整流程
    pub async fn add(pool: &SqlitePool, request: AddProjectRequest) -> AppResult<AddProjectResult> {
        let raw_path = PathBuf::from(&request.path);

        // 1. 检查路径是否存在
        if !raw_path.exists() {
            return Err(AppError::new(ErrorCode::PathNotFound, "项目目录不存在"));
        }

        // 2. 检查路径是否为目录
        if !raw_path.is_dir() {
            return Err(AppError::new(ErrorCode::PathInvalid, "路径不是目录"));
        }

        // 3. 规范化路径
        let canonical_path = PathGuard::canonical_path_string(&raw_path)?;

        // 4. 检查路径长度
        PathGuard::check_length(&raw_path)?;

        // 5. 检查可写性
        PathGuard::ensure_writable(&raw_path)?;

        // 6. 检查重复（未移除）
        if let Some(existing) =
            ProjectRepository::find_active_by_canonical_path(pool, &canonical_path).await?
        {
            return Err(AppError::with_details(
                ErrorCode::ProjectAlreadyExists,
                "该项目已添加",
                serde_json::json!({ "existing_id": existing.id, "name": existing.name }),
            ));
        }

        // 7. 检查是否有已移除的同路径项目
        if let Some(removed) =
            ProjectRepository::find_removed_by_canonical_path(pool, &canonical_path).await?
        {
            return Err(AppError::with_details(
                ErrorCode::ProjectRemovedExists,
                "该项目曾已移除，可从回收站恢复",
                serde_json::json!({ "removed_id": removed.id, "name": removed.name }),
            ));
        }

        // 8. 准备文件变更
        let project_path = dunce::canonicalize(&raw_path).map_err(|e| {
            AppError::with_details(
                ErrorCode::PathInvalid,
                "路径规范化失败",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        let name = request.name.clone().unwrap_or_else(|| {
            project_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未命名项目")
                .to_string()
        });

        let project_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // 9. 创建 .project-manager.json
        let identifier = ProjectIdentifier {
            schema_version: 1,
            project_id: project_id.clone(),
            created_at: now.clone(),
        };

        let identifier_path = project_path.join(IDENTIFIER_FILE);
        let identifier_json = serde_json::to_string_pretty(&identifier)?;

        // 保存需要回滚的文件原始状态
        let agents_md_path = project_path.join("AGENTS.md");
        let claude_md_path = project_path.join("CLAUDE.md");
        let pm_log_path = project_path.join(PM_LOG_DIR);

        let agents_md_original: Option<Vec<u8>> = std::fs::read(&agents_md_path).ok();
        let claude_md_original: Option<Vec<u8>> = std::fs::read(&claude_md_path).ok();
        let pm_log_existed = pm_log_path.exists();

        // 原子写入标识文件
        write_atomic(&identifier_path, identifier_json.as_bytes())?;

        // 10. 合并 AGENTS.md
        let agents_block = instruction_file_service::agents_md_block();
        let agents_result =
            instruction_file_service::merge_and_write(&agents_md_path, &agents_block);
        let agents_md_created;
        let agents_md_updated;

        match agents_result {
            Ok(r) => {
                agents_md_created = r.created;
                agents_md_updated = r.updated;
            }
            Err(e) => {
                rollback_files(
                    &identifier_path,
                    &agents_md_path,
                    agents_md_original.as_deref(),
                    &claude_md_path,
                    None,
                    &pm_log_path,
                    pm_log_existed,
                );
                return Err(e);
            }
        }

        // 11. 合并 CLAUDE.md
        let claude_block = instruction_file_service::claude_md_block();
        let claude_result =
            instruction_file_service::merge_and_write(&claude_md_path, &claude_block);
        let claude_md_created;
        let claude_md_updated;

        match claude_result {
            Ok(r) => {
                claude_md_created = r.created;
                claude_md_updated = r.updated;
            }
            Err(e) => {
                rollback_files(
                    &identifier_path,
                    &agents_md_path,
                    agents_md_original.as_deref(),
                    &claude_md_path,
                    claude_md_original.as_deref(),
                    &pm_log_path,
                    pm_log_existed,
                );
                return Err(e);
            }
        }

        // 12. 创建 pm_log 目录
        let pm_log_created = if !pm_log_path.exists() {
            match std::fs::create_dir_all(&pm_log_path) {
                Ok(_) => true,
                Err(e) => {
                    rollback_files(
                        &identifier_path,
                        &agents_md_path,
                        agents_md_original.as_deref(),
                        &claude_md_path,
                        claude_md_original.as_deref(),
                        &pm_log_path,
                        pm_log_existed,
                    );
                    return Err(AppError::with_details(
                        ErrorCode::ProjectPmLogDirCreateFailed,
                        "无法创建 pm_log 目录",
                        serde_json::json!({ "error": e.to_string() }),
                    ));
                }
            }
        } else {
            false
        };

        // 13. 在数据库事务中创建项目记录
        let project = Project {
            id: project_id.clone(),
            name: name.clone(),
            path: project_path.to_string_lossy().to_string(),
            canonical_path: canonical_path.clone(),
            is_favorite: request.is_favorite.unwrap_or(false),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_opened_at: Some(now.clone()),
            last_activity_at: None,
            removed_at: None,
        };

        match ProjectRepository::insert(pool, &project).await {
            Ok(_) => Ok(AddProjectResult {
                project,
                agents_md_created,
                agents_md_updated,
                claude_md_created,
                claude_md_updated,
                pm_log_created,
                id_file_created: true,
            }),
            Err(e) => {
                // 数据库失败，回滚所有文件系统变更
                rollback_files(
                    &identifier_path,
                    &agents_md_path,
                    agents_md_original.as_deref(),
                    &claude_md_path,
                    claude_md_original.as_deref(),
                    &pm_log_path,
                    pm_log_existed,
                );
                Err(e)
            }
        }
    }

    /// 列出所有有效项目（带目录可用状态）
    pub async fn list_active(pool: &SqlitePool) -> AppResult<Vec<ProjectListItem>> {
        let projects = ProjectRepository::list_active(pool).await?;
        Ok(projects.into_iter().map(Self::to_list_item).collect())
    }

    /// 列出收藏项目
    pub async fn list_favorites(pool: &SqlitePool) -> AppResult<Vec<ProjectListItem>> {
        let projects = ProjectRepository::list_favorites(pool).await?;
        Ok(projects.into_iter().map(Self::to_list_item).collect())
    }

    /// 列出最近使用项目
    pub async fn list_recent(pool: &SqlitePool, limit: u64) -> AppResult<Vec<ProjectListItem>> {
        let projects = ProjectRepository::list_recent(pool, limit).await?;
        Ok(projects.into_iter().map(Self::to_list_item).collect())
    }

    /// 列出已移除项目
    pub async fn list_removed(pool: &SqlitePool) -> AppResult<Vec<ProjectListItem>> {
        let projects = ProjectRepository::list_removed(pool).await?;
        Ok(projects.into_iter().map(Self::to_list_item).collect())
    }

    /// 搜索项目
    pub async fn search(pool: &SqlitePool, query: &str) -> AppResult<Vec<ProjectListItem>> {
        if query.trim().is_empty() {
            return Self::list_active(pool).await;
        }
        let projects = ProjectRepository::search(pool, query).await?;
        Ok(projects.into_iter().map(Self::to_list_item).collect())
    }

    /// 重命名项目
    pub async fn rename(pool: &SqlitePool, request: RenameProjectRequest) -> AppResult<Project> {
        let name = request.new_name.trim();
        if name.is_empty() {
            return Err(AppError::new(
                ErrorCode::InvalidParameter,
                "项目名称不能为空",
            ));
        }
        if name.len() > 200 {
            return Err(AppError::new(ErrorCode::InvalidParameter, "项目名称过长"));
        }

        ProjectRepository::update_name(pool, &request.project_id, name).await?;
        let project = ProjectRepository::find_by_id(pool, &request.project_id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::ProjectNotFound, "项目不存在"))?;
        Ok(project)
    }

    /// 设置收藏
    pub async fn set_favorite(
        pool: &SqlitePool,
        project_id: &str,
        favorite: bool,
    ) -> AppResult<()> {
        ProjectRepository::set_favorite(pool, project_id, favorite).await
    }

    /// 软移除项目
    pub async fn soft_remove(pool: &SqlitePool, project_id: &str) -> AppResult<()> {
        ProjectRepository::soft_remove(pool, project_id).await
    }

    /// 批量软移除
    pub async fn batch_soft_remove(pool: &SqlitePool, project_ids: &[String]) -> AppResult<u64> {
        let mut count = 0u64;
        for id in project_ids {
            match ProjectRepository::soft_remove(pool, id).await {
                Ok(_) => count += 1,
                Err(e) => {
                    tracing::warn!("Failed to remove project {}: {}", id, e);
                }
            }
        }
        Ok(count)
    }

    /// 恢复项目
    pub async fn restore(pool: &SqlitePool, project_id: &str) -> AppResult<Project> {
        ProjectRepository::restore(pool, project_id).await?;
        let project = ProjectRepository::find_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::ProjectNotFound, "项目不存在"))?;
        Ok(project)
    }

    /// 更新最近打开时间
    pub async fn touch_opened(pool: &SqlitePool, project_id: &str) -> AppResult<()> {
        ProjectRepository::update_last_opened(pool, project_id).await
    }

    /// 获取项目详情
    pub async fn get(pool: &SqlitePool, project_id: &str) -> AppResult<Project> {
        ProjectRepository::find_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::ProjectNotFound, "项目不存在"))
    }

    /// 检查指令文件状态
    pub async fn check_instruction_files(project_path: &Path) -> AppResult<serde_json::Value> {
        let agents_status =
            instruction_file_service::check_block_status(&project_path.join("AGENTS.md"))?;
        let claude_status =
            instruction_file_service::check_block_status(&project_path.join("CLAUDE.md"))?;
        let pm_log_exists = project_path.join(PM_LOG_DIR).exists();
        let id_exists = project_path.join(IDENTIFIER_FILE).exists();

        Ok(serde_json::json!({
            "agents_md": agents_status,
            "claude_md": claude_status,
            "pm_log_exists": pm_log_exists,
            "identifier_exists": id_exists,
        }))
    }

    fn to_list_item(project: Project) -> ProjectListItem {
        let dir_available = Path::new(&project.path).exists() && Path::new(&project.path).is_dir();
        ProjectListItem {
            id: project.id,
            name: project.name,
            path: project.path,
            is_favorite: project.is_favorite,
            created_at: project.created_at,
            last_opened_at: project.last_opened_at,
            last_activity_at: project.last_activity_at,
            removed_at: project.removed_at,
            directory_available: dir_available,
        }
    }
}

/// 原子写入文件：临时文件 + 重命名
fn write_atomic(path: &Path, content: &[u8]) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::new(ErrorCode::PathInvalid, "无法确定文件父目录"))?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension("json.tmp");

    std::fs::write(&temp_path, content).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        AppError::with_details(
            ErrorCode::FileWriteFailed,
            "写入临时文件失败",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    if path.exists() {
        if std::fs::rename(&temp_path, path).is_err() {
            let backup = path.with_extension("json.bak");
            let _ = std::fs::rename(path, &backup);
            if std::fs::rename(&temp_path, path).is_err() {
                let _ = std::fs::rename(&backup, path);
                return Err(AppError::new(ErrorCode::FileWriteFailed, "原子替换失败"));
            }
            let _ = std::fs::remove_file(&backup);
        }
    } else {
        std::fs::rename(&temp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            AppError::with_details(
                ErrorCode::FileWriteFailed,
                "重命名临时文件失败",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    Ok(())
}

/// 回滚文件系统变更：恢复或删除被修改的文件
///
/// - `identifier_path`: 删除标识文件
/// - `agents_md_path` + `agents_original`: 恢复 AGENTS.md 原始内容，或删除（如果原本不存在）
/// - `claude_md_path` + `claude_original`: 恢复 CLAUDE.md 原始内容，或删除（如果原本不存在）
/// - `pm_log_path` + `pm_log_existed`: 如果本次创建了 pm_log 目录则删除
fn rollback_files(
    identifier_path: &Path,
    agents_md_path: &Path,
    agents_original: Option<&[u8]>,
    claude_md_path: &Path,
    claude_original: Option<&[u8]>,
    pm_log_path: &Path,
    pm_log_existed: bool,
) {
    // 删除标识文件（本次操作创建的）
    let _ = std::fs::remove_file(identifier_path);

    // 恢复 AGENTS.md
    match agents_original {
        Some(content) => {
            let _ = std::fs::write(agents_md_path, content);
        }
        None => {
            // 原本不存在，删除本次创建的
            let _ = std::fs::remove_file(agents_md_path);
        }
    }

    // 恢复 CLAUDE.md
    match claude_original {
        Some(content) => {
            let _ = std::fs::write(claude_md_path, content);
        }
        None => {
            let _ = std::fs::remove_file(claude_md_path);
        }
    }

    // 如果本次创建了 pm_log 目录且原本不存在，删除它
    if !pm_log_existed && pm_log_path.exists() {
        let _ = std::fs::remove_dir(pm_log_path);
    }

    tracing::warn!("Rolled back project files after add failure");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_json_format() {
        let id = ProjectIdentifier {
            schema_version: 1,
            project_id: "test-uuid".to_string(),
            created_at: "2026-07-15T10:00:00+08:00".to_string(),
        };
        let json = serde_json::to_string_pretty(&id).unwrap();
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"projectId\": \"test-uuid\""));
    }
}
