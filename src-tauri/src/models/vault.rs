use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub source_filename: Option<String>,
    pub encrypted_path: String,
    pub tags_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub removed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultListItem {
    pub id: String,
    pub name: String,
    pub source_filename: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub removed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultCreateRequest {
    pub name: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub source_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultUpdateRequest {
    pub id: String,
    pub name: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultImportRequest {
    pub name: String,
    pub source_file_path: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultContent {
    pub id: String,
    pub name: String,
    pub content: String,
    pub tags: Vec<String>,
    pub updated_at: String,
}
