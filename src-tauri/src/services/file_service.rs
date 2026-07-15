use base64::Engine;
use std::path::Path;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::files::{FileNode, FilePreview, FileType};
use crate::security::path_guard::PathGuard;

pub struct FileService {
    pub show_hidden: bool,
    pub markdown_max_size: u64,
    pub image_max_size: u64,
}

impl FileService {
    pub fn new(show_hidden: bool, markdown_max_size: u64, image_max_size: u64) -> Self {
        FileService {
            show_hidden,
            markdown_max_size,
            image_max_size,
        }
    }

    /// 列出目录的直接子项（不递归）
    pub fn list_directory(
        &self,
        project_root: &Path,
        relative_dir: &str,
    ) -> AppResult<Vec<FileNode>> {
        let dir_path = if relative_dir.is_empty() || relative_dir == "." {
            project_root.to_path_buf()
        } else {
            PathGuard::resolve_relative(project_root, relative_dir)?
        };

        if !dir_path.exists() {
            return Err(AppError::new(ErrorCode::PathNotFound, "目录不存在"));
        }

        if !dir_path.is_dir() {
            return Err(AppError::new(ErrorCode::PathInvalid, "路径不是目录"));
        }

        let entries = match std::fs::read_dir(&dir_path) {
            Ok(e) => e,
            Err(e) => {
                return Err(AppError::with_details(
                    ErrorCode::PathAccessDenied,
                    "无法读取目录",
                    serde_json::json!({ "path": dir_path.to_string_lossy(), "error": e.to_string() }),
                ));
            }
        };

        let mut nodes: Vec<FileNode> = Vec::new();

        for entry in entries {
            match entry {
                Ok(entry) => {
                    let name = entry.file_name().to_string_lossy().to_string();

                    // 隐藏文件过滤
                    let is_hidden = name.starts_with('.') || Self::is_hidden_attr(&entry.path());

                    if is_hidden && !self.show_hidden {
                        continue;
                    }

                    let path = entry.path();
                    let relative_path = path
                        .strip_prefix(project_root)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_else(|_| name.clone());

                    let metadata = entry.metadata();
                    let (is_dir, size, modified) = match metadata {
                        Ok(m) => {
                            let modified = m
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| {
                                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                        .map(|dt| dt.to_rfc3339())
                                        .unwrap_or_default()
                                });
                            (
                                m.is_dir(),
                                if m.is_file() { Some(m.len()) } else { None },
                                modified,
                            )
                        }
                        Err(_) => (false, None, None),
                    };

                    // 检查是否为不安全的链接
                    let error = if PathGuard::is_unsafe_link(&path) {
                        Some("符号链接已跳过（安全限制）".to_string())
                    } else {
                        None
                    };

                    nodes.push(FileNode {
                        name,
                        relative_path,
                        absolute_path: path.to_string_lossy().to_string(),
                        is_dir,
                        size,
                        modified,
                        is_hidden,
                        error,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to read directory entry: {}", e);
                }
            }
        }

        // 排序：目录优先、自然排序
        nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => natord::compare(&a.name, &b.name),
        });

