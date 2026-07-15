use std::path::Path;

use crate::error::{AppError, AppResult, ErrorCode};

/// 管理块标记
const BLOCK_START_PREFIX: &str = "<!-- project-manager:log-rules:start v";
const BLOCK_START_SUFFIX: &str = " -->";
const BLOCK_END: &str = "<!-- project-manager:log-rules:end -->";
const CURRENT_VERSION: u32 = 1;

/// AGENTS.md 管理块模板（Codex）
pub fn agents_md_block() -> String {
    r#"<!-- project-manager:log-rules:start v1 -->

## Project task logging

After completing or becoming blocked on a task that changes this project:

1. Create one new Markdown log under `pm_log/YYYY-MM-DD/`.
2. Name it `HHmmss-codex-<short-id>.md` using local time and a collision-resistant short ID.
3. Include YAML front matter with `pm_log_version`, `agent`, `started_at`, `finished_at`, and `status`.
4. Use `agent: codex` and one of `completed`, `failed`, or `blocked` for `status`.
5. Record the task goal, completed changes, changed files, verification results, and remaining issues.
6. Write the log after verification and before the final response.
7. Do not create a log for pure discussion or read-only analysis without project changes.
8. Never include passwords, tokens, cookies, private keys, environment variable values, or other secrets.
9. Never rewrite or delete existing project log files.

<!-- project-manager:log-rules:end -->"#.to_string()
}

/// CLAUDE.md 管理块模板（Claude Code）
pub fn claude_md_block() -> String {
    r#"<!-- project-manager:log-rules:start v1 -->

## Project task logging

After completing or becoming blocked on a task that changes this project:

1. Create one new Markdown log under `pm_log/YYYY-MM-DD/`.
2. Name it `HHmmss-claude-<short-id>.md` using local time and a collision-resistant short ID.
3. Include YAML front matter with `pm_log_version`, `agent`, `started_at`, `finished_at`, and `status`.
4. Use `agent: claude` and one of `completed`, `failed`, or `blocked` for `status`.
5. Record the task goal, completed changes, changed files, verification results, and remaining issues.
6. Write the log after verification and before the final response.
7. Do not create a log for pure discussion or read-only analysis without project changes.
8. Never include passwords, tokens, cookies, private keys, environment variable values, or other secrets.
9. Never rewrite or delete existing project log files.

<!-- project-manager:log-rules:end -->"#.to_string()
}

/// 合并结果
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub created: bool,
    pub updated: bool,
    pub new_content: String,
}

/// 管理块状态
#[derive(Debug, Clone, PartialEq)]
enum BlockState {
    /// 文件不存在
    #[allow(dead_code)]
    NotExists,
    /// 文件存在但没有管理块
    NoBlock,
    /// 存在指定版本的管理块
    SameVersion(u32),
    /// 存在旧版本管理块
    OlderVersion(u32),
    /// 管理块损坏：只有开始标记
    OnlyStart,
    /// 管理块损坏：只有结束标记
    OnlyEnd,
    /// 管理块损坏：多个开始或结束标记
    MultipleBlocks,
}

/// 分析文件中的管理块状态
fn analyze_block(content: &str) -> BlockState {
    let start_count = content.matches(BLOCK_START_PREFIX).count();
    let end_count = content.matches(BLOCK_END).count();

    if start_count == 0 && end_count == 0 {
        return BlockState::NoBlock;
    }

    if start_count > 1 || end_count > 1 {
        return BlockState::MultipleBlocks;
    }

    match (start_count, end_count) {
        (1, 0) => BlockState::OnlyStart,
        (0, 1) => BlockState::OnlyEnd,
        (1, 1) => {
            // 提取版本号
            if let Some(start_idx) = content.find(BLOCK_START_PREFIX) {
                let after_prefix = &content[start_idx + BLOCK_START_PREFIX.len()..];
                if let Some(suffix_idx) = after_prefix.find(BLOCK_START_SUFFIX) {
                    let version_str = &after_prefix[..suffix_idx];
                    if let Ok(version) = version_str.trim().parse::<u32>() {
                        if version == CURRENT_VERSION {
                            return BlockState::SameVersion(version);
                        } else if version < CURRENT_VERSION {
                            return BlockState::OlderVersion(version);
                        } else {
                            // 更高版本，视为同版本不修改
                            return BlockState::SameVersion(version);
                        }
                    }
                }
            }
            // 版本号解析失败，视为损坏
            BlockState::OnlyStart
        }
        _ => BlockState::MultipleBlocks,
    }
}

