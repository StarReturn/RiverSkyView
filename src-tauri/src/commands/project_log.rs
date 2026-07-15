use tauri::State;

use crate::error::AppResult;
use crate::models::project_log::*;
use crate::services::project_log_service::ProjectLogService;
use crate::state::AppState;

#[tauri::command]
pub async fn sync_project_logs(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<LogSyncResult> {
    ProjectLogService::sync_logs(&state.db, &project_id).await
}

#[tauri::command]
pub async fn list_project_logs(
    state: State<'_, AppState>,
    project_id: String,
    agent: Option<String>,
    status: Option<String>,
    date: Option<String>,
) -> AppResult<Vec<ProjectLog>> {
    ProjectLogService::list_logs(
        &state.db,
        &project_id,
        agent.as_deref(),
        status.as_deref(),
        date.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn get_log_content(
    state: State<'_, AppState>,
    project_id: String,
    log_id: String,
) -> AppResult<Option<String>> {
    let settings = state.settings.read().await;
    let _max_size = settings.settings.markdown_max_size_mb;
    drop(settings);

    ProjectLogService::get_log_content(&state.db, &project_id, &log_id).await
}

#[tauri::command]
pub async fn get_activity_summary(
    state: State<'_, AppState>,
    project_id: String,
    days: Option<i64>,
) -> AppResult<ActivitySummary> {
    ProjectLogService::get_activity_summary(&state.db, &project_id, days.unwrap_or(365)).await
}
