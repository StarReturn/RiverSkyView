use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EditorProfile {
    pub id: String,
    pub project_id: Option<String>,
    pub name: String,
    pub executable: String,
    pub args_json: String,
    pub working_directory: Option<String>,
    pub sort_order: i64,
    pub enabled: bool,
}

impl EditorProfile {
    pub fn editor_key(&self) -> String {
        format!("custom:{}", self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorProfileInput {
    pub project_id: Option<String>,
    pub name: String,
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    pub sort_order: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectEditorPreference {
    pub project_id: String,
    pub editor_key: String,
    pub target_relative_path: Option<String>,
    pub open_mode: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPreferenceInput {
    pub editor_key: String,
    pub target_relative_path: Option<String>,
    pub open_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorDescriptor {
    pub key: String,
    pub name: String,
    pub family: String,
    pub available: bool,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub source: Option<String>,
    pub supports_open_mode: bool,
    pub supports_solution_target: bool,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EditorInstallationRecord {
    pub editor_key: String,
    pub manual_executable: Option<String>,
    pub detected_executable: Option<String>,
    pub active_source: String,
    pub enabled: bool,
    pub verification_status: String,
    pub detected_source: Option<String>,
    pub version: Option<String>,
    pub last_detected_at: Option<String>,
    pub last_verified_at: Option<String>,
    pub last_error: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorInstallation {
    pub editor_key: String,
    pub name: String,
    pub family: String,
    pub manual_executable: Option<String>,
    pub detected_executable: Option<String>,
    pub active_executable: Option<String>,
    pub active_source: String,
    pub available: bool,
    pub enabled: bool,
    pub verification_status: String,
    pub detected_source: Option<String>,
    pub version: Option<String>,
    pub last_detected_at: Option<String>,
    pub last_verified_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTestLaunchResult {
    pub editor_key: String,
    pub executable: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTarget {
    pub relative_path: Option<String>,
    pub display_name: String,
    pub kind: String,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEditorRequest {
    pub project_id: String,
    pub editor_key: Option<String>,
    pub target_relative_path: Option<String>,
    pub open_mode: Option<String>,
    pub remember_for_project: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEditorResult {
    pub editor_key: String,
    pub editor_name: String,
    pub executable: String,
    pub target_display: String,
    pub used_project_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub default_editor_key: Option<String>,
    pub open_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettingsInput {
    pub default_editor_key: Option<String>,
    pub open_mode: String,
}
