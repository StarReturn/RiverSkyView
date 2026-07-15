use std::path::PathBuf;

use tauri::State;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::editor::*;
use crate::repositories::editor_preference_repository::EditorPreferenceRepository;
use crate::repositories::editor_profile_repository::EditorProfileRepository;
use crate::repositories::project_repository::ProjectRepository;
use crate::repositories::settings_repository::SettingsRepository;
use crate::services::editor_detection_service::EditorDetectionService;
use crate::services::editor_installation_service::EditorInstallationService;
use crate::services::editor_launch_service::EditorLaunchService;
use crate::services::editor_target_service::EditorTargetService;
use crate::services::project_service::ProjectService;
use crate::state::AppState;

#[tauri::command]
pub async fn list_editors(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> AppResult<Vec<EditorDescriptor>> {
    let mut editors = EditorInstallationService::descriptors(&state.db).await?;
    let profiles = EditorProfileRepository::list(&state.db, project_id.as_deref()).await?;
    editors.extend(
        profiles
            .iter()
            .map(EditorDetectionService::custom_descriptor),
    );
    Ok(editors)
}

#[tauri::command]
pub async fn refresh_editor_detection(
    state: State<'_, AppState>,
    project_id: Option<String>,
    editor_key: Option<String>,
) -> AppResult<Vec<EditorDescriptor>> {
    EditorInstallationService::refresh(&state.db, editor_key.as_deref()).await?;
    list_editors(state, project_id).await
}

#[tauri::command]
pub async fn list_editor_installations(
    state: State<'_, AppState>,
) -> AppResult<Vec<EditorInstallation>> {
    EditorInstallationService::list(&state.db).await
}

#[tauri::command]
pub async fn set_editor_manual_executable(
    state: State<'_, AppState>,
    editor_key: String,
    executable: String,
) -> AppResult<EditorInstallation> {
    EditorInstallationService::set_manual(&state.db, &editor_key, &executable).await
}

#[tauri::command]
pub async fn clear_editor_manual_executable(
    state: State<'_, AppState>,
    editor_key: String,
) -> AppResult<EditorInstallation> {
    EditorInstallationService::clear_manual(&state.db, &editor_key).await
}

#[tauri::command]
pub async fn verify_editor_executable(
    state: State<'_, AppState>,
    editor_key: String,
    executable: Option<String>,
) -> AppResult<EditorInstallation> {
    EditorInstallationService::verify(&state.db, &editor_key, executable.as_deref()).await
}

#[tauri::command]
pub async fn test_launch_editor(
    state: State<'_, AppState>,
    editor_key: String,
) -> AppResult<EditorTestLaunchResult> {
    EditorInstallationService::test_launch(&state.db, &editor_key).await
}

#[tauri::command]
pub async fn set_editor_enabled(
    state: State<'_, AppState>,
    editor_key: String,
    enabled: bool,
) -> AppResult<EditorInstallation> {
    EditorInstallationService::set_enabled(&state.db, &editor_key, enabled).await
}

#[tauri::command]
pub async fn open_editor_location(state: State<'_, AppState>, editor_key: String) -> AppResult<()> {
    EditorInstallationService::open_location(&state.db, &editor_key).await
}

#[tauri::command]
pub async fn resolve_editor_targets(
    state: State<'_, AppState>,
    project_id: String,
    editor_key: String,
) -> AppResult<Vec<EditorTarget>> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let root = PathBuf::from(&project.path);
    if !root.is_dir() {
        return Err(AppError::new(ErrorCode::PathNotFound, "项目目录不存在"));
    }
    EditorTargetService::resolve(&root, &project.name, &editor_key)
}

#[tauri::command]
pub async fn get_editor_settings(state: State<'_, AppState>) -> AppResult<EditorSettings> {
    Ok(EditorSettings {
        default_editor_key: SettingsRepository::get(&state.db, "default_editor_key")
            .await?
            .filter(|value| !value.is_empty()),
        open_mode: SettingsRepository::get(&state.db, "editor_open_mode")
            .await?
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "default".to_string()),
    })
}

