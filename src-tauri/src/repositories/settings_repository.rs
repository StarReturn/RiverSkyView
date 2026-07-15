use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::settings::AppSettings;

pub struct SettingsRepository;

impl SettingsRepository {
    pub async fn get_all(pool: &SqlitePool) -> AppResult<AppSettings> {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM app_settings")
            .fetch_all(pool)
            .await?;

        let mut settings = AppSettings::default();
        for (key, value) in rows {
            match key.as_str() {
                "theme" => settings.theme = value,
                "minimize_to_tray" => settings.minimize_to_tray = value == "true",
                "global_shortcut" => {
                    settings.global_shortcut = if value.is_empty() { None } else { Some(value) }
                }
                "launch_at_login" => settings.launch_at_login = value == "true",
                "show_hidden_files" => settings.show_hidden_files = value == "true",
                "markdown_max_size_mb" => {
                    settings.markdown_max_size_mb = value.parse().unwrap_or(2)
                }
                "image_max_size_mb" => settings.image_max_size_mb = value.parse().unwrap_or(20),
                "allow_remote_resources" => settings.allow_remote_resources = value == "true",
                "clipboard_clear_seconds" => {
                    settings.clipboard_clear_seconds = value.parse().unwrap_or(30)
                }
                "vault_auto_mask_seconds" => {
                    settings.vault_auto_mask_seconds = value.parse().unwrap_or(30)
                }
                "default_codex_action" => settings.default_codex_action = value,
                "default_claude_action" => settings.default_claude_action = value,
                "window_width" => settings.window_width = value.parse().ok(),
                "window_height" => settings.window_height = value.parse().ok(),
                "window_x" => settings.window_x = value.parse().ok(),
                "window_y" => settings.window_y = value.parse().ok(),
                "window_maximized" => settings.window_maximized = value == "true",
                "sidebar_collapsed" => settings.sidebar_collapsed = value == "true",
                "file_tree_width" => settings.file_tree_width = value.parse().ok(),
                _ => {}
            }
        }
        Ok(settings)
    }

    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
               ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"#,
        )
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_many(pool: &SqlitePool, settings: &AppSettings) -> AppResult<()> {
        let mut tx = pool.begin().await?;

        let pairs: Vec<(&str, String)> = vec![
            ("theme", settings.theme.clone()),
            ("minimize_to_tray", settings.minimize_to_tray.to_string()),
            (
                "global_shortcut",
                settings.global_shortcut.clone().unwrap_or_default(),
            ),
            ("launch_at_login", settings.launch_at_login.to_string()),
            ("show_hidden_files", settings.show_hidden_files.to_string()),
            (
                "markdown_max_size_mb",
                settings.markdown_max_size_mb.to_string(),
            ),
            ("image_max_size_mb", settings.image_max_size_mb.to_string()),
            (
                "allow_remote_resources",
                settings.allow_remote_resources.to_string(),
            ),
            (
                "clipboard_clear_seconds",
                settings.clipboard_clear_seconds.to_string(),
            ),
            (
                "vault_auto_mask_seconds",
                settings.vault_auto_mask_seconds.to_string(),
            ),
            (
                "default_codex_action",
                settings.default_codex_action.clone(),
            ),
            (
                "default_claude_action",
                settings.default_claude_action.clone(),
            ),
            (
                "window_width",
                settings
                    .window_width
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
            (
                "window_height",
                settings
                    .window_height
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
            (
                "window_x",
                settings.window_x.map(|v| v.to_string()).unwrap_or_default(),
            ),
            (
                "window_y",
                settings.window_y.map(|v| v.to_string()).unwrap_or_default(),
            ),
            ("window_maximized", settings.window_maximized.to_string()),
            ("sidebar_collapsed", settings.sidebar_collapsed.to_string()),
            (
                "file_tree_width",
                settings
                    .file_tree_width
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
        ];

        let now = Utc::now().to_rfc3339();
        for (key, value) in pairs {
            sqlx::query(
                r#"INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                   ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"#,
            )
            .bind(key)
            .bind(&value)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get(pool: &SqlitePool, key: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = ?1")
                .bind(key)
                .fetch_optional(pool)
                .await?;
        Ok(row.map(|(v,)| v))
    }
}
