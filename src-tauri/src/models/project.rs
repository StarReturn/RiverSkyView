use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub canonical_path: String,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub removed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_favorite: bool,
    pub created_at: String,
    pub last_opened_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub removed_at: Option<String>,
    /// 目录当前是否可用（运行时检查）
    pub directory_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProjectRequest {
    pub path: String,
    pub name: Option<String>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProjectResult {
    pub project: Project,
    pub agents_md_created: bool,
    pub agents_md_updated: bool,
    pub claude_md_created: bool,
    pub claude_md_updated: bool,
    pub pm_log_created: bool,
    pub id_file_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameProjectRequest {
    pub project_id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdentifier {
    pub schema_version: u32,
    pub project_id: String,
    pub created_at: String,
}