#[tauri::command]
pub async fn set_editor_settings(
    state: State<'_, AppState>,
    input: EditorSettingsInput,
) -> AppResult<EditorSettings> {
    EditorLaunchService::validate_open_mode(&input.open_mode)?;
    if let Some(key) = input
        .default_editor_key
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        ensure_editor_key(&state.db, key, None).await?;
    }
    SettingsRepository::set(
        &state.db,
        "default_editor_key",
        input.default_editor_key.as_deref().unwrap_or(""),
    )
    .await?;
    SettingsRepository::set(&state.db, "editor_open_mode", &input.open_mode).await?;
    get_editor_settings(state).await
}

#[tauri::command]
pub async fn get_project_editor_preference(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Option<ProjectEditorPreference>> {
    EditorPreferenceRepository::get(&state.db, &project_id).await
}

#[tauri::command]
pub async fn set_project_editor_preference(
    state: State<'_, AppState>,
    project_id: String,
    input: EditorPreferenceInput,
) -> AppResult<ProjectEditorPreference> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    ensure_editor_key(&state.db, &input.editor_key, Some(&project_id)).await?;
    EditorLaunchService::validate_open_mode(input.open_mode.as_deref().unwrap_or("default"))?;
    if let Some(relative) = input.target_relative_path.as_deref() {
        EditorTargetService::validate_target(
            PathBuf::from(project.path).as_path(),
            Some(relative),
        )?;
    }
    EditorPreferenceRepository::set(&state.db, &project_id, &input).await
}

#[tauri::command]
pub async fn clear_project_editor_preference(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    EditorPreferenceRepository::clear(&state.db, &project_id).await
}

