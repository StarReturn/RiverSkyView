-- 第二期：编辑器与 IDE 项目偏好
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS project_editor_preferences (
    project_id TEXT PRIMARY KEY NOT NULL,
    editor_key TEXT NOT NULL,
    target_relative_path TEXT,
    open_mode TEXT NOT NULL DEFAULT 'default',
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_launch_profiles_editor
    ON launch_profiles(tool_kind, enabled, sort_order);

UPDATE app_meta SET value = '2' WHERE key = 'schema_version';

