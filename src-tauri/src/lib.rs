pub mod commands;
pub mod error;
pub mod models;
pub mod repositories;
pub mod security;
pub mod services;
pub mod state;

use std::path::PathBuf;
use tauri::{Manager, PhysicalPosition, PhysicalSize, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        tracing::info!("Shortcut pressed: {:?}", shortcut);
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            let data_dir = get_data_dir().expect("failed to get data dir");

            let state = tauri::async_runtime::block_on(async { AppState::new(data_dir).await })?;

            let vault_dir = state.vault_dir.clone();
            let vault_service = services::vault_service::VaultService::new(vault_dir.clone());
            vault_service.cleanup_temp_files();

            // 读取设置中的快捷键和窗口状态
            let settings_snapshot = {
                let settings_guard = state.settings.blocking_read();
                settings_guard.settings.clone()
            };

            // 注册全局快捷键（从设置读取，而非硬编码）
            if let Some(ref shortcut_str) = settings_snapshot.global_shortcut {
                if !shortcut_str.is_empty() {
                    if let Err(e) = register_global_shortcut(app, shortcut_str) {
                        tracing::warn!(
                            "Failed to register global shortcut '{}': {}",
                            shortcut_str,
                            e
                        );
                    }
                }
            }

            // 应用开机启动设置
            if settings_snapshot.launch_at_login {
                if let Err(e) = app.autolaunch().enable() {
                    tracing::warn!("Failed to enable autostart: {}", e);
                }
            }

            app.manage(state);

            // 恢复窗口状态
            restore_window_state(app, &settings_snapshot);

            // 设置托盘图标
            if let Err(e) = setup_tray(app) {
                tracing::error!("Failed to setup tray: {:?}", e);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let settings = state.settings.blocking_read();
                    if settings.settings.minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
            WindowEvent::Destroyed => {
                save_window_state(window);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::editor::list_editors,
            commands::editor::refresh_editor_detection,
            commands::editor::resolve_editor_targets,
            commands::editor::get_editor_settings,
            commands::editor::set_editor_settings,
            commands::editor::get_project_editor_preference,
            commands::editor::set_project_editor_preference,
            commands::editor::clear_project_editor_preference,
            commands::editor::list_editor_profiles,
            commands::editor::create_editor_profile,
            commands::editor::update_editor_profile,
            commands::editor::delete_editor_profile,
            commands::editor::launch_project_editor,
            commands::project::add_project,
            commands::project::list_projects,
            commands::project::list_favorite_projects,
            commands::project::list_recent_projects,
            commands::project::list_removed_projects,
            commands::project::search_projects,
            commands::project::get_project,
            commands::project::rename_project,
            commands::project::set_project_favorite,
            commands::project::remove_project,
            commands::project::batch_remove_projects,
            commands::project::restore_project,
            commands::project::check_instruction_files,
            commands::project::count_projects,
            commands::files::list_directory,
            commands::files::read_file_for_preview,
            commands::files::get_absolute_path,
            commands::files::open_in_explorer,
            commands::files::reveal_in_explorer,
            commands::files::copy_file_to_clipboard,
            commands::terminal::launch_terminal,
            commands::terminal::check_tools_availability,
            commands::terminal::launch_codex,
            commands::terminal::launch_claude,
            commands::terminal::set_project_default_action,
            commands::terminal::get_project_default_action,
            commands::project_log::sync_project_logs,
            commands::project_log::list_project_logs,
            commands::project_log::get_log_content,
            commands::project_log::get_activity_summary,
            commands::vault::create_vault_entry,
            commands::vault::import_vault_txt,
            commands::vault::get_vault_content,
            commands::vault::update_vault_entry,
            commands::vault::list_vault_entries,
            commands::vault::list_removed_vault_entries,
            commands::vault::search_vault_entries,
            commands::vault::remove_vault_entry,
            commands::vault::restore_vault_entry,
            commands::vault::permanent_delete_vault_entry,
            commands::vault::clear_vault_plaintext,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::update_settings,
            commands::settings::apply_setting_side_effects,
            commands::settings::get_data_dir,
            commands::settings::open_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 注册全局快捷键
fn register_global_shortcut(
    app: &tauri::App,
    shortcut_str: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut: tauri_plugin_global_shortcut::Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("无法解析快捷键 '{}': {}", shortcut_str, e))?;

    app.global_shortcut().register(shortcut)?;
    tracing::info!("Registered global shortcut: {}", shortcut_str);
    Ok(())
}

/// 注销全局快捷键
#[allow(dead_code)]
fn unregister_global_shortcut(
    app: &tauri::App,
    shortcut_str: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut: tauri_plugin_global_shortcut::Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("无法解析快捷键 '{}': {}", shortcut_str, e))?;

    app.global_shortcut().unregister(shortcut)?;
    tracing::info!("Unregistered global shortcut: {}", shortcut_str);
    Ok(())
}

/// 恢复窗口状态（尺寸、位置、最大化）
fn restore_window_state(app: &tauri::App, settings: &models::settings::AppSettings) {
    if let Some(window) = app.get_webview_window("main") {
        if settings.window_maximized {
            let _ = window.maximize();
            return;
        }

        // 恢复尺寸
        if let (Some(w), Some(h)) = (settings.window_width, settings.window_height) {
            if w > 200 && h > 150 {
                let _ = window.set_size(PhysicalSize::new(w as u32, h as u32));
            }
        }

        // 恢复位置（检查是否在可见屏幕范围内）
        if let (Some(x), Some(y)) = (settings.window_x, settings.window_y) {
            if is_position_visible(app, x, y) {
                let _ = window.set_position(PhysicalPosition::new(x, y));
            } else {
                // 位置不在可见屏幕上，居中窗口
                let _ = window.center();
            }
        }
    }
}

/// 检查窗口位置是否在某个可见显示器上
fn is_position_visible(app: &tauri::App, x: i32, y: i32) -> bool {
    if let Ok(monitors) = app.available_monitors() {
        for monitor in monitors {
            let pos = monitor.position();
            let size = monitor.size();
            let mx = pos.x;
            let my = pos.y;
            let mw = size.width as i32;
            let mh = size.height as i32;
            // 检查窗口左上角是否在显示器的合理范围内（允许部分超出）
            if x >= mx - 100 && x <= mx + mw - 100 && y >= my - 50 && y <= my + mh - 50 {
                return true;
            }
        }
    }
    false
}

/// 保存窗口状态到数据库
fn save_window_state(window: &tauri::Window) {
    let app = window.app_handle();
    if let Some(state) = app.try_state::<AppState>() {
        let pool = state.db.clone();

        let is_maximized = window.is_maximized().unwrap_or(false);
        let size = window.outer_size().unwrap_or_default();
        let pos = window.outer_position().unwrap_or_default();

        tauri::async_runtime::block_on(async {
            let _ = repositories::settings_repository::SettingsRepository::set(
                &pool,
                "window_maximized",
                &is_maximized.to_string(),
            )
            .await;
            let _ = repositories::settings_repository::SettingsRepository::set(
                &pool,
                "window_width",
                &size.width.to_string(),
            )
            .await;
            let _ = repositories::settings_repository::SettingsRepository::set(
                &pool,
                "window_height",
                &size.height.to_string(),
            )
            .await;
            let _ = repositories::settings_repository::SettingsRepository::set(
                &pool,
                "window_x",
                &pos.x.to_string(),
            )
            .await;
            let _ = repositories::settings_repository::SettingsRepository::set(
                &pool,
                "window_y",
                &pos.y.to_string(),
            )
            .await;
        });
    }
}

fn get_data_dir() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let dir = dirs::data_local_dir()
        .ok_or("无法确定本地数据目录")?
        .join("ProjectManager");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("江天一览 RiverSkyView")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    Ok(())
}
