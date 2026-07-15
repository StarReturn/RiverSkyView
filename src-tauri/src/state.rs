use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::settings_service::SettingsService;

/// 应用共享状态，通过 Tauri State 注入到各命令
pub struct AppState {
    pub db: SqlitePool,
    pub data_dir: std::path::PathBuf,
    pub vault_dir: std::path::PathBuf,
    pub thumbnails_dir: std::path::PathBuf,
    pub logs_dir: std::path::PathBuf,
    pub backups_dir: std::path::PathBuf,
    pub settings: Arc<RwLock<SettingsService>>,
}

impl AppState {
    pub async fn new(data_dir: std::path::PathBuf) -> Result<Self, crate::error::AppError> {
        // 确保所有子目录存在
        let vault_dir = data_dir.join("encrypted-vault");
        let thumbnails_dir = data_dir.join("thumbnails");
        let logs_dir = data_dir.join("logs");
        let backups_dir = data_dir.join("backups");

        for dir in [
            &data_dir,
            &vault_dir,
            &thumbnails_dir,
            &logs_dir,
            &backups_dir,
        ] {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| {
                    crate::error::AppError::with_details(
                        crate::error::ErrorCode::DatabaseInitFailed,
                        "无法创建应用数据目录",
                        serde_json::json!({
                            "dir": dir.to_string_lossy(),
                            "error": e.to_string(),
                        }),
                    )
                })?;
            }
        }

        // 初始化日志
        let log_file = tracing_appender::rolling::daily(&logs_dir, "app.log");
        let (log_writer, _guard) = tracing_appender::non_blocking(log_file);
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_writer(log_writer)
            .with_ansi(false)
            .try_init()
            .ok();

        // 初始化数据库
        let db_path = data_dir.join("project-manager.db");
        let db_url = format!(
            "sqlite://{}?mode=rwc",
            db_path.to_string_lossy().replace('\\', "/")
        );
        tracing::info!("Database URL: {}", db_url);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| {
                crate::error::AppError::with_details(
                    crate::error::ErrorCode::DatabaseInitFailed,
                    "数据库连接失败",
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        // 设置 PRAGMA
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(|e| {
                crate::error::AppError::with_details(
                    crate::error::ErrorCode::DatabaseInitFailed,
                    "设置 WAL 模式失败",
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;

        // 执行迁移
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| {
                crate::error::AppError::with_details(
                    crate::error::ErrorCode::DatabaseMigrationFailed,
                    "数据库迁移失败",
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        // 加载设置
        let settings = SettingsService::load(&pool).await?;

        Ok(AppState {
            db: pool,
            data_dir,
            vault_dir,
            thumbnails_dir,
            logs_dir,
            backups_dir,
            settings: Arc::new(RwLock::new(settings)),
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;
    use tempfile::TempDir;

    #[tokio::test]
    async fn migrations_create_editor_preferences_and_installations() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("migration.db");
        let url = format!(
            "sqlite://{}?mode=rwc",
            db_path.to_string_lossy().replace('\\', "/")
        );
        let pool = sqlx::SqlitePool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let row = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'project_editor_preferences'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<String, _>("name"), "project_editor_preferences");
        let row = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'editor_installations'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<String, _>("name"), "editor_installations");
        let version: String =
            sqlx::query_scalar("SELECT value FROM app_meta WHERE key = 'schema_version'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(version, "3");
    }
}
