use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::editor::EditorTarget;
use crate::security::path_guard::PathGuard;

pub struct EditorTargetService;

impl EditorTargetService {
    pub fn supports_solution_target(editor_key: &str) -> bool {
        matches!(editor_key, "builtin:visual-studio" | "builtin:rider")
            || editor_key.starts_with("custom:")
    }

    pub fn resolve(
        project_root: &Path,
        project_name: &str,
        editor_key: &str,
    ) -> AppResult<Vec<EditorTarget>> {
        let canonical_root = PathGuard::ensure_within(project_root, project_root)?;
        if !Self::supports_solution_target(editor_key) {
            return Ok(vec![Self::folder_target(true)]);
        }

        let mut candidates: Vec<(PathBuf, usize, u8)> = Vec::new();
        for entry in WalkDir::new(&canonical_root)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::include_entry)
            .filter_map(Result::ok)
        {
            if candidates.len() >= 50 || !entry.file_type().is_file() {
                continue;
            }
            let Some(ext) = entry.path().extension().and_then(|value| value.to_str()) else {
                continue;
            };
            let rank = match ext.to_ascii_lowercase().as_str() {
                "slnx" => 0,
                "sln" => 1,
                "csproj" | "fsproj" | "vbproj" | "vcxproj" => 2,
                _ => continue,
            };
            let canonical = PathGuard::ensure_within(&canonical_root, entry.path())?;
            candidates.push((canonical, entry.depth(), rank));
        }

        candidates.sort_by(|(a, a_depth, a_rank), (b, b_depth, b_rank)| {
            let a_match = Self::stem_matches(a, project_name);
            let b_match = Self::stem_matches(b, project_name);
            b_match
                .cmp(&a_match)
                .then_with(|| a_depth.cmp(b_depth))
                .then_with(|| a_rank.cmp(b_rank))
                .then_with(|| a.to_string_lossy().cmp(&b.to_string_lossy()))
        });

        let recommended_index = if candidates.len() == 1 {
            Some(0)
        } else {
            candidates
                .iter()
                .position(|(path, _, _)| Self::stem_matches(path, project_name))
        };

        let mut targets = Vec::with_capacity(candidates.len() + 1);
        for (index, (path, _, _)) in candidates.into_iter().enumerate() {
            let relative = path.strip_prefix(&canonical_root).map_err(|_| {
                AppError::new(ErrorCode::EditorTargetInvalid, "编辑器目标不在项目目录内")
            })?;
            let relative_string = relative.to_string_lossy().replace('/', "\\");
            targets.push(EditorTarget {
                display_name: relative_string.clone(),
                relative_path: Some(relative_string),
                kind: path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("project")
                    .to_ascii_lowercase(),
                recommended: recommended_index == Some(index),
            });
        }
        targets.push(Self::folder_target(targets.is_empty()));
        Ok(targets)
    }

    pub fn validate_target(
        project_root: &Path,
        relative_path: Option<&str>,
    ) -> AppResult<Option<PathBuf>> {
        let Some(relative_path) = relative_path.filter(|value| !value.trim().is_empty()) else {
            return Ok(None);
        };
        let path = PathGuard::resolve_relative(project_root, relative_path)?;
        if !path.is_file() {
            return Err(AppError::with_details(
                ErrorCode::EditorTargetInvalid,
                "编辑器目标不存在或不是文件",
                serde_json::json!({ "target": relative_path }),
            ));
        }
        Ok(Some(path))
    }

    fn folder_target(recommended: bool) -> EditorTarget {
        EditorTarget {
            relative_path: None,
            display_name: "直接打开项目目录".to_string(),
            kind: "folder".to_string(),
            recommended,
        }
    }

    fn include_entry(entry: &DirEntry) -> bool {
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            return true;
        }
        !matches!(
            entry
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .as_str(),
            ".git" | ".idea" | ".vs" | "node_modules" | "target" | "dist" | "build"
        )
    }

    fn stem_matches(path: &Path, project_name: &str) -> bool {
        path.file_stem()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(project_name))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn folder_editor_returns_root_only() {
        let temp = TempDir::new().unwrap();
        let targets = EditorTargetService::resolve(temp.path(), "demo", "builtin:vscode").unwrap();
        assert_eq!(targets.len(), 1);
        assert!(targets[0].relative_path.is_none());
    }

    #[test]
    fn solution_files_are_sorted_and_bounded() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("demo.sln"), "").unwrap();
        std::fs::create_dir_all(temp.path().join("backend")).unwrap();
        std::fs::write(temp.path().join("backend").join("other.sln"), "").unwrap();
        let targets = EditorTargetService::resolve(temp.path(), "demo", "builtin:rider").unwrap();
        assert_eq!(targets[0].relative_path.as_deref(), Some("demo.sln"));
        assert!(targets[0].recommended);
        assert!(targets.last().unwrap().relative_path.is_none());
    }

    #[test]
    fn target_escape_is_rejected() {
        let temp = TempDir::new().unwrap();
        assert!(
            EditorTargetService::validate_target(temp.path(), Some("..\\outside.sln")).is_err()
        );
    }
}