/// 查找管理块在内容中的起始和结束位置
fn find_block_range(content: &str) -> Option<(usize, usize)> {
    let start_idx = content.find(BLOCK_START_PREFIX)?;
    let end_marker_idx = content[start_idx..].rfind(BLOCK_END)?;
    Some((start_idx, start_idx + end_marker_idx + BLOCK_END.len()))
}

/// 合并管理块到文件内容
pub fn merge_block(content: Option<&str>, block: &str) -> AppResult<MergeResult> {
    match content {
        None => {
            // 文件不存在：创建并写入管理块
            Ok(MergeResult {
                created: true,
                updated: false,
                new_content: block.to_string(),
            })
        }
        Some(existing) => {
            let state = analyze_block(existing);
            match state {
                BlockState::NotExists => Ok(MergeResult {
                    created: true,
                    updated: false,
                    new_content: block.to_string(),
                }),
                BlockState::NoBlock => {
                    // 保留原文，在末尾补足换行后追加管理块
                    let trimmed = existing.trim_end();
                    let separator = if trimmed.is_empty() { "" } else { "\n\n" };
                    let new_content = format!("{}{}{}\n", trimmed, separator, block);
                    Ok(MergeResult {
                        created: false,
                        updated: true,
                        new_content,
                    })
                }
                BlockState::SameVersion(_) => {
                    // 幂等：保持不变
                    Ok(MergeResult {
                        created: false,
                        updated: false,
                        new_content: existing.to_string(),
                    })
                }
                BlockState::OlderVersion(_) => {
                    // 只替换管理块内容
                    if let Some((start, end)) = find_block_range(existing) {
                        let new_content =
                            format!("{}{}{}", &existing[..start], block, &existing[end..]);
                        Ok(MergeResult {
                            created: false,
                            updated: true,
                            new_content,
                        })
                    } else {
                        Err(AppError::new(
                            ErrorCode::ProjectInstructionFileCorrupted,
                            "无法定位管理块位置",
                        ))
                    }
                }
                BlockState::OnlyStart => Err(AppError::new(
                    ErrorCode::ProjectInstructionFileCorrupted,
                    "指令文件管理块损坏：只有开始标记，缺少结束标记。请人工检查文件内容后重试。",
                )),
                BlockState::OnlyEnd => Err(AppError::new(
                    ErrorCode::ProjectInstructionFileCorrupted,
                    "指令文件管理块损坏：只有结束标记，缺少开始标记。请人工检查文件内容后重试。",
                )),
                BlockState::MultipleBlocks => Err(AppError::new(
                    ErrorCode::ProjectInstructionFileCorrupted,
                    "指令文件管理块损坏：存在多个管理块标记。请人工检查文件内容后重试。",
                )),
            }
        }
    }
}

/// 读取文件内容，保留 BOM 状态和换行风格信息
struct FileContent {
    has_bom: bool,
    content: String,
    is_crlf: bool,
}

