use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::settings::AppSettings;
use crate::repositories::settings_repository::SettingsRepository;

pub struct SettingsService {
    pub settings: AppSettings,
}

impl SettingsService {
    pub async fn load(pool: &SqlitePool) -> AppResult<Self> {
        let settings = SettingsRepository::get_all(pool).await?;
        Ok(SettingsService { settings })
    }

    pub async fn save(&self, pool: &SqlitePool) -> AppResult<()> {
        SettingsRepository::set_many(pool, &self.settings).await
    }

    pub async fn update_setting(
        &mut self,
        pool: &SqlitePool,
        key: &str,
        value: &str,
    ) -> AppResult<AppSettings> {
        SettingsRepository::set(pool, key, value).await?;

        // 更新内存中的设置
        match key {
            "theme" => self.settings.theme = value.to_string(),
            "minimize_to_tray" => self.settings.minimize_to_tray = value == "true",
            "global_shortcut" => {
                self.settings.global_shortcut = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "launch_at_login" => self.settings.launch_at_login = value == "true",
            "show_hidden_files" => self.settings.show_hidden_files = value == "true",
            "markdown_max_size_mb" => {
                if let Ok(v) = value.parse() {
                    self.settings.markdown_max_size_mb = v
                }
            }
            "image_max_size_mb" => {
                if let Ok(v) = value.parse() {
                    self.settings.image_max_size_mb = v
                }
            }
            "allow_remote_resources" => self.settings.allow_remote_resources = value == "true",
            "clipboard_clear_seconds" => {
                if let Ok(v) = value.parse() {
                    self.settings.clipboard_clear_seconds = v
                }
            }
            "vault_auto_mask_seconds" => {
                if let Ok(v) = value.parse() {
                    self.settings.vault_auto_mask_seconds = v
                }
            }
            "default_codex_action" => self.settings.default_codex_action = value.to_string(),
            "default_claude_action" => self.settings.default_claude_action = value.to_string(),
            "window_width" => self.settings.window_width = value.parse().ok(),
            "window_height" => self.settings.window_height = value.parse().ok(),
            "window_x" => self.settings.window_x = value.parse().ok(),
            "window_y" => self.settings.window_y = value.parse().ok(),
            "window_maximized" => self.settings.window_maximized = value == "true",
            "sidebar_collapsed" => self.settings.sidebar_collapsed = value == "true",
            "file_tree_width" => self.settings.file_tree_width = value.parse().ok(),
            _ => {}
        }

        Ok(self.settings.clone())
    }
}
