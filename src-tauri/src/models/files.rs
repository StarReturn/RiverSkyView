use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub is_hidden: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_type: FileType,
    pub content: Option<String>,
    pub size: u64,
    pub encoding: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Markdown,
    Image,
    Svg,
    Text,
    Binary,
    Directory,
    NotFound,
    AccessDenied,
    TooLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePreview {
    pub base64: String,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
