-- 1.1.1：内置开发工具手动路径与自动检测结果分离存储
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS editor_installations (
    editor_key TEXT PRIMARY KEY NOT NULL,
    manual_executable TEXT,
    detected_executable TEXT,
    active_source TEXT NOT NULL DEFAULT 'auto'
        CHECK (active_source IN ('auto', 'manual')),
    enabled INTEGER NOT NULL DEFAULT 1,
    verification_status TEXT NOT NULL DEFAULT 'unknown'
        CHECK (verification_status IN ('unknown', 'valid', 'missing', 'invalid', 'launch_failed')),
    detected_source TEXT,
    version TEXT,
    last_detected_at TEXT,
    last_verified_at TEXT,
    last_error TEXT,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_editor_installations_status
    ON editor_installations(enabled, verification_status);

UPDATE app_meta SET value = '3' WHERE key = 'schema_version';
