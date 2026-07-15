use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolKind {
    Cmd,
    Powershell,
    Codex,
    Claude,
    Vscode,
    Cursor,
    Custom,
}

impl ToolKind {
    pub fn parse_str(s: &str) -> Self {
        match s {
            "cmd" => ToolKind::Cmd,
            "powershell" => ToolKind::Powershell,
            "codex" => ToolKind::Codex,
            "claude" => ToolKind::Claude,
            "vscode" => ToolKind::Vscode,
            "cursor" => ToolKind::Cursor,
            _ => ToolKind::Custom,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ToolKind::Cmd => "cmd",
            ToolKind::Powershell => "powershell",
            ToolKind::Codex => "codex",
            ToolKind::Claude => "claude",
            ToolKind::Vscode => "vscode",
            ToolKind::Cursor => "cursor",
            ToolKind::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionKind {
    New,
    ResumePicker,
    ContinueLatest,
    Custom,
}

impl ActionKind {
    pub fn parse_str(s: &str) -> Self {
        match s {
            "new" => ActionKind::New,
            "resume_picker" => ActionKind::ResumePicker,
            "continue_latest" => ActionKind::ContinueLatest,
            _ => ActionKind::Custom,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ActionKind::New => "new",
            ActionKind::ResumePicker => "resume_picker",
            ActionKind::ContinueLatest => "continue_latest",
            ActionKind::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LaunchProfile {
    pub id: String,
    pub project_id: Option<String>,
    pub name: String,
    pub tool_kind: String,
    pub action_kind: String,
    pub executable: String,
    pub args_json: String,
    pub working_directory: Option<String>,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub project_id: String,
    pub tool_kind: String,
    pub action_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAvailability {
    pub tool_kind: String,
    pub available: bool,
    pub executable: String,
    pub version: Option<String>,
}

/// 默认启动配置（不保存到数据库，运行时生成）
pub struct DefaultLaunchConfig;

impl DefaultLaunchConfig {
    pub fn cmd() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "cmd.exe".to_string(),
            ],
        )
    }

    pub fn powershell() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "powershell.exe".to_string(),
            ],
        )
    }

    pub fn codex_new() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "--title".to_string(),
                "Codex · {projectName}".to_string(),
                "codex".to_string(),
            ],
        )
    }

    pub fn codex_resume_picker() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "--title".to_string(),
                "Codex · {projectName}".to_string(),
                "codex".to_string(),
                "resume".to_string(),
            ],
        )
    }

    pub fn codex_continue_latest() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "codex".to_string(),
                "resume".to_string(),
                "--last".to_string(),
            ],
        )
    }

    pub fn claude_new() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "--title".to_string(),
                "Claude · {projectName}".to_string(),
                "claude".to_string(),
            ],
        )
    }

    pub fn claude_resume_picker() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "--title".to_string(),
                "Claude · {projectName}".to_string(),
                "claude".to_string(),
                "-r".to_string(),
            ],
        )
    }

    pub fn claude_continue_latest() -> (String, Vec<String>) {
        (
            "wt.exe".to_string(),
            vec![
                "-w".to_string(),
                "0".to_string(),
                "new-tab".to_string(),
                "-d".to_string(),
                "{projectPath}".to_string(),
                "claude".to_string(),
                "-c".to_string(),
            ],
        )
    }

    pub fn vscode() -> (String, Vec<String>) {
        ("code".to_string(), vec!["{projectPath}".to_string()])
    }

    pub fn cursor() -> (String, Vec<String>) {
        ("cursor".to_string(), vec!["{projectPath}".to_string()])
    }

    /// wt.exe 不可用时 cmd 的回退
    pub fn cmd_fallback() -> (String, Vec<String>) {
        ("cmd.exe".to_string(), vec!["/K".to_string()])
    }
}
