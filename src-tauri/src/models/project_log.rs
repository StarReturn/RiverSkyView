use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectLog {
    pub id: String,
    pub project_id: String,
    pub relative_path: String,
    pub content_hash: String,
    pub agent: String,
    pub status: String,
    pub title: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: String,
    pub time_inferred: bool,
    pub parse_status: String,
    pub parse_error: Option<String>,
    pub indexed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLog {
    pub agent: String,
    pub status: String,
    pub title: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: String,
    pub time_inferred: bool,
    pub parse_status: String,
    pub parse_error: Option<String>,
    pub content_hash: String,
    pub raw_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSyncResult {
    pub project_id: String,
    pub scanned: u64,
    pub added: u64,
    pub updated: u64,
    pub removed: u64,
    pub errors: u64,
    pub last_synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    pub date: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub total_tasks: u64,
    pub completed: u64,
    pub failed: u64,
    pub blocked: u64,
    pub heatmap: Vec<HeatmapCell>,
    pub period_start: String,
    pub period_end: String,
}
