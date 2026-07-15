use std::path::PathBuf;
use tauri::State;

use crate::error::AppResult;
use crate::models::project::*;
use crate::services::project_service::ProjectService;
use crate::state::AppState;

#[tauri::command]
pub async fn add_project(
    state: State<'_, AppState>,
    request: AddProjectRequest,
) -> AppResult<AddProjectResult> {
    ProjectService::add(&state.db, request).await
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<ProjectListItem>> {
    ProjectService::list_active(&state.db).await
}

#[tauri::command]
pub async fn list_favorite_projects(state: State<'_, AppState>) -> AppResult<Vec<ProjectListItem>> {
    ProjectService::list_favorites(&state.db).await
}

#[tauri::command]
pub async fn list_recent_projects(
    state: State<'_, AppState>,
    limit: Option<u64>,
) -> AppResult<Vec<ProjectListItem>> {
    ProjectService::list_recent(&state.db, limit.unwrap_or(20)).await
}

#[tauri::command]
pub async fn list_removed_projects(state: State<'_, AppState>) -> AppResult<Vec<ProjectListItem>> {
    ProjectService::list_removed(&state.db).await
}

#[tauri::command]
pub async fn search_projects(
    state: State<'_, AppState>,
    query: String,
) -> AppResult<Vec<ProjectListItem>> {
    ProjectService::search(&state.db, &query).await
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, project_id: String) -> AppResult<Project> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    ProjectService::touch_opened(&state.db, &project_id).await?;
    Ok(project)
}

#[tauri::command]
pub async fn rename_project(
    state: State<'_, AppState>,
    request: RenameProjectRequest,
) -> AppResult<Project> {
    ProjectService::rename(&state.db, request).await
}

#[tauri::command]
pub async fn set_project_favorite(
    state: State<'_, AppState>,
    project_id: String,
    favorite: bool,
) -> AppResult<()> {
    ProjectService::set_favorite(&state.db, &project_id, favorite).await
}

#[tauri::command]
pub async fn remove_project(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    ProjectService::soft_remove(&state.db, &project_id).await
}

#[tauri::command]
pub async fn batch_remove_projects(
    state: State<'_, AppState>,
    project_ids: Vec<String>,
) -> AppResult<u64> {
    ProjectService::batch_soft_remove(&state.db, &project_ids).await
}

#[tauri::command]
pub async fn restore_project(state: State<'_, AppState>, project_id: String) -> AppResult<Project> {
    ProjectService::restore(&state.db, &project_id).await
}

#[tauri::command]
pub async fn check_instruction_files(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<serde_json::Value> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let path = PathBuf::from(&project.path);
    ProjectService::check_instruction_files(&path).await
}

#[tauri::command]
pub async fn count_projects(state: State<'_, AppState>) -> AppResult<i64> {
    crate::repositories::project_repository::ProjectRepository::count_active(&state.db).await
}
