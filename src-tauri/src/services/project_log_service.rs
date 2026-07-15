use chrono::{Local, NaiveDate, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::project_log::*;
use crate::repositories::log_repository::LogRepository;
use crate::repositories::project_repository::ProjectRepository;
use sqlx::SqlitePool;

pub struct ProjectLogService;

impl ProjectLogService {
    /// 同步项目的 pm_log 目录
    pub async fn sync_logs(pool: &SqlitePool, project_id: &str) -> AppResult<LogSyncResult> {
        let project = ProjectRepository::find_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::ProjectNotFound, "项目不存在"))?;

        let project_path = PathBuf::from(&project.path);
        let pm_log_path = project_path.join("pm_log");

        let now = Utc::now().to_rfc3339();

        if !pm_log_path.exists() {
            return Ok(LogSyncResult {
                project_id: project_id.to_string(),
                scanned: 0,
                added: 0,
                updated: 0,
                removed: 0,
                errors: 0,
                last_synced_at: now,
            });
        }

        // 扫描所有日志文件
        let log_files = Self::scan_pm_log_dir(&pm_log_path)?;
        let scanned = log_files.len() as u64;

        // 获取已索引的日志
        let existing_logs = LogRepository::find_by_project(pool, project_id).await?;
        let existing_map: HashMap<String, ProjectLog> = existing_logs
            .iter()
            .map(|log| (log.relative_path.clone(), log.clone()))
            .collect();

        let mut added = 0u64;
        let mut updated = 0u64;
        let mut errors = 0u64;
        let mut current_paths: Vec<String> = Vec::new();

        for (relative_path, full_path) in &log_files {
            current_paths.push(relative_path.clone());

            // 读取文件内容计算哈希
            let content = match std::fs::read_to_string(full_path) {
                Ok(c) => c,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };

            let content_hash = Self::compute_hash(&content);

            // 检查是否需要更新
            if let Some(existing) = existing_map.get(relative_path) {
                if existing.content_hash == content_hash {
                    continue; // 无变化
                }
            }

            // 解析日志
            let parsed = Self::parse_log(&content, full_path, relative_path);

            let log = ProjectLog {
                id: existing_map
                    .get(relative_path)
                    .map(|e| e.id.clone())
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
                project_id: project_id.to_string(),
                relative_path: relative_path.clone(),
                content_hash: content_hash.clone(),
                agent: parsed.agent.clone(),
                status: parsed.status.clone(),
                title: parsed.title.clone(),
                started_at: parsed.started_at.clone(),
                finished_at: parsed.finished_at.clone(),
                time_inferred: parsed.time_inferred,
                parse_status: parsed.parse_status.clone(),
                parse_error: parsed.parse_error.clone(),
                indexed_at: now.clone(),
            };

            match LogRepository::upsert(pool, &log).await {
                Ok(_) => {
                    if existing_map.contains_key(relative_path) {
                        updated += 1;
                    } else {
                        added += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to upsert log {}: {}", relative_path, e);
                    errors += 1;
                }
            }
        }

        // 删除已不存在的日志
        let removed = LogRepository::delete_missing(pool, project_id, &current_paths).await?;

        // 更新项目的最近活动时间
        if let Some(latest) = LogRepository::latest_finished_at(pool, project_id).await? {
            let _ = ProjectRepository::update_last_activity(pool, project_id, &latest).await;
        }

        Ok(LogSyncResult {
            project_id: project_id.to_string(),
            scanned,
            added,
            updated,
            removed,
            errors,
            last_synced_at: now,
        })
    }

    /// 扫描 pm_log 目录，返回 (相对路径, 完整路径) 列表
    fn scan_pm_log_dir(pm_log_path: &Path) -> AppResult<Vec<(String, PathBuf)>> {
        let mut files = Vec::new();

        if !pm_log_path.exists() {
            return Ok(files);
        }

        // 遍历 pm_log/YYYY-MM-DD/*.md
        let date_dirs = std::fs::read_dir(pm_log_path).map_err(|e| {
            AppError::with_details(
                ErrorCode::PathAccessDenied,
                "无法读取 pm_log 目录",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        for date_entry in date_dirs {
            let date_entry = match date_entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let date_path = date_entry.path();
            if !date_path.is_dir() {
                continue;
            }

            let date_name = date_entry.file_name().to_string_lossy().to_string();
            // 验证日期格式 YYYY-MM-DD
            if NaiveDate::parse_from_str(&date_name, "%Y-%m-%d").is_err() {
                continue;
            }

            let log_files = match std::fs::read_dir(&date_path) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for log_entry in log_files {
                let log_entry = match log_entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let log_path = log_entry.path();
                if !log_path.is_file() {
                    continue;
                }

                let extension = log_path.extension().and_then(|e| e.to_str()).unwrap_or("");

                if extension != "md" && extension != "markdown" {
                    continue;
                }

                let file_name = log_entry.file_name().to_string_lossy().to_string();
                let relative_path = format!("{}/{}", date_name, file_name);

                files.push((relative_path, log_path));
            }
        }

        Ok(files)
    }

    /// 计算内容哈希
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 解析日志文件
    pub fn parse_log(content: &str, file_path: &Path, _relative_path: &str) -> ParsedLog {
        let content_hash = Self::compute_hash(content);
        let raw_content = content.to_string();

        // 尝试从文件名解析 agent 和时间
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let (name_agent, name_time) = Self::parse_filename(file_name);

        // 尝试解析 YAML Front Matter
        let (fm_agent, fm_status, fm_title, fm_started, fm_finished, fm_valid) =
            Self::parse_front_matter(content);

        // 确定 agent
        let agent = fm_agent
            .or(name_agent)
            .unwrap_or_else(|| "other".to_string());

        // 确定 status
        let status = fm_status.unwrap_or_else(|| "completed".to_string());

        // 确定 finished_at
        let (finished_at, time_inferred) = match fm_finished {
            Some(f) => (f, false),
            None => match name_time {
                Some(t) => (t, false),
                None => {
                    // 使用文件修改时间
                    let mtime = std::fs::metadata(file_path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .map(|dt| dt.with_timezone(&Local).to_rfc3339())
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| Utc::now().to_rfc3339());
                    (mtime, true)
                }
            },
        };

        let started_at = fm_started;

        let title = fm_title.or_else(|| Self::extract_title(content));

        let (parse_status, parse_error) = if fm_valid {
            ("valid".to_string(), None)
        } else {
            (
                "invalid".to_string(),
                Some("Front Matter 解析失败或格式不完整".to_string()),
            )
        };

        ParsedLog {
            agent,
            status,
            title,
            started_at,
            finished_at,
            time_inferred,
            parse_status,
            parse_error,
            content_hash,
            raw_content,
        }
    }

    /// 解析文件名：HHmmss-agent-shortid.md
    fn parse_filename(filename: &str) -> (Option<String>, Option<String>) {
        let re = Regex::new(r"^(\d{6})-(codex|claude|other)-[a-zA-Z0-9]+\.md$").unwrap();
        if let Some(caps) = re.captures(filename) {
            let time = caps.get(1).unwrap().as_str();
            let agent = caps.get(2).unwrap().as_str().to_string();

            // 从文件名提取的时间需要结合日期目录
            let _agent_clone = agent.clone();
            return (Some(agent), Some(time.to_string()));
        }
        (None, None)
    }

    /// 解析 YAML Front Matter
    #[allow(clippy::type_complexity)]
    fn parse_front_matter(
        content: &str,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
    ) {
        let trimmed = content.trim_start();

        if !trimmed.starts_with("---") {
            return (None, None, None, None, None, true); // 无 Front Matter 视为有效
        }

        // 找到结束的 ---
        let rest = &trimmed[3..];
        let end_pos = match rest.find("\n---") {
            Some(pos) => pos,
            None => return (None, None, None, None, None, false), // Front Matter 不完整
        };

        let front_matter = &rest[..end_pos];
        let mut agent = None;
        let mut status = None;
        let mut started_at = None;
        let mut finished_at = None;
        let mut has_required = false;

        for line in front_matter.lines() {
            let line = line.trim();
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();

                match key {
                    "agent" => agent = Some(value.to_string()),
                    "status" => status = Some(value.to_string()),
                    "started_at" => started_at = Some(value.to_string()),
                    "finished_at" => finished_at = Some(value.to_string()),
                    "pm_log_version" => has_required = true,
                    _ => {}
                }
            }
        }

        // 验证 status
        if let Some(ref s) = status {
            if !["completed", "failed", "blocked"].contains(&s.as_str()) {
                return (agent, None, None, started_at, finished_at, false);
            }
        }

        // 验证 agent
        if let Some(ref a) = agent {
            if !["codex", "claude", "other"].contains(&a.as_str()) {
                return (None, status, None, started_at, finished_at, false);
            }
        }

        let has_any = has_required || agent.is_some() || status.is_some();
        (agent, status, None, started_at, finished_at, has_any)
    }

    /// 从 Markdown 内容提取第一个标题
    fn extract_title(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
        }
        None
    }

    /// 获取项目日志列表（带筛选）
    pub async fn list_logs(
        pool: &SqlitePool,
        project_id: &str,
        agent: Option<&str>,
        status: Option<&str>,
        date: Option<&str>,
    ) -> AppResult<Vec<ProjectLog>> {
        LogRepository::filter(pool, project_id, agent, status, date).await
    }

    /// 获取日志内容
    pub async fn get_log_content(
        pool: &SqlitePool,
        project_id: &str,
        log_id: &str,
    ) -> AppResult<Option<String>> {
        let log = LogRepository::find_by_id(pool, log_id).await?;

        match log {
            Some(log) if log.project_id == project_id => {
                let project = ProjectRepository::find_by_id(pool, project_id)
                    .await?
                    .ok_or_else(|| AppError::new(ErrorCode::ProjectNotFound, "项目不存在"))?;

                let project_path = PathBuf::from(&project.path);
                let log_path = project_path.join("pm_log").join(&log.relative_path);

                if !log_path.exists() {
                    return Ok(None);
                }

                let content = std::fs::read_to_string(&log_path).map_err(|e| {
                    AppError::with_details(
                        ErrorCode::FileReadFailed,
                        "读取日志文件失败",
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;

                Ok(Some(content))
            }
            _ => Ok(None),
        }
    }

    /// 获取活动摘要
    pub async fn get_activity_summary(
        pool: &SqlitePool,
        project_id: &str,
        days: i64,
    ) -> AppResult<ActivitySummary> {
        LogRepository::activity_summary(pool, project_id, days).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filename_valid() {
        let (agent, time) = ProjectLogService::parse_filename("143205-codex-a81f.md");
        assert_eq!(agent.as_deref(), Some("codex"));
        assert_eq!(time.as_deref(), Some("143205"));
    }

    #[test]
    fn test_parse_filename_claude() {
        let (agent, time) = ProjectLogService::parse_filename("151430-claude-c902.md");
        assert_eq!(agent.as_deref(), Some("claude"));
        assert_eq!(time.as_deref(), Some("151430"));
    }

    #[test]
    fn test_parse_filename_invalid() {
        let (agent, time) = ProjectLogService::parse_filename("invalid.txt");
        assert!(agent.is_none());
        assert!(time.is_none());
    }

    #[test]
    fn test_parse_front_matter_valid() {
        let content = r#"---
pm_log_version: 1
agent: codex
started_at: 2026-07-15T14:20:00+08:00
finished_at: 2026-07-15T14:32:05+08:00
status: completed
---

# Title
"#;
        let (agent, status, _title, started, finished, valid) =
            ProjectLogService::parse_front_matter(content);
        assert_eq!(agent.as_deref(), Some("codex"));
        assert_eq!(status.as_deref(), Some("completed"));
        assert!(started.is_some());
        assert!(finished.is_some());
        assert!(valid);
    }

    #[test]
    fn test_parse_front_matter_no_fm() {
        let content = "# Just a title\n\nNo front matter.";
        let (_, _, _, _, _, valid) = ProjectLogService::parse_front_matter(content);
        assert!(valid); // 无 FM 视为有效
    }

    #[test]
    fn test_parse_front_matter_incomplete() {
        let content = "---\nagent: codex\n# missing closing";
        let (_, _, _, _, _, valid) = ProjectLogService::parse_front_matter(content);
        assert!(!valid);
    }

    #[test]
    fn test_parse_front_matter_invalid_status() {
        let content = "---\nagent: codex\nstatus: invalid_status\n---\n";
        let (_, status, _, _, _, valid) = ProjectLogService::parse_front_matter(content);
        assert!(!valid);
        assert!(status.is_none());
    }

    #[test]
    fn test_parse_front_matter_invalid_agent() {
        let content = "---\nagent: invalid\nstatus: completed\n---\n";
        let (agent, _, _, _, _, valid) = ProjectLogService::parse_front_matter(content);
        assert!(!valid);
        assert!(agent.is_none());
    }

    #[test]
    fn test_extract_title() {
        let content = "# My Task Title\n\nSome content.";
        let title = ProjectLogService::extract_title(content);
        assert_eq!(title.as_deref(), Some("My Task Title"));
    }

    #[test]
    fn test_compute_hash_consistency() {
        let content = "test content";
        let hash1 = ProjectLogService::compute_hash(content);
        let hash2 = ProjectLogService::compute_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_parse_log_complete() {
        let content = r#"---
pm_log_version: 1
agent: codex
started_at: 2026-07-15T14:20:00+08:00
finished_at: 2026-07-15T14:32:05+08:00
status: completed
---

# 完成搜索功能

## 任务目标
增加搜索
"#;
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("143205-codex-a81f.md");
        std::fs::write(&path, content).unwrap();

        let parsed =
            ProjectLogService::parse_log(content, &path, "2026-07-15/143205-codex-a81f.md");

        assert_eq!(parsed.agent, "codex");
        assert_eq!(parsed.status, "completed");
        assert!(!parsed.time_inferred);
        assert_eq!(parsed.parse_status, "valid");
        assert!(parsed.title.as_deref().is_some());
    }
}
