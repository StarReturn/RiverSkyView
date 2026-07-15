use tauri::State;

use crate::error::AppResult;
use crate::models::vault::*;
use crate::services::vault_service::VaultService;
use crate::state::AppState;

#[tauri::command]
pub async fn create_vault_entry(
    state: State<'_, AppState>,
    request: VaultCreateRequest,
) -> AppResult<VaultEntry> {
    let service = VaultService::new(state.vault_dir.clone());
    service.create(&state.db, request).await
}

#[tauri::command]
pub async fn import_vault_txt(
    state: State<'_, AppState>,
    request: VaultImportRequest,
) -> AppResult<VaultEntry> {
    let service = VaultService::new(state.vault_dir.clone());
    service.import_txt(&state.db, request).await
}

#[tauri::command]
pub async fn get_vault_content(state: State<'_, AppState>, id: String) -> AppResult<VaultContent> {
    let service = VaultService::new(state.vault_dir.clone());
    service.get_content(&state.db, &id).await
}

#[tauri::command]
pub async fn update_vault_entry(
    state: State<'_, AppState>,
    request: VaultUpdateRequest,
) -> AppResult<VaultEntry> {
    let service = VaultService::new(state.vault_dir.clone());
    service.update(&state.db, request).await
}

#[tauri::command]
pub async fn list_vault_entries(state: State<'_, AppState>) -> AppResult<Vec<VaultListItem>> {
    VaultService::list_active(&state.db).await
}

#[tauri::command]
pub async fn list_removed_vault_entries(
    state: State<'_, AppState>,
) -> AppResult<Vec<VaultListItem>> {
    VaultService::list_removed(&state.db).await
}

#[tauri::command]
pub async fn search_vault_entries(
    state: State<'_, AppState>,
    query: String,
) -> AppResult<Vec<VaultListItem>> {
    VaultService::search(&state.db, &query).await
}

#[tauri::command]
pub async fn remove_vault_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    VaultService::soft_remove(&state.db, &id).await
}

#[tauri::command]
pub async fn restore_vault_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    VaultService::restore(&state.db, &id).await
}

#[tauri::command]
pub async fn permanent_delete_vault_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let service = VaultService::new(state.vault_dir.clone());
    service.permanent_delete(&state.db, &id).await
}

#[tauri::command]
pub async fn clear_vault_plaintext() -> AppResult<()> {
    // 前端调用此命令通知后端清除内存（前端侧状态由前端管理）
    // 后端不持有明文状态，此命令作为信号用于日志记录
    tracing::info!("Vault plaintext cleared by frontend request");
    Ok(())
}
