use serde::Serialize;

/// 统一错误码，前端根据此码展示中文信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // 项目相关
    ProjectNotFound,
    ProjectPathNotWritable,
    ProjectAlreadyExists,
    ProjectCanonicalPathConflict,
    ProjectRemovedExists,
    ProjectIdFileWriteFailed,
    ProjectInstructionFileCorrupted,
    ProjectInstructionFileMergeFailed,
    ProjectPmLogDirCreateFailed,

    // 路径安全
    PathEscapeProject,
    PathInvalid,
    PathTooLong,
    PathAccessDenied,
    PathNotFound,

    // 文件
    FileNotFound,
    FileReadFailed,
    FileWriteFailed,
    FileTooLarge,
    FileEncodingInvalid,
    FileTypeUnsupported,

    // 终端
    TerminalLaunchFailed,
    TerminalWtNotFound,
    TerminalExecutableNotFound,

    // 编辑器与 IDE
    EditorNotConfigured,
    EditorNotFound,
    EditorExecutableMissing,
    EditorSelectionRequired,
    EditorTargetInvalid,
    EditorProfileInvalid,
    EditorDetectionTimeout,
    EditorLaunchFailed,

    // 日志
    LogParseFailed,
    LogSyncFailed,

    // 资料库
    VaultEncryptFailed,
    VaultDecryptFailed,
    VaultCiphertextCorrupted,
    VaultNotFound,
    VaultNameRequired,

    // 数据库
    DatabaseInitFailed,
    DatabaseMigrationFailed,
    DatabaseQueryFailed,

    // 设置
    SettingsInvalidValue,

    // 通用
    Internal,
    InvalidParameter,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::ProjectNotFound => "PROJECT_NOT_FOUND",
            ErrorCode::ProjectPathNotWritable => "PROJECT_PATH_NOT_WRITABLE",
            ErrorCode::ProjectAlreadyExists => "PROJECT_ALREADY_EXISTS",
            ErrorCode::ProjectCanonicalPathConflict => "PROJECT_CANONICAL_PATH_CONFLICT",
            ErrorCode::ProjectRemovedExists => "PROJECT_REMOVED_EXISTS",
            ErrorCode::ProjectIdFileWriteFailed => "PROJECT_ID_FILE_WRITE_FAILED",
            ErrorCode::ProjectInstructionFileCorrupted => "PROJECT_INSTRUCTION_FILE_CORRUPTED",
            ErrorCode::ProjectInstructionFileMergeFailed => "PROJECT_INSTRUCTION_FILE_MERGE_FAILED",
            ErrorCode::ProjectPmLogDirCreateFailed => "PROJECT_PM_LOG_DIR_CREATE_FAILED",

            ErrorCode::PathEscapeProject => "PATH_ESCAPE_PROJECT",
            ErrorCode::PathInvalid => "PATH_INVALID",
            ErrorCode::PathTooLong => "PATH_TOO_LONG",
            ErrorCode::PathAccessDenied => "PATH_ACCESS_DENIED",
            ErrorCode::PathNotFound => "PATH_NOT_FOUND",

            ErrorCode::FileNotFound => "FILE_NOT_FOUND",
            ErrorCode::FileReadFailed => "FILE_READ_FAILED",
            ErrorCode::FileWriteFailed => "FILE_WRITE_FAILED",
            ErrorCode::FileTooLarge => "FILE_TOO_LARGE",
            ErrorCode::FileEncodingInvalid => "FILE_ENCODING_INVALID",
            ErrorCode::FileTypeUnsupported => "FILE_TYPE_UNSUPPORTED",

            ErrorCode::TerminalLaunchFailed => "TERMINAL_LAUNCH_FAILED",
            ErrorCode::TerminalWtNotFound => "TERMINAL_WT_NOT_FOUND",
            ErrorCode::TerminalExecutableNotFound => "TERMINAL_EXECUTABLE_NOT_FOUND",

            ErrorCode::EditorNotConfigured => "EDITOR_NOT_CONFIGURED",
            ErrorCode::EditorNotFound => "EDITOR_NOT_FOUND",
            ErrorCode::EditorExecutableMissing => "EDITOR_EXECUTABLE_MISSING",
            ErrorCode::EditorSelectionRequired => "EDITOR_SELECTION_REQUIRED",
            ErrorCode::EditorTargetInvalid => "EDITOR_TARGET_INVALID",
            ErrorCode::EditorProfileInvalid => "EDITOR_PROFILE_INVALID",
            ErrorCode::EditorDetectionTimeout => "EDITOR_DETECTION_TIMEOUT",
            ErrorCode::EditorLaunchFailed => "EDITOR_LAUNCH_FAILED",

            ErrorCode::LogParseFailed => "LOG_PARSE_FAILED",
            ErrorCode::LogSyncFailed => "LOG_SYNC_FAILED",

            ErrorCode::VaultEncryptFailed => "VAULT_ENCRYPT_FAILED",
            ErrorCode::VaultDecryptFailed => "VAULT_DECRYPT_FAILED",
            ErrorCode::VaultCiphertextCorrupted => "VAULT_CIPHERTEXT_CORRUPTED",
            ErrorCode::VaultNotFound => "VAULT_NOT_FOUND",
            ErrorCode::VaultNameRequired => "VAULT_NAME_REQUIRED",

            ErrorCode::DatabaseInitFailed => "DATABASE_INIT_FAILED",
            ErrorCode::DatabaseMigrationFailed => "DATABASE_MIGRATION_FAILED",
            ErrorCode::DatabaseQueryFailed => "DATABASE_QUERY_FAILED",

            ErrorCode::SettingsInvalidValue => "SETTINGS_INVALID_VALUE",

            ErrorCode::Internal => "INTERNAL",
            ErrorCode::InvalidParameter => "INVALID_PARAMETER",
        }
    }
}

/// 统一错误返回结构
#[derive(Debug, Clone)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError {
            code: code.as_str().to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        AppError {
            code: code.as_str().to_string(),
            message: message.into(),
            details: Some(details),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, msg)
    }

    pub fn invalid_param(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidParameter, msg)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        AppError::new(ErrorCode::DatabaseQueryFailed, "数据库操作失败")
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON error: {:?}", err);
        AppError::new(ErrorCode::Internal, "JSON 序列化失败")
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!("IO error: {:?}", err);
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::new(ErrorCode::PathNotFound, "路径不存在"),
            std::io::ErrorKind::PermissionDenied => {
                AppError::new(ErrorCode::PathAccessDenied, "无访问权限")
            }
            _ => AppError::new(ErrorCode::FileWriteFailed, "文件操作失败"),
        }
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        tracing::error!("Tauri error: {:?}", err);
        AppError::new(ErrorCode::Internal, "Tauri 内部错误")
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 3)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("details", &self.details)?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