#[tauri::command]
pub async fn list_editor_profiles(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> AppResult<Vec<EditorProfile>> {
    EditorProfileRepository::list(&state.db, project_id.as_deref()).await
}

#[tauri::command]
pub async fn create_editor_profile(
    state: State<'_, AppState>,
    input: EditorProfileInput,
) -> AppResult<EditorProfile> {
    EditorLaunchService::validate_profile_input(&input)?;
    if let Some(project_id) = input.project_id.as_deref() {
        ProjectService::get(&state.db, project_id).await?;
    }
    EditorProfileRepository::create(&state.db, &input).await
}

#[tauri::command]
pub async fn update_editor_profile(
    state: State<'_, AppState>,
    id: String,
    input: EditorProfileInput,
) -> AppResult<EditorProfile> {
    EditorLaunchService::validate_profile_input(&input)?;
    if let Some(project_id) = input.project_id.as_deref() {
        ProjectService::get(&state.db, project_id).await?;
    }
    EditorProfileRepository::update(&state.db, &id, &input).await
}

#[tauri::command]
pub async fn delete_editor_profile(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let editor_key = format!("custom:{}", id);
    EditorProfileRepository::delete(&state.db, &id).await?;
    EditorPreferenceRepository::clear_editor(&state.db, &editor_key).await?;
    if SettingsRepository::get(&state.db, "default_editor_key")
        .await?
        .as_deref()
        == Some(editor_key.as_str())
    {
        SettingsRepository::set(&state.db, "default_editor_key", "").await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn launch_project_editor(
    state: State<'_, AppState>,
    request: LaunchEditorRequest,
) -> AppResult<LaunchEditorResult> {
    let project = ProjectService::get(&state.db, &request.project_id).await?;
    let root = PathBuf::from(&project.path);
    if !root.is_dir() {
        return Err(AppError::new(ErrorCode::PathNotFound, "项目目录不存在"));
    }

    let preference = EditorPreferenceRepository::get(&state.db, &request.project_id).await?;
    let global_settings = EditorSettings {
        default_editor_key: SettingsRepository::get(&state.db, "default_editor_key")
            .await?
            .filter(|value| !value.is_empty()),
        open_mode: SettingsRepository::get(&state.db, "editor_open_mode")
            .await?
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "default".to_string()),
    };

    let used_project_default = request.editor_key.is_none() && preference.is_some();
    let editor_key = request
        .editor_key
        .clone()
        .or_else(|| preference.as_ref().map(|value| value.editor_key.clone()))
        .or(global_settings.default_editor_key)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::EditorNotConfigured,
                "尚未设置默认编辑器，请先选择一个编辑器",
            )
        })?;
    ensure_editor_key(&state.db, &editor_key, Some(&request.project_id)).await?;

    let open_mode = request
        .open_mode
        .clone()
        .or_else(|| {
            preference
                .as_ref()
                .filter(|value| value.editor_key == editor_key)
                .map(|value| value.open_mode.clone())
        })
        .unwrap_or_else(|| global_settings.open_mode.clone());
    EditorLaunchService::validate_open_mode(&open_mode)?;

    let mut target_relative = request.target_relative_path.clone().or_else(|| {
        preference
            .as_ref()
            .filter(|value| value.editor_key == editor_key)
            .and_then(|value| value.target_relative_path.clone())
    });
    if target_relative.is_none() && EditorTargetService::supports_solution_target(&editor_key) {
        let targets = EditorTargetService::resolve(&root, &project.name, &editor_key)?;
        let file_targets: Vec<_> = targets
            .iter()
            .filter(|target| target.relative_path.is_some())
            .collect();
        if file_targets.len() > 1 {
            return Err(AppError::with_details(
                ErrorCode::EditorSelectionRequired,
                "项目中有多个可打开目标，请选择一个",
                serde_json::json!({ "targets": targets, "editor_key": editor_key }),
            ));
        }
        target_relative = file_targets
            .first()
            .and_then(|target| target.relative_path.clone());
    }

    let target = EditorTargetService::validate_target(&root, target_relative.as_deref())?;
    let profiles = EditorProfileRepository::list(&state.db, Some(&request.project_id)).await?;
    let builtin_executable = if editor_key.starts_with("builtin:") {
        Some(EditorInstallationService::resolve_active(&state.db, &editor_key).await?)
    } else {
        None
    };
    let mut result = EditorLaunchService::launch(
        &root,
        &project.name,
        &editor_key,
        builtin_executable.as_deref(),
        target.as_deref(),
        &open_mode,
        &profiles,
    )?;
    result.used_project_default = used_project_default;
    ProjectRepository::update_last_opened(&state.db, &request.project_id).await?;

    if request.remember_for_project.unwrap_or(false) {
        EditorPreferenceRepository::set(
            &state.db,
            &request.project_id,
            &EditorPreferenceInput {
                editor_key,
                target_relative_path: target_relative,
                open_mode: Some(open_mode),
            },
        )
        .await?;
    }
    Ok(result)
}

async fn ensure_editor_key(
    pool: &sqlx::SqlitePool,
    editor_key: &str,
    project_id: Option<&str>,
) -> AppResult<()> {
    if EditorDetectionService::specs()
        .iter()
        .any(|spec| spec.key == editor_key)
    {
        return Ok(());
    }
    let Some(id) = editor_key.strip_prefix("custom:") else {
        return Err(AppError::new(ErrorCode::EditorNotFound, "编辑器配置不存在"));
    };
    let profile = EditorProfileRepository::find(pool, id)
        .await?
        .filter(|profile| profile.enabled)
        .ok_or_else(|| AppError::new(ErrorCode::EditorNotFound, "编辑器配置不存在或已禁用"))?;
    if profile.project_id.as_deref().is_some() && profile.project_id.as_deref() != project_id {
        return Err(AppError::new(
            ErrorCode::EditorNotFound,
            "项目专用编辑器不属于当前项目",
        ));
    }
    Ok(())
}
