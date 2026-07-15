use tauri::State;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::settings::*;
use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    let settings = state.settings.read().await;
    Ok(settings.settings.clone())
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    key: String,
    value: String,
) -> AppResult<AppSettings> {
    // 保存旧值用于副作用比较
    let old_value = {
        let settings = state.settings.read().await;
        get_setting_value(&settings.settings, &key)
    };

    let mut settings = state.settings.write().await;
    let result = settings.update_setting(&state.db, &key, &value).await?;
    drop(settings);

    // 处理需要产生实际副作用的设置项
    let old_str = old_value.as_deref().unwrap_or("");
    if old_str != value.as_str() {
        apply_setting_side_effects_internal(&app_handle, &key, old_str, &value)?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    updates: Vec<SettingUpdate>,
) -> AppResult<AppSettings> {
    // 收集旧值
    let old_values: Vec<(String, Option<String>)> = {
        let settings = state.settings.read().await;
        updates
            .iter()
            .map(|u| {
                let old = get_setting_value(&settings.settings, &u.key);
                (u.key.clone(), old)
            })
            .collect()
    };

    let mut settings = state.settings.write().await;
    for update in &updates {
        settings
            .update_setting(&state.db, &update.key, &update.value)
            .await?;
    }
    let result = settings.settings.clone();
    drop(settings);

    // 处理副作用
    for (i, update) in updates.iter().enumerate() {
        if old_values[i].1.as_deref() != Some(update.value.as_str()) {
            let _ = apply_setting_side_effects_internal(
                &app_handle,
                &update.key,
                &old_values[i].1.clone().unwrap_or_default(),
                &update.value,
            );
        }
    }

    Ok(result)
}

/// 前端可调用的命令：手动应用设置副作用（用于设置页面初始化后同步状态）
#[tauri::command]
pub async fn apply_setting_side_effects(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    key: String,
) -> AppResult<()> {
    let settings = state.settings.read().await;
    let current_value = get_setting_value(&settings.settings, &key);
    let current_str = current_value.unwrap_or_default();
    drop(settings);

    apply_setting_side_effects_internal(&app_handle, &key, "", &current_str)?;
    Ok(())
}

/// 获取设置项的当前值（字符串形式）
fn get_setting_value(settings: &AppSettings, key: &str) -> Option<String> {
    match key {
        "theme" => Some(settings.theme.clone()),
        "minimize_to_tray" => Some(settings.minimize_to_tray.to_string()),
        "global_shortcut" => Some(settings.global_shortcut.clone().unwrap_or_default()),
        "launch_at_login" => Some(settings.launch_at_login.to_string()),
        "show_hidden_files" => Some(settings.show_hidden_files.to_string()),
        "markdown_max_size_mb" => Some(settings.markdown_max_size_mb.to_string()),
        "image_max_size_mb" => Some(settings.image_max_size_mb.to_string()),
        "allow_remote_resources" => Some(settings.allow_remote_resources.to_string()),
        "clipboard_clear_seconds" => Some(settings.clipboard_clear_seconds.to_string()),
        "vault_auto_mask_seconds" => Some(settings.vault_auto_mask_seconds.to_string()),
        "default_codex_action" => Some(settings.default_codex_action.clone()),
        "default_claude_action" => Some(settings.default_claude_action.clone()),
        "window_maximized" => Some(settings.window_maximized.to_string()),
        _ => None,
    }
}

/// 应用设置变更的实际副作用
fn apply_setting_side_effects_internal(
    app_handle: &tauri::AppHandle,
    key: &str,
    old_value: &str,
    new_value: &str,
) -> AppResult<()> {
    match key {
        "launch_at_login" => {
            let manager = app_handle.autolaunch();
            if new_value == "true" {
                manager.enable().map_err(|e| {
                    AppError::with_details(
                        ErrorCode::Internal,
                        "无法启用开机启动",
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
                tracing::info!("Autostart enabled");
            } else {
                manager.disable().map_err(|e| {
                    AppError::with_details(
                        ErrorCode::Internal,
                        "无法禁用开机启动",
                        serde_json::json!({ "error": e.to_string() }),
                    )
                })?;
                tracing::info!("Autostart disabled");
            }
        }
        "global_shortcut" => {
            let gs = app_handle.global_shortcut();

            // 注销旧快捷键
            if !old_value.is_empty() {
                if let Ok(old_shortcut) =
                    old_value.parse::<tauri_plugin_global_shortcut::Shortcut>()
                {
                    let _ = gs.unregister(old_shortcut);
                    tracing::info!("Unregistered old shortcut: {}", old_value);
                }
            }

            // 注册新快捷键
            if !new_value.is_empty() {
                let new_shortcut: tauri_plugin_global_shortcut::Shortcut =
                    new_value.parse().map_err(|e| {
                        AppError::with_details(
                            ErrorCode::SettingsInvalidValue,
                            format!("无法解析快捷键 '{}': {}", new_value, e),
                            serde_json::json!({ "shortcut": new_value }),
                        )
                    })?;

                gs.register(new_shortcut).map_err(|e| {
                    AppError::with_details(
                        ErrorCode::SettingsInvalidValue,
                        format!("快捷键 '{}' 注册失败，可能已被其他应用占用", new_value),
                        serde_json::json!({ "shortcut": new_value, "error": e.to_string() }),
                    )
                })?;

                // 验证是否真正注册成功
                if !gs.is_registered(new_shortcut) {
                    return Err(AppError::with_details(
                        ErrorCode::SettingsInvalidValue,
                        format!("快捷键 '{}' 注册后验证失败", new_value),
                        serde_json::json!({ "shortcut": new_value }),
                    ));
                }

                tracing::info!("Registered new shortcut: {}", new_value);
            }
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn get_data_dir(state: State<'_, AppState>) -> AppResult<String> {
    Ok(state.data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_data_dir(state: State<'_, AppState>) -> AppResult<()> {
    crate::services::file_service::FileService::open_in_explorer(&state.data_dir)
}
