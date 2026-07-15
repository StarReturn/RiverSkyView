use std::path::{Path, PathBuf};
use std::process::Command;

use sqlx::SqlitePool;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::editor::{
    EditorDescriptor, EditorInstallation, EditorInstallationRecord, EditorTestLaunchResult,
};
use crate::repositories::editor_installation_repository::EditorInstallationRepository;
use crate::services::editor_detection_service::{BuiltinEditorSpec, EditorDetectionService};

pub struct EditorInstallationService;

impl EditorInstallationService {
    pub async fn list(pool: &SqlitePool) -> AppResult<Vec<EditorInstallation>> {
        Self::ensure_seeded(pool).await?;
        let records = EditorInstallationRepository::list(pool).await?;
        let mut result = Vec::new();
        for spec in EditorDetectionService::specs() {
            if let Some(record) = records.iter().find(|item| item.editor_key == spec.key) {
                result.push(Self::to_installation(&spec, record));
            }
        }
        Ok(result)
    }

    pub async fn descriptors(pool: &SqlitePool) -> AppResult<Vec<EditorDescriptor>> {
        Ok(Self::list(pool)
            .await?
            .into_iter()
            .filter_map(|installation| {
                let spec = EditorDetectionService::specs()
                    .into_iter()
                    .find(|spec| spec.key == installation.editor_key)?;
                Some(EditorDescriptor {
                    key: installation.editor_key,
                    name: installation.name,
                    family: installation.family,
                    available: installation.available,
                    version: installation.version,
                    executable: installation.active_executable,
                    source: Some(installation.active_source),
                    supports_open_mode: spec.supports_open_mode,
                    supports_solution_target: spec.supports_solution_target,
                    is_custom: false,
                })
            })
            .collect())
    }

    pub async fn refresh(
        pool: &SqlitePool,
        editor_key: Option<&str>,
    ) -> AppResult<Vec<EditorInstallation>> {
        let specs = EditorDetectionService::specs();
        if let Some(key) = editor_key {
            Self::require_builtin(key)?;
        }
        for spec in specs
            .iter()
            .filter(|spec| editor_key.map_or(true, |key| spec.key == key))
        {
            let descriptor = EditorDetectionService::detect_builtin(spec);
            EditorInstallationRepository::upsert_detection(
                pool,
                spec.key,
                descriptor.executable.as_deref(),
                descriptor.source.as_deref(),
                descriptor.version.as_deref(),
            )
            .await?;
        }
        Self::list(pool).await
    }

    pub async fn set_manual(
        pool: &SqlitePool,
        editor_key: &str,
        executable: &str,
    ) -> AppResult<EditorInstallation> {
        Self::require_builtin(editor_key)?;
        let path = Self::validate_executable(executable)?;
        EditorInstallationRepository::set_manual(pool, editor_key, &path.to_string_lossy()).await?;
        Self::get(pool, editor_key).await
    }

    pub async fn clear_manual(
        pool: &SqlitePool,
        editor_key: &str,
    ) -> AppResult<EditorInstallation> {
        Self::require_builtin(editor_key)?;
        Self::ensure_seeded(pool).await?;
        EditorInstallationRepository::clear_manual(pool, editor_key).await?;
        Self::refresh(pool, Some(editor_key)).await?;
        Self::get(pool, editor_key).await
    }

    pub async fn set_enabled(
        pool: &SqlitePool,
        editor_key: &str,
        enabled: bool,
    ) -> AppResult<EditorInstallation> {
        Self::require_builtin(editor_key)?;
        Self::ensure_seeded(pool).await?;
        EditorInstallationRepository::set_enabled(pool, editor_key, enabled).await?;
        Self::get(pool, editor_key).await
    }

    pub async fn verify(
        pool: &SqlitePool,
        editor_key: &str,
        executable: Option<&str>,
    ) -> AppResult<EditorInstallation> {
        Self::require_builtin(editor_key)?;
        Self::ensure_seeded(pool).await?;
        let validation = match executable {
            Some(value) => Self::validate_executable(value),
            None => Self::resolve_active(pool, editor_key).await,
        };
        match validation {
            Ok(_) => {
                EditorInstallationRepository::set_verification(pool, editor_key, "valid", None)
                    .await?;
            }
            Err(error) => {
                let status = if error.code == ErrorCode::EditorExecutableMissing.as_str() {
                    "missing"
                } else {
                    "invalid"
                };
                EditorInstallationRepository::set_verification(
                    pool,
                    editor_key,
                    status,
                    Some(&error.message),
                )
                .await?;
                return Err(error);
            }
        }
        Self::get(pool, editor_key).await
    }

