use std::path::{Component, Path, PathBuf};

use crate::error::{AppError, AppResult, ErrorCode};

/// 路径安全守卫：确保所有文件操作路径位于登记项目根目录内
pub struct PathGuard;

impl PathGuard {
    /// 规范化路径：解析 `.`、`..`、联接和符号链接（安全范围内），统一为小写（Windows 不区分大小写）
    pub fn canonicalize(path: &Path) -> AppResult<PathBuf> {
        // 先使用 dunce canonicalize（比 std::fs::canonicalize 更友好）
        match dunce::canonicalize(path) {
            Ok(p) => Ok(Self::normalize_separators(&p)),
            Err(e) => {
                // 如果路径不存在，尝试词法规范化
                if !path.exists() {
                    Ok(Self::lexical_normalize(path)?)
                } else {
                    Err(AppError::with_details(
                        ErrorCode::PathInvalid,
                        "路径规范化失败",
                        serde_json::json!({ "path": path.to_string_lossy(), "error": e.to_string() }),
                    ))
                }
            }
        }
    }

    /// 生成 canonical_path：小写、正斜杠、无尾部斜杠
    pub fn canonical_path_string(path: &Path) -> AppResult<String> {
        let canonical = Self::canonicalize(path)?;
        let mut s = canonical
            .to_string_lossy()
            .to_lowercase()
            .replace('\\', "/");
        while s.ends_with('/') && s.len() > 3 {
            s.pop();
        }
        Ok(s)
    }

    /// 词法规范化（不访问文件系统）：处理 `.` 和 `..`，不跟随符号链接
    fn lexical_normalize(path: &Path) -> AppResult<PathBuf> {
        let mut result = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    // 不允许逃逸到根以上
                    if !result.pop() {
                        return Err(AppError::new(
                            ErrorCode::PathEscapeProject,
                            "路径包含非法的 .. 引用",
                        ));
                    }
                }
                Component::RootDir => {
                    result.push("/");
                }
                Component::Prefix(p) => {
                    result.push(p.as_os_str());
                }
                Component::Normal(c) => {
                    result.push(c);
                }
            }
        }
        Ok(Self::normalize_separators(&result))
    }

    /// 统一路径分隔符为 Windows 风格（反斜杠）
    fn normalize_separators(path: &Path) -> PathBuf {
        let s = path.to_string_lossy().replace('/', "\\");
        PathBuf::from(s)
    }

    /// 验证 `child` 是否位于 `parent` 目录内（不跟随逃逸链接）
    pub fn ensure_within(project_root: &Path, child: &Path) -> AppResult<PathBuf> {
        let canonical_root = Self::canonicalize(project_root)?;
        let canonical_child = Self::canonicalize(child)?;

        let root_lower = canonical_root.to_string_lossy().to_lowercase();
        let child_lower = canonical_child.to_string_lossy().to_lowercase();

        if child_lower == root_lower {
            return Ok(canonical_child);
        }

        // 子路径必须以 root + 分隔符 开头
        let prefix = format!("{}\\", root_lower);
        if child_lower.starts_with(&prefix) {
            Ok(canonical_child)
        } else {
            Err(AppError::with_details(
                ErrorCode::PathEscapeProject,
                "目标路径位于项目根目录之外",
                serde_json::json!({
                    "project_root": canonical_root.to_string_lossy(),
                    "requested": canonical_child.to_string_lossy(),
                }),
            ))
        }
    }

    /// 验证相对路径不会逃逸项目根目录
    pub fn resolve_relative(project_root: &Path, relative: &str) -> AppResult<PathBuf> {
        // 检查相对路径不含绝对路径前缀
        if relative.contains(':')
            && (relative.starts_with('/') || relative.starts_with('\\') || relative.len() > 1)
        {
            // Windows 盘符如 C:
            if relative.len() > 1 && relative.as_bytes()[1] == b':' {
                return Err(AppError::new(
                    ErrorCode::PathEscapeProject,
                    "相对路径不能包含盘符",
                ));
            }
        }

        let joined = project_root.join(relative);
        let normalized = Self::lexical_normalize(&joined)?;
        Self::ensure_within(project_root, &normalized)
    }

    /// 检查路径是否为目录联接或符号链接（可能逃逸项目边界）
    pub fn is_unsafe_link(path: &Path) -> bool {
        // 使用 std::fs::symlink_metadata 检测链接本身（不跟随）
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            meta.file_type().is_symlink()
        } else {
            false
        }
    }

    /// 检查路径长度是否超过 Windows 限制
    pub fn check_length(path: &Path) -> AppResult<()> {
        let len = path.to_string_lossy().len();
        if len > 260 {
            Err(AppError::with_details(
                ErrorCode::PathTooLong,
                "路径长度超过 Windows 限制",
                serde_json::json!({ "length": len, "max": 260 }),
            ))
        } else {
            Ok(())
        }
    }

    /// 检查路径是否可写（用于添加项目前的校验）
    pub fn ensure_writable(path: &Path) -> AppResult<()> {
        if !path.exists() {
            return Err(AppError::new(ErrorCode::PathNotFound, "路径不存在"));
        }
        if !path.is_dir() {
            return Err(AppError::new(ErrorCode::PathInvalid, "路径不是目录"));
        }

        // 尝试创建临时文件来验证可写性
        let test_file = path.join(format!(".pm_write_test_{}", uuid::Uuid::new_v4()));
        match std::fs::File::create(&test_file) {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => {
                let _ = std::fs::remove_file(&test_file);
                Err(AppError::with_details(
                    ErrorCode::ProjectPathNotWritable,
                    "项目目录不可写",
                    serde_json::json!({ "path": path.to_string_lossy(), "error": e.to_string() }),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_canonical_path_string() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let canonical = PathGuard::canonical_path_string(path).unwrap();
        assert!(!canonical.contains('\\'));
        assert!(!canonical.ends_with('/'));
    }

    #[test]
    fn test_ensure_within() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let child = root.join("subdir").join("file.txt");
        std::fs::create_dir_all(root.join("subdir")).unwrap();
        std::fs::write(&child, "test").unwrap();

        let result = PathGuard::ensure_within(root, &child);
        assert!(result.is_ok());
    }

    #[test]
    fn test_escape_detection() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let outside = std::env::temp_dir().join("pm_escape_test_outside");
        std::fs::create_dir_all(&outside).unwrap();

        let result = PathGuard::ensure_within(root, &outside);
        assert!(result.is_err());
    }

    #[test]
    fn test_relative_escape() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let result = PathGuard::resolve_relative(root, "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_dotdot_in_relative() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let result = PathGuard::resolve_relative(root, "subdir/../valid.txt");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.to_string_lossy().ends_with("valid.txt"));
    }
}
