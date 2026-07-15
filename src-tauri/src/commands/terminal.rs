use std::path::PathBuf;
use tauri::State;

use crate::error::AppResult;
use crate::models::terminal::*;
use crate::services::project_service::ProjectService;
use crate::services::terminal_service::TerminalService;
use crate::state::AppState;

#[tauri::command]
pub async fn launch_terminal(
    state: State<'_, AppState>,
    request: LaunchRequest,
) -> AppResult<LaunchResult> {
    let project = ProjectService::get(&state.db, &request.project_id).await?;
    let project_path = PathBuf::from(&project.path);

    if !project_path.exists() {
        return Err(crate::error::AppError::new(
            crate::error::ErrorCode::PathNotFound,
            "项目目录不存在",
        ));
    }

    let action = request.action_kind.unwrap_or_else(|| "new".to_string());

    TerminalService::launch(&project_path, &project.name, &request.tool_kind, &action)
}

#[tauri::command]
pub async fn check_tools_availability() -> AppResult<Vec<ToolAvailability>> {
    Ok(TerminalService::check_tools())
}

#[tauri::command]
pub async fn launch_codex(
    state: State<'_, AppState>,
    project_id: String,
    action: Option<String>,
) -> AppResult<LaunchResult> {
    let settings = state.settings.read().await;
    let action = action.unwrap_or_else(|| settings.settings.default_codex_action.clone());
    drop(settings);

    launch_terminal(
        state,
        LaunchRequest {
            project_id,
            tool_kind: "codex".to_string(),
            action_kind: Some(action),
        },
    )
    .await
}

#[tauri::command]
pub async fn launch_claude(
    state: State<'_, AppState>,
    project_id: String,
    action: Option<String>,
) -> AppResult<LaunchResult> {
    let settings = state.settings.read().await;
    let action = action.unwrap_or_else(|| settings.settings.default_claude_action.clone());
    drop(settings);

    launch_terminal(
        state,
        LaunchRequest {
            project_id,
            tool_kind: "claude".to_string(),
            action_kind: Some(action),
        },
    )
    .await
}

#[tauri::command]
pub async fn set_project_default_action(
    state: State<'_, AppState>,
    project_id: String,
    tool_kind: String,
    action: String,
) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();

    // 使用 upsert 保存项目级默认动作
    let pool = &state.db;

    if sqlx::query("SELECT 1 FROM project_cli_defaults WHERE project_id = ?1")
        .bind(&project_id)
        .fetch_optional(pool)
        .await?
        .is_some()
    {
        let field = if tool_kind == "codex" {
            "default_codex_action"
        } else {
            "default_claude_action"
        };
        let sql = format!(
            "UPDATE project_cli_defaults SET {} = ?1, updated_at = ?2 WHERE project_id = ?3",
            field
        );
        sqlx::query(&sql)
            .bind(&action)
            .bind(&now)
            .bind(&project_id)
            .execute(pool)
            .await?;
    } else {
        let (codex_action, claude_action) = if tool_kind == "codex" {
            (action.clone(), "resume_picker".to_string())
        } else {
            ("resume_picker".to_string(), action.clone())
        };

        sqlx::query(
            r#"INSERT INTO project_cli_defaults
               (project_id, default_codex_action, default_claude_action, updated_at)
               VALUES (?1, ?2, ?3, ?4)"#,
        )
        .bind(&project_id)
        .bind(&codex_action)
        .bind(&claude_action)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_project_default_action(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<serde_json::Value> {
    let pool = &state.db;

    let row = sqlx::query(
        "SELECT default_codex_action, default_claude_action FROM project_cli_defaults WHERE project_id = ?1",
    )
    .bind(&project_id)
    .fetch_optional(pool)
    .await?;

    let settings = state.settings.read().await;

    match row {
        Some(row) => {
            use sqlx::Row;
            let codex: String = row.get("default_codex_action");
            let claude: String = row.get("default_claude_action");
            Ok(serde_json::json!({
                "codex": codex,
                "claude": claude,
            }))
        }
        None => Ok(serde_json::json!({
            "codex": settings.settings.default_codex_action,
            "claude": settings.settings.default_claude_action,
        })),
    }
}
