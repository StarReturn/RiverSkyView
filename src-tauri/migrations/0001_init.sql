-- 初始数据库架构 v1
-- 本地项目桌面管理系统

-- 启用外键约束（也在连接时设置）
PRAGMA foreign_keys = ON;

-- 项目表
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    canonical_path TEXT NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_opened_at TEXT,
    last_activity_at TEXT,
    removed_at TEXT
);

-- canonical_path 唯一索引（仅未移除项目）
-- SQLite 不支持部分唯一索引的 IF NOT EXISTS，直接创建
CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_canonical_active
    ON projects(canonical_path)
    WHERE removed_at IS NULL;

-- 路径索引（用于搜索）
CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
CREATE INDEX IF NOT EXISTS idx_projects_removed ON projects(removed_at);
CREATE INDEX IF NOT EXISTS idx_projects_favorite ON projects(is_favorite) WHERE removed_at IS NULL;

-- 启动配置表
CREATE TABLE IF NOT EXISTS launch_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT,
    name TEXT NOT NULL,
    tool_kind TEXT NOT NULL DEFAULT 'custom',
    action_kind TEXT NOT NULL DEFAULT 'custom',
    executable TEXT NOT NULL,
    args_json TEXT NOT NULL DEFAULT '[]',
    working_directory TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_launch_profiles_project ON launch_profiles(project_id);
CREATE INDEX IF NOT EXISTS idx_launch_profiles_global ON launch_profiles(project_id) WHERE project_id IS NULL;

-- 项目默认 CLI 动作
CREATE TABLE IF NOT EXISTS project_cli_defaults (
    project_id TEXT PRIMARY KEY NOT NULL,
    default_codex_action TEXT NOT NULL DEFAULT 'resume_picker',
    default_claude_action TEXT NOT NULL DEFAULT 'resume_picker',
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 项目日志表
CREATE TABLE IF NOT EXISTS project_logs (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    agent TEXT NOT NULL DEFAULT 'other',
    status TEXT NOT NULL DEFAULT 'completed',
    title TEXT,
    started_at TEXT,
    finished_at TEXT NOT NULL,
    time_inferred INTEGER NOT NULL DEFAULT 0,
    parse_status TEXT NOT NULL DEFAULT 'valid',
    parse_error TEXT,
    indexed_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 日志唯一约束：project_id + relative_path
CREATE UNIQUE INDEX IF NOT EXISTS idx_project_logs_unique
    ON project_logs(project_id, relative_path);
CREATE INDEX IF NOT EXISTS idx_project_logs_project ON project_logs(project_id);
CREATE INDEX IF NOT EXISTS idx_project_logs_finished ON project_logs(finished_at);
CREATE INDEX IF NOT EXISTS idx_project_logs_agent ON project_logs(agent);
CREATE INDEX IF NOT EXISTS idx_project_logs_status ON project_logs(status);

-- 服务器资料表（元数据，密文在文件系统）
CREATE TABLE IF NOT EXISTS vault_entries (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source_filename TEXT,
    encrypted_path TEXT NOT NULL,
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    removed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_vault_name ON vault_entries(name);
CREATE INDEX IF NOT EXISTS idx_vault_removed ON vault_entries(removed_at);

-- 应用设置表（键值对）
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 应用元数据表
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO app_meta (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO app_meta (key, value) VALUES ('created_at', datetime('now'));
