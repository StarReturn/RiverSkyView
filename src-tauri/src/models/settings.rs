use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub minimize_to_tray: bool,
    pub global_shortcut: Option<String>,
    pub launch_at_login: bool,
    pub show_hidden_files: bool,
    pub markdown_max_size_mb: u64,
    pub image_max_size_mb: u64,
    pub allow_remote_resources: bool,
    pub clipboard_clear_seconds: u64,
    pub vault_auto_mask_seconds: u64,
    pub default_codex_action: String,
    pub default_claude_action: String,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: bool,
    pub sidebar_collapsed: bool,
    pub file_tree_width: Option<i32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: "system".to_string(),
            minimize_to_tray: true,
            global_shortcut: Some("Ctrl+Alt+P".to_string()),
            launch_at_login: false,
            show_hidden_files: false,
            markdown_max_size_mb: 2,
            image_max_size_mb: 20,
            allow_remote_resources: false,
            clipboard_clear_seconds: 30,
            vault_auto_mask_seconds: 30,
            default_codex_action: "resume_picker".to_string(),
            default_claude_action: "resume_picker".to_string(),
            window_width: Some(1440),
            window_height: Some(900),
            window_x: None,
            window_y: None,
            window_maximized: false,
            sidebar_collapsed: false,
            file_tree_width: Some(300),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdate {
    pub key: String,
    pub value: String,
}