fn read_file_preserving_style(path: &Path) -> AppResult<Option<FileContent>> {
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(path)?;
    if bytes.is_empty() {
        return Ok(Some(FileContent {
            has_bom: false,
            content: String::new(),
            is_crlf: false,
        }));
    }

    // 检查 BOM
    let (has_bom, body) =
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            (true, &bytes[3..])
        } else {
            (false, &bytes[..])
        };

    let content = String::from_utf8(body.to_vec()).map_err(|e| {
        AppError::with_details(
            ErrorCode::FileEncodingInvalid,
            "指令文件编码无效，应为 UTF-8",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let is_crlf = content.contains("\r\n");

    Ok(Some(FileContent {
        has_bom,
        content,
        is_crlf,
    }))
}

/// 写入文件，使用临时文件 + 原子替换
fn write_file_atomic(path: &Path, content: &str, has_bom: bool, is_crlf: bool) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::new(ErrorCode::PathInvalid, "无法确定文件父目录"))?;

    // 确保父目录存在
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    // 处理换行风格
    let final_content = if is_crlf {
        content.replace('\n', "\r\n")
    } else {
        content.to_string()
    };

    // 构建字节内容
    let mut bytes = Vec::new();
    if has_bom {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    bytes.extend_from_slice(final_content.as_bytes());

    // 写入临时文件
    let temp_path = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("tmp")
    ));

    std::fs::write(&temp_path, &bytes).map_err(|e| {
        // 清理临时文件
        let _ = std::fs::remove_file(&temp_path);
        AppError::with_details(
            ErrorCode::FileWriteFailed,
            "写入临时文件失败",
            serde_json::json!({ "temp_path": temp_path.to_string_lossy(), "error": e.to_string() }),
        )
    })?;

    // 原子替换
    // Windows 上 std::fs::rename 在目标已存在时会失败，需要先删除
    if path.exists() {
        // 尝试重命名，如果失败则先删除再重命名
        if std::fs::rename(&temp_path, path).is_err() {
            // 保存备份
            let backup_path = path.with_extension("bak.pm");
            let _ = std::fs::rename(path, &backup_path);
            if std::fs::rename(&temp_path, path).is_err() {
                // 恢复备份
                let _ = std::fs::rename(&backup_path, path);
                return Err(AppError::new(
                    ErrorCode::FileWriteFailed,
                    "原子替换文件失败，原文件已保留",
                ));
            }
            let _ = std::fs::remove_file(&backup_path);
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

/// 合并并写入指令文件
pub fn merge_and_write(path: &Path, block: &str) -> AppResult<MergeResult> {
    let existing = read_file_preserving_style(path)?;

    let (content_ref, has_bom, is_crlf) = match &existing {
        Some(fc) => (Some(fc.content.as_str()), fc.has_bom, fc.is_crlf),
        None => (None, false, false),
    };

    let result = merge_block(content_ref, block)?;

    // 如果内容没有变化，不需要写入
    if !result.created && !result.updated {
        Ok(result)
    } else {
        write_file_atomic(path, &result.new_content, has_bom, is_crlf)?;
        Ok(result)
    }
}

/// 检查指令文件管理块状态（不修改）
pub fn check_block_status(path: &Path) -> AppResult<String> {
    let existing = read_file_preserving_style(path)?;
    match existing {
        None => Ok("not_exists".to_string()),
        Some(fc) => {
            let state = analyze_block(&fc.content);
            match state {
                BlockState::NotExists => Ok("not_exists".to_string()),
                BlockState::NoBlock => Ok("no_block".to_string()),
                BlockState::SameVersion(v) => Ok(format!("same_version_{}", v)),
                BlockState::OlderVersion(v) => Ok(format!("older_version_{}", v)),
                BlockState::OnlyStart => Ok("corrupted_only_start".to_string()),
                BlockState::OnlyEnd => Ok("corrupted_only_end".to_string()),
                BlockState::MultipleBlocks => Ok("corrupted_multiple".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_merge_nonexistent_file() {
        let block = agents_md_block();
        let result = merge_block(None, &block).unwrap();
        assert!(result.created);
        assert!(!result.updated);
        assert!(result
            .new_content
            .contains("project-manager:log-rules:start"));
    }

    #[test]
    fn test_merge_no_block() {
        let existing = "# Existing Rules\n\n- Keep this line.";
        let block = agents_md_block();
        let result = merge_block(Some(existing), &block).unwrap();
        assert!(!result.created);
        assert!(result.updated);
        assert!(result.new_content.contains("Keep this line."));
        assert!(result
            .new_content
            .contains("project-manager:log-rules:start"));
    }

    #[test]
    fn test_merge_same_version_idempotent() {
        let block = agents_md_block();
        let existing = format!("# Existing\n\n{}\n", block);
        let result = merge_block(Some(&existing), &block).unwrap();
        assert!(!result.created);
        assert!(!result.updated);
        assert_eq!(result.new_content, existing);
    }

    #[test]
    fn test_merge_older_version() {
        let old_block = "<!-- project-manager:log-rules:start v0 -->\nold rules\n<!-- project-manager:log-rules:end -->";
        let existing = format!("# Existing\n\n{}\n", old_block);
        let new_block = agents_md_block();
        let result = merge_block(Some(&existing), &new_block).unwrap();
        assert!(result.updated);
        assert!(result.new_content.contains("v1"));
        assert!(!result.new_content.contains("v0"));
        assert!(result.new_content.contains("# Existing"));
    }

    #[test]
    fn test_corrupted_only_start() {
        let existing = "# Existing\n\n<!-- project-manager:log-rules:start v1 -->\nsome rules\n";
        let block = agents_md_block();
        let result = merge_block(Some(existing), &block);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.code,
            ErrorCode::ProjectInstructionFileCorrupted.as_str()
        );
    }

    #[test]
    fn test_corrupted_only_end() {
        let existing = "# Existing\n\nsome rules\n<!-- project-manager:log-rules:end -->\n";
        let block = agents_md_block();
        let result = merge_block(Some(existing), &block);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_multiple_blocks() {
        let block = agents_md_block();
        let existing = format!("{}\n\n{}\n", block, block);
        let result = merge_block(Some(existing.as_str()), &block);
        assert!(result.is_err());
    }

    #[test]
    fn test_preserves_existing_content() {
        let existing = "# My Rules\n\n- Rule 1\n- Rule 2\n\nSome more content";
        let block = agents_md_block();
        let result = merge_block(Some(existing), &block).unwrap();
        assert!(result.new_content.contains("# My Rules"));
        assert!(result.new_content.contains("- Rule 1"));
        assert!(result.new_content.contains("- Rule 2"));
        assert!(result.new_content.contains("Some more content"));
    }

    #[test]
    fn test_atomic_write() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");
        let block = agents_md_block();

        let result = merge_and_write(&path, &block).unwrap();
        assert!(result.created);

        // 再次合并应该是幂等的
        let result2 = merge_and_write(&path, &block).unwrap();
        assert!(!result2.created);
        assert!(!result2.updated);
    }

    #[test]
    fn test_atomic_write_with_existing_content() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");

        std::fs::write(&path, "# Existing Rules\n\n- Keep this line.\n").unwrap();

        let block = agents_md_block();
        let result = merge_and_write(&path, &block).unwrap();
        assert!(result.updated);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Existing Rules"));
        assert!(content.contains("Keep this line"));
        assert!(content.contains("project-manager:log-rules:start"));
    }

    #[test]
    fn test_check_block_status() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");

        // 不存在
        assert_eq!(check_block_status(&path).unwrap(), "not_exists");

        // 无管理块
        std::fs::write(&path, "# Rules\n").unwrap();
        assert_eq!(check_block_status(&path).unwrap(), "no_block");

        // 有管理块
        let block = agents_md_block();
        std::fs::write(&path, format!("# Rules\n\n{}\n", block)).unwrap();
        assert!(check_block_status(&path)
            .unwrap()
            .starts_with("same_version_"));

        // 损坏：只有开始
        std::fs::write(
            &path,
            "<!-- project-manager:log-rules:start v1 -->\nrules\n",
        )
        .unwrap();
        assert_eq!(check_block_status(&path).unwrap(), "corrupted_only_start");
    }

    #[test]
    fn test_crlf_preservation() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");

        // 写入 CRLF 文件
        std::fs::write(&path, "# Existing\r\n\r\n- Rule\r\n").unwrap();

        let block = agents_md_block();
        let result = merge_and_write(&path, &block).unwrap();
        assert!(result.updated);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\r\n"));
    }
}
