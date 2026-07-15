use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::editor::{EditorProfile, EditorProfileInput, LaunchEditorResult};
use crate::services::editor_detection_service::EditorDetectionService;

pub struct EditorLaunchService;

impl EditorLaunchService {
    pub fn validate_open_mode(value: &str) -> AppResult<()> {
        if matches!(value, "default" | "new_window" | "reuse_window") {
            Ok(())
        } else {
            Err(AppError::new(
                ErrorCode::EditorProfileInvalid,
                "编辑器窗口模式无效",
            ))
        }
    }

    pub fn validate_profile_input(input: &EditorProfileInput) -> AppResult<()> {
        let name = input.name.trim();
        if name.is_empty() || name.chars().count() > 80 {
            return Err(AppError::new(
                ErrorCode::EditorProfileInvalid,
                "编辑器名称不能为空且不能超过 80 个字符",
            ));
        }
        let executable = PathBuf::from(input.executable.trim());
        if !executable.is_absolute()
            || !executable.is_file()
            || !EditorDetectionService::is_allowed_custom_executable(&executable)
        {
            return Err(AppError::new(
                ErrorCode::EditorProfileInvalid,
                "自定义编辑器必须是存在的 .exe 或 .com 绝对路径",
            ));
        }
        if input.args.len() > 32 {
            return Err(AppError::new(
                ErrorCode::EditorProfileInvalid,
                "自定义编辑器参数不能超过 32 个",
            ));
        }
        for arg in &input.args {
            if arg.len() > 2048 || arg.contains('\0') {
                return Err(AppError::new(
                    ErrorCode::EditorProfileInvalid,
                    "自定义编辑器参数过长或包含非法字符",
                ));
            }
            Self::validate_template(arg)?;
        }
        if let Some(working_directory) = &input.working_directory {
            Self::validate_template(working_directory)?;
            if working_directory != "{projectPath}" && !working_directory.trim().is_empty() {
                return Err(AppError::new(
                    ErrorCode::EditorProfileInvalid,
                    "工作目录目前只允许 {projectPath}",
                ));
            }
        }
        Ok(())
    }

    pub fn launch(
        project_root: &Path,
        project_name: &str,
        editor_key: &str,
        target: Option<&Path>,
        open_mode: &str,
        profiles: &[EditorProfile],
    ) -> AppResult<LaunchEditorResult> {
        Self::validate_open_mode(open_mode)?;
        let (name, executable, args) = if let Some(id) = editor_key.strip_prefix("custom:") {
            let profile = profiles
                .iter()
                .find(|profile| profile.id == id && profile.enabled)
                .ok_or_else(|| {
                    AppError::new(ErrorCode::EditorNotFound, "自定义编辑器不存在或已禁用")
                })?;
            let executable = EditorDetectionService::resolve_executable(&profile.executable)
                .ok_or_else(|| {
                    AppError::new(
                        ErrorCode::EditorExecutableMissing,
                        "自定义编辑器可执行文件不存在",
                    )
                })?;
            let args = Self::build_custom_args(profile, project_root, project_name, target)?;
            (profile.name.clone(), executable, args)
        } else {
            let descriptor = EditorDetectionService::find_builtin(editor_key)
                .ok_or_else(|| AppError::new(ErrorCode::EditorNotFound, "未知的内置编辑器"))?;
            let executable = descriptor
                .executable
                .map(PathBuf::from)
                .filter(|path| path.is_file())
                .ok_or_else(|| {
                    AppError::with_details(
                        ErrorCode::EditorExecutableMissing,
                        format!("未检测到 {}", descriptor.name),
                        serde_json::json!({ "editor_key": editor_key }),
                    )
                })?;
            let args = Self::build_builtin_args(editor_key, project_root, target, open_mode)?;
            (descriptor.name, executable, args)
        };

        tracing::info!(
            "Launching editor '{}' ({}) with {} argument(s)",
            name,
            editor_key,
            args.len()
        );
        Command::new(&executable)
            .args(&args)
            .current_dir(project_root)
            .spawn()
            .map_err(|error| {
                AppError::with_details(
                    ErrorCode::EditorLaunchFailed,
                    format!("启动 {} 失败", name),
                    serde_json::json!({ "error": error.to_string(), "editor_key": editor_key }),
                )
            })?;

        let target_display = target
            .and_then(|path| path.strip_prefix(project_root).ok())
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| project_root.to_string_lossy().to_string());
        Ok(LaunchEditorResult {
            editor_key: editor_key.to_string(),
            editor_name: name,
            executable: executable.to_string_lossy().to_string(),
            target_display,
            used_project_default: false,
        })
    }

    fn build_builtin_args(
        editor_key: &str,
        project_root: &Path,
        target: Option<&Path>,
        open_mode: &str,
    ) -> AppResult<Vec<String>> {
        let mut args = Vec::new();
        if matches!(
            editor_key,
            "builtin:vscode" | "builtin:vscode-insiders" | "builtin:cursor"
        ) {
            match open_mode {
                "new_window" => args.push("--new-window".to_string()),
                "reuse_window" => args.push("--reuse-window".to_string()),
                "default" => {}
                _ => Self::validate_open_mode(open_mode)?,
            }
        }
        args.push(target.unwrap_or(project_root).to_string_lossy().to_string());
        Ok(args)
    }

    fn build_custom_args(
        profile: &EditorProfile,
        project_root: &Path,
        project_name: &str,
        target: Option<&Path>,
    ) -> AppResult<Vec<String>> {
        let mut templates: Vec<String> =
            serde_json::from_str(&profile.args_json).map_err(|_| {
                AppError::new(
                    ErrorCode::EditorProfileInvalid,
                    "自定义编辑器参数 JSON 无效",
                )
            })?;
        if templates.is_empty() {
            templates.push("{projectPath}".to_string());
        }
        let project_path = project_root.to_string_lossy();
        let target_path = target.unwrap_or(project_root).to_string_lossy();
        templates
            .into_iter()
            .map(|template| {
                Self::validate_template(&template)?;
                Ok(template
                    .replace("{projectPath}", &project_path)
                    .replace("{projectName}", project_name)
                    .replace("{targetPath}", &target_path))
            })
            .collect()
    }

    fn validate_template(value: &str) -> AppResult<()> {
        let stripped = value
            .replace("{projectPath}", "")
            .replace("{projectName}", "")
            .replace("{targetPath}", "");
        if stripped.contains('{') || stripped.contains('}') {
            return Err(AppError::new(
                ErrorCode::EditorProfileInvalid,
                "参数包含未知占位符",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_path_stays_one_argument() {
        let path = Path::new(r"D:\中文 Project & # (demo)");
        let args =
            EditorLaunchService::build_builtin_args("builtin:vscode", path, None, "new_window")
                .unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[1], path.to_string_lossy());
    }

    #[test]
    fn unknown_placeholder_is_rejected() {
        assert!(EditorLaunchService::validate_template("{projectPath} {shell}").is_err());
    }

    #[test]
    fn invalid_open_mode_is_rejected() {
        assert!(EditorLaunchService::validate_open_mode("shell").is_err());
    }
}