    pub async fn resolve_active(pool: &SqlitePool, editor_key: &str) -> AppResult<PathBuf> {
        Self::require_builtin(editor_key)?;
        Self::ensure_seeded(pool).await?;
        let record = EditorInstallationRepository::get(pool, editor_key)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::EditorNotConfigured, "开发工具尚未配置"))?;
        if !record.enabled {
            return Err(AppError::new(
                ErrorCode::EditorInstallationDisabled,
                "开发工具已禁用，请先在设置中启用",
            ));
        }
        let value = if record.active_source == "manual" {
            record.manual_executable.ok_or_else(|| {
                AppError::new(
                    ErrorCode::EditorManualPathRequired,
                    "手动路径为空，请重新选择可执行文件",
                )
            })?
        } else {
            record.detected_executable.ok_or_else(|| {
                AppError::new(
                    ErrorCode::EditorNotConfigured,
                    "未检测到开发工具，请手动选择可执行文件",
                )
            })?
        };
        Self::validate_executable(&value)
    }

    pub async fn test_launch(
        pool: &SqlitePool,
        editor_key: &str,
    ) -> AppResult<EditorTestLaunchResult> {
        let executable = Self::resolve_active(pool, editor_key).await?;
        let test_dir = std::env::temp_dir()
            .join("RiverSkyView")
            .join("editor-test");
        std::fs::create_dir_all(&test_dir).map_err(|error| {
            AppError::with_details(
                ErrorCode::EditorTestLaunchFailed,
                "无法创建编辑器测试目录",
                serde_json::json!({ "error": error.to_string() }),
            )
        })?;
        let mut command = Command::new(&executable);
        if editor_key != "builtin:visual-studio" {
            command.arg(&test_dir);
        }
        match command.current_dir(&test_dir).spawn() {
            Ok(_) => {
                EditorInstallationRepository::set_verification(pool, editor_key, "valid", None)
                    .await?;
                Ok(EditorTestLaunchResult {
                    editor_key: editor_key.to_string(),
                    executable: executable.to_string_lossy().to_string(),
                    success: true,
                    message: "测试进程已成功创建".to_string(),
                })
            }
            Err(error) => {
                let message = format!("测试启动失败：{}", error);
                EditorInstallationRepository::set_verification(
                    pool,
                    editor_key,
                    "launch_failed",
                    Some(&message),
                )
                .await?;
                Err(AppError::with_details(
                    ErrorCode::EditorTestLaunchFailed,
                    "开发工具测试启动失败",
                    serde_json::json!({ "editor_key": editor_key, "error": error.to_string() }),
                ))
            }
        }
    }

    pub async fn open_location(pool: &SqlitePool, editor_key: &str) -> AppResult<()> {
        let executable = Self::resolve_active(pool, editor_key).await?;
        Command::new("explorer.exe")
            .arg(format!("/select,{}", executable.to_string_lossy()))
            .spawn()
            .map_err(|error| {
                AppError::with_details(
                    ErrorCode::EditorLaunchFailed,
                    "无法打开开发工具所在位置",
                    serde_json::json!({ "error": error.to_string() }),
                )
            })?;
        Ok(())
    }

    pub fn validate_executable(value: &str) -> AppResult<PathBuf> {
        let value = value.trim();
        if value.is_empty() || value.len() > 32_767 || value.contains('\0') {
            return Err(AppError::new(
                ErrorCode::EditorExecutableInvalid,
                "可执行文件路径为空、过长或包含非法字符",
            ));
        }
        if value.starts_with(r"\\.\") {
            return Err(AppError::new(
                ErrorCode::EditorExecutableInvalid,
                "不允许使用 Windows 设备路径",
            ));
        }
        let path = PathBuf::from(value);
        if !path.is_absolute() {
            return Err(AppError::new(
                ErrorCode::EditorExecutableInvalid,
                "必须选择绝对路径的 .exe 或 .com 文件",
            ));
        }
        if !path.is_file() {
            return Err(AppError::new(
                ErrorCode::EditorExecutableMissing,
                "选择的可执行文件不存在",
            ));
        }
        if !EditorDetectionService::is_allowed_custom_executable(&path) {
            return Err(AppError::new(
                ErrorCode::EditorExecutableInvalid,
                "仅允许直接启动 .exe 或 .com，不允许脚本或快捷方式",
            ));
        }
        dunce::canonicalize(&path).map_err(|_| {
            AppError::new(
                ErrorCode::EditorExecutableInvalid,
                "无法解析所选可执行文件的真实路径",
            )
        })
    }

    fn require_builtin(editor_key: &str) -> AppResult<BuiltinEditorSpec> {
        EditorDetectionService::specs()
            .into_iter()
            .find(|spec| spec.key == editor_key)
            .ok_or_else(|| AppError::new(ErrorCode::EditorNotFound, "未知的内置开发工具"))
    }

    async fn ensure_seeded(pool: &SqlitePool) -> AppResult<()> {
        let existing = EditorInstallationRepository::list(pool).await?;
        for spec in EditorDetectionService::specs() {
            if existing.iter().any(|item| item.editor_key == spec.key) {
                continue;
            }
            let descriptor = EditorDetectionService::detect_builtin(&spec);
            EditorInstallationRepository::upsert_detection(
                pool,
                spec.key,
                descriptor.executable.as_deref(),
                descriptor.source.as_deref(),
                descriptor.version.as_deref(),
            )
            .await?;
        }
        Ok(())
    }

    async fn get(pool: &SqlitePool, editor_key: &str) -> AppResult<EditorInstallation> {
        let record = EditorInstallationRepository::get(pool, editor_key)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::EditorNotConfigured, "开发工具尚未配置"))?;
        let spec = Self::require_builtin(editor_key)?;
        Ok(Self::to_installation(&spec, &record))
    }

    fn to_installation(
        spec: &BuiltinEditorSpec,
        record: &EditorInstallationRecord,
    ) -> EditorInstallation {
        let active = if record.active_source == "manual" {
            record.manual_executable.clone()
        } else {
            record.detected_executable.clone()
        };
        let path_valid = active.as_deref().map(Path::new).is_some_and(|path| {
            path.is_file() && EditorDetectionService::is_allowed_custom_executable(path)
        });
        let status = if path_valid {
            if record.verification_status == "launch_failed" {
                "launch_failed".to_string()
            } else {
                "valid".to_string()
            }
        } else {
            "missing".to_string()
        };
        EditorInstallation {
            editor_key: record.editor_key.clone(),
            name: spec.name.to_string(),
            family: spec.family.to_string(),
            manual_executable: record.manual_executable.clone(),
            detected_executable: record.detected_executable.clone(),
            active_executable: active,
            active_source: record.active_source.clone(),
            available: record.enabled && path_valid,
            enabled: record.enabled,
            verification_status: status,
            detected_source: record.detected_source.clone(),
            version: record.version.clone(),
            last_detected_at: record.last_detected_at.clone(),
            last_verified_at: record.last_verified_at.clone(),
            last_error: record.last_error.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn rejects_relative_and_script_paths() {
        assert!(EditorInstallationService::validate_executable("tool.exe").is_err());
        assert!(EditorInstallationService::validate_executable(r"C:\tool.cmd").is_err());
        assert!(EditorInstallationService::validate_executable(r"C:\tool.ps1").is_err());
    }

    #[test]
    fn accepts_existing_native_executable_extension() {
        let temp = NamedTempFile::with_suffix(".exe").unwrap();
        assert!(
            EditorInstallationService::validate_executable(&temp.path().to_string_lossy()).is_ok()
        );
    }

    #[tokio::test]
    async fn manual_path_survives_detection_and_never_falls_back() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("installations.db");
        let url = format!(
            "sqlite://{}?mode=rwc",
            db_path.to_string_lossy().replace('\\', "/")
        );
        let pool = SqlitePool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let manual = temp.path().join("manual.exe");
        let detected = temp.path().join("detected.exe");
        std::fs::write(&manual, b"manual").unwrap();
        std::fs::write(&detected, b"detected").unwrap();

        EditorInstallationService::set_manual(&pool, "builtin:vscode", &manual.to_string_lossy())
            .await
            .unwrap();
        EditorInstallationRepository::upsert_detection(
            &pool,
            "builtin:vscode",
            Some(&detected.to_string_lossy()),
            Some("test"),
            None,
        )
        .await
        .unwrap();

        let record = EditorInstallationRepository::get(&pool, "builtin:vscode")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.active_source, "manual");
        assert_eq!(
            record.manual_executable.as_deref(),
            Some(
                dunce::canonicalize(&manual)
                    .unwrap()
                    .to_string_lossy()
                    .as_ref()
            )
        );
        assert_eq!(
            record.detected_executable.as_deref(),
            Some(detected.to_string_lossy().as_ref())
        );

        std::fs::remove_file(&manual).unwrap();
        let error = EditorInstallationService::resolve_active(&pool, "builtin:vscode")
            .await
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::EditorExecutableMissing.as_str());
    }
}