        Ok(nodes)
    }

    /// 读取文件用于预览
    pub fn read_file_for_preview(
        &self,
        project_root: &Path,
        relative_path: &str,
    ) -> AppResult<FilePreview> {
        let file_path = PathGuard::resolve_relative(project_root, relative_path)?;

        if !file_path.exists() {
            return Ok(FilePreview {
                file_type: FileType::NotFound,
                content: None,
                size: 0,
                encoding: None,
                error: Some("文件不存在".to_string()),
            });
        }

        if !file_path.is_file() {
            return Ok(FilePreview {
                file_type: FileType::Directory,
                content: None,
                size: 0,
                encoding: None,
                error: Some("路径是目录".to_string()),
            });
        }

        let metadata = std::fs::metadata(&file_path)?;
        let size = metadata.len();
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_type = Self::detect_file_type(&extension);

        match file_type {
            FileType::Markdown => {
                // 检查大小限制
                let max_bytes = self.markdown_max_size * 1024 * 1024;
                if size > max_bytes {
                    return Ok(FilePreview {
                        file_type: FileType::TooLarge,
                        content: None,
                        size,
                        encoding: None,
                        error: Some(format!("文件超过预览限制 ({} MB)", self.markdown_max_size)),
                    });
                }

                let bytes = std::fs::read(&file_path)?;
                match String::from_utf8(bytes) {
                    Ok(content) => Ok(FilePreview {
                        file_type: FileType::Markdown,
                        content: Some(content),
                        size,
                        encoding: Some("utf-8".to_string()),
                        error: None,
                    }),
                    Err(_) => Ok(FilePreview {
                        file_type: FileType::Binary,
                        content: None,
                        size,
                        encoding: None,
                        error: Some("文件编码无效，无法预览".to_string()),
                    }),
                }
            }
            FileType::Image | FileType::Svg => {
                let max_bytes = self.image_max_size * 1024 * 1024;
                if size > max_bytes {
                    return Ok(FilePreview {
                        file_type: FileType::TooLarge,
                        content: None,
                        size,
                        encoding: None,
                        error: Some(format!("图片超过预览限制 ({} MB)", self.image_max_size)),
                    });
                }

                let bytes = std::fs::read(&file_path)?;
                let mime_type = Self::mime_type(&extension);
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);

                Ok(FilePreview {
                    file_type,
                    content: Some(base64_data),
                    size,
                    encoding: Some(mime_type),
                    error: None,
                })
            }
            FileType::Text => {
                let max_bytes = self.markdown_max_size * 1024 * 1024;
                if size > max_bytes {
                    return Ok(FilePreview {
                        file_type: FileType::TooLarge,
                        content: None,
                        size,
                        encoding: None,
                        error: Some(format!("文件超过预览限制 ({} MB)", self.markdown_max_size)),
                    });
                }

                let bytes = std::fs::read(&file_path)?;
                match String::from_utf8(bytes) {
                    Ok(content) => Ok(FilePreview {
                        file_type: FileType::Text,
                        content: Some(content),
                        size,
                        encoding: Some("utf-8".to_string()),
                        error: None,
                    }),
                    Err(_) => Ok(FilePreview {
                        file_type: FileType::Binary,
                        content: None,
                        size,
                        encoding: None,
                        error: Some("文件编码无效".to_string()),
                    }),
                }
            }
            _ => Ok(FilePreview {
                file_type: FileType::Binary,
                content: None,
                size,
                encoding: None,
                error: Some("二进制文件，不支持预览".to_string()),
            }),
        }
    }

    /// 读取文件原始内容（用于日志详情预览）
    pub fn read_text_file(
        &self,
        project_root: &Path,
        relative_path: &str,
        max_size: u64,
    ) -> AppResult<Option<String>> {
        let file_path = PathGuard::resolve_relative(project_root, relative_path)?;

        if !file_path.exists() || !file_path.is_file() {
            return Ok(None);
        }

        let metadata = std::fs::metadata(&file_path)?;
        if metadata.len() > max_size * 1024 * 1024 {
            return Err(AppError::new(ErrorCode::FileTooLarge, "文件过大"));
        }

        let bytes = std::fs::read(&file_path)?;
        match String::from_utf8(bytes) {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    }

    /// 获取文件的绝对路径
    pub fn get_absolute_path(project_root: &Path, relative_path: &str) -> AppResult<String> {
        let path = PathGuard::resolve_relative(project_root, relative_path)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// 检查 Windows 隐藏属性
    fn is_hidden_attr(path: &Path) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata(path) {
                const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                return meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
            }
        }
        false
    }

    /// 根据扩展名检测文件类型
    fn detect_file_type(extension: &str) -> FileType {
        match extension {
            "md" | "markdown" => FileType::Markdown,
            "svg" => FileType::Svg,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => FileType::Image,
            "txt" | "log" | "json" | "yaml" | "yml" | "toml" | "ini" | "conf" | "ts" | "js"
            | "tsx" | "jsx" | "vue" | "rs" | "py" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
            | "cs" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bat" | "ps1" | "html"
            | "css" | "scss" | "less" | "xml" | "sql" | "dockerfile" | "gitignore" | "env"
            | "lock" => FileType::Text,
            _ => FileType::Binary,
        }
    }

    /// 获取 MIME 类型
    fn mime_type(extension: &str) -> String {
        match extension {
            "png" => "image/png".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "gif" => "image/gif".to_string(),
            "webp" => "image/webp".to_string(),
            "bmp" => "image/bmp".to_string(),
            "svg" => "image/svg+xml".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// 在 Windows 资源管理器中打开目录
    pub fn open_in_explorer(path: &Path) -> AppResult<()> {
        if !path.exists() {
            return Err(AppError::new(ErrorCode::PathNotFound, "路径不存在"));
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("explorer.exe")
                .arg(path)
                .spawn()
                .map_err(|e| {
                    AppError::with_details(
                        ErrorCode::Internal,
                        "无法打开资源管理器",
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
            Ok(())
        }

        #[cfg(not(windows))]
        {
            Err(AppError::new(ErrorCode::Internal, "此功能仅支持 Windows"))
        }
    }

    /// 在资源管理器中定位文件
    pub fn reveal_in_explorer(path: &Path) -> AppResult<()> {
        if !path.exists() {
            return Err(AppError::new(ErrorCode::PathNotFound, "文件不存在"));
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("explorer.exe")
                .args(["/select,", &path.to_string_lossy()])
                .spawn()
                .map_err(|e| {
                    AppError::with_details(
                        ErrorCode::Internal,
                        "无法在资源管理器中定位文件",
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
            Ok(())
        }

        #[cfg(not(windows))]
        {
            Err(AppError::new(ErrorCode::Internal, "此功能仅支持 Windows"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_directory() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        std::fs::write(root.join("file1.txt"), "content").unwrap();
        std::fs::write(root.join("file2.md"), "# Markdown").unwrap();
        std::fs::create_dir(root.join("subdir")).unwrap();

        let service = FileService::new(false, 2, 20);
        let nodes = service.list_directory(root, "").unwrap();

        // 目录应排在前面
        assert!(nodes[0].is_dir);
        assert_eq!(nodes[0].name, "subdir");
        assert!(!nodes.iter().any(|n| n.name.starts_with('.')));
    }

    #[test]
    fn test_list_directory_hidden() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        std::fs::write(root.join(".hidden"), "secret").unwrap();
        std::fs::write(root.join("visible.txt"), "content").unwrap();

        // 不显示隐藏
        let service = FileService::new(false, 2, 20);
        let nodes = service.list_directory(root, "").unwrap();
        assert!(!nodes.iter().any(|n| n.name == ".hidden"));

        // 显示隐藏
        let service = FileService::new(true, 2, 20);
        let nodes = service.list_directory(root, "").unwrap();
        assert!(nodes.iter().any(|n| n.name == ".hidden"));
    }

    #[test]
    fn test_read_markdown() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        std::fs::write(root.join("test.md"), "# Hello World").unwrap();

        let service = FileService::new(false, 2, 20);
        let preview = service.read_file_for_preview(root, "test.md").unwrap();
        assert_eq!(preview.file_type, FileType::Markdown);
        assert!(preview.content.unwrap().contains("Hello World"));
    }

    #[test]
    fn test_path_escape_blocked() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let service = FileService::new(false, 2, 20);
        let result = service.read_file_for_preview(root, "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(FileService::detect_file_type("md"), FileType::Markdown);
        assert_eq!(FileService::detect_file_type("svg"), FileType::Svg);
        assert_eq!(FileService::detect_file_type("png"), FileType::Image);
        assert_eq!(FileService::detect_file_type("exe"), FileType::Binary);
    }
}
