use std::path::Path;
use std::process::Command;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::terminal::*;

pub struct TerminalService;

impl TerminalService {
    /// 启动终端或 CLI 智能体
    pub fn launch(
        project_path: &Path,
        project_name: &str,
        tool_kind: &str,
        action_kind: &str,
    ) -> AppResult<LaunchResult> {
        let (executable, args) =
            Self::build_command(project_path, project_name, tool_kind, action_kind)?;

        // 检查可执行文件是否可用
        if !Self::is_executable_available(&executable) {
            // wt.exe 不可用时尝试回退
            if executable == "wt.exe" {
                return Self::launch_fallback(
                    project_path,
                    project_name,
                    tool_kind,
                    action_kind,
                    &args,
                );
            }
            return Err(AppError::with_details(
                ErrorCode::TerminalExecutableNotFound,
                format!("未检测到可执行文件: {}", executable),
                serde_json::json!({ "executable": executable }),
            ));
        }

        let working_dir = project_path.to_string_lossy().to_string();

        tracing::info!(
            "Launching terminal: {} with args {:?} in {}",
            executable,
            args,
            working_dir
        );

        let mut cmd = Command::new(&executable);
        cmd.args(&args);
        cmd.current_dir(&working_dir);

        match cmd.spawn() {
            Ok(_) => Ok(LaunchResult {
                success: true,
                message: "启动成功".to_string(),
                executable,
                args,
                working_directory: working_dir,
            }),
            Err(e) => Err(AppError::with_details(
                ErrorCode::TerminalLaunchFailed,
                "启动终端失败",
                serde_json::json!({ "executable": executable, "error": e.to_string() }),
            )),
        }
    }

    /// 构建命令参数数组
    fn build_command(
        project_path: &Path,
        project_name: &str,
        tool_kind: &str,
        action_kind: &str,
    ) -> AppResult<(String, Vec<String>)> {
        let project_path_str = project_path.to_string_lossy().to_string();

        let (exe, template_args) = match tool_kind {
            "cmd" => DefaultLaunchConfig::cmd(),
            "powershell" => DefaultLaunchConfig::powershell(),
            "codex" => match action_kind {
                "new" => DefaultLaunchConfig::codex_new(),
                "resume_picker" => DefaultLaunchConfig::codex_resume_picker(),
                "continue_latest" => DefaultLaunchConfig::codex_continue_latest(),
                _ => DefaultLaunchConfig::codex_new(),
            },
            "claude" => match action_kind {
                "new" => DefaultLaunchConfig::claude_new(),
                "resume_picker" => DefaultLaunchConfig::claude_resume_picker(),
                "continue_latest" => DefaultLaunchConfig::claude_continue_latest(),
                _ => DefaultLaunchConfig::claude_new(),
            },
            "vscode" => DefaultLaunchConfig::vscode(),
            "cursor" => DefaultLaunchConfig::cursor(),
            _ => {
                return Err(AppError::with_details(
                    ErrorCode::InvalidParameter,
                    "未知的工具类型",
                    serde_json::json!({ "tool_kind": tool_kind }),
                ));
            }
        };

        // 替换占位符
        let args: Vec<String> = template_args
            .iter()
            .map(|arg| {
                arg.replace("{projectPath}", &project_path_str)
                    .replace("{projectName}", project_name)
            })
            .collect();

        Ok((exe, args))
    }

    /// wt.exe 不可用时的回退方案
    fn launch_fallback(
        project_path: &Path,
        _project_name: &str,
        tool_kind: &str,
        action_kind: &str,
        _original_args: &[String],
    ) -> AppResult<LaunchResult> {
        let working_dir = project_path.to_string_lossy().to_string();

        let (executable, args) = match tool_kind {
            "cmd" | "powershell" => {
                // 直接启动 cmd.exe 或 powershell.exe
                let exe = if tool_kind == "powershell" {
                    "powershell.exe"
                } else {
                    "cmd.exe"
                };
                (exe.to_string(), vec![])
            }
            "codex" => {
                // 直接启动 codex
                let mut args = vec![];
                if action_kind == "resume_picker" {
                    args.push("resume".to_string());
                } else if action_kind == "continue_latest" {
                    args.push("resume".to_string());
                    args.push("--last".to_string());
                }
                ("codex".to_string(), args)
            }
            "claude" => {
                let mut args = vec![];
                if action_kind == "resume_picker" {
                    args.push("-r".to_string());
                } else if action_kind == "continue_latest" {
                    args.push("-c".to_string());
                }
                ("claude".to_string(), args)
            }
            "vscode" => DefaultLaunchConfig::vscode(),
            "cursor" => DefaultLaunchConfig::cursor(),
            _ => {
                return Err(AppError::new(
                    ErrorCode::TerminalWtNotFound,
                    "Windows Terminal 不可用且无回退方案",
                ));
            }
        };

        if !Self::is_executable_available(&executable) {
            return Err(AppError::with_details(
                ErrorCode::TerminalExecutableNotFound,
                format!("未检测到可执行文件: {}", executable),
                serde_json::json!({ "executable": executable }),
            ));
        }

        let mut cmd = Command::new(&executable);
        cmd.args(&args);
        cmd.current_dir(&working_dir);

        match cmd.spawn() {
            Ok(_) => Ok(LaunchResult {
                success: true,
                message: "启动成功（回退模式）".to_string(),
                executable,
                args,
                working_directory: working_dir,
            }),
            Err(e) => Err(AppError::with_details(
                ErrorCode::TerminalLaunchFailed,
                "启动终端失败（回退模式）",
                serde_json::json!({ "executable": executable, "error": e.to_string() }),
            )),
        }
    }

    /// 检查可执行文件是否可用
    pub fn is_executable_available(exe: &str) -> bool {
        // 尝试在 PATH 中查找
        if which::which(exe).is_ok() {
            return true;
        }

        // Windows 上检查常见路径
        #[cfg(windows)]
        {
            if exe == "wt.exe" {
                // 检查 Windows Terminal 安装路径
                let _candidates = [
                    r"C:\Program Files\WindowsApps",
                    r"C:\Users\*\AppData\Local\Microsoft\WindowsApps",
                ];
                // 简化检查：如果 where 命令能找到就算可用
                if std::process::Command::new("where")
                    .arg("wt.exe")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    return true;
                }
            }
        }

        false
    }

    /// 检查所有工具可用性
    pub fn check_tools() -> Vec<ToolAvailability> {
        let tools = [
            ("cmd", "wt.exe"),
            ("powershell", "wt.exe"),
            ("codex", "codex"),
            ("claude", "claude"),
            ("vscode", "code"),
            ("cursor", "cursor"),
        ];

        tools
            .into_iter()
            .map(|(tool_kind, exe)| ToolAvailability {
                tool_kind: tool_kind.to_string(),
                available: Self::is_executable_available(exe),
                executable: exe.to_string(),
                version: None,
            })
            .collect()
    }

    /// 检查工具版本（用于诊断）
    pub fn get_version(exe: &str) -> Option<String> {
        let output = Command::new(exe).arg("--version").output().ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_cmd() {
        let (exe, args) = TerminalService::build_command(
            Path::new("D:\\Test Project"),
            "TestProject",
            "cmd",
            "new",
        )
        .unwrap();

        assert_eq!(exe, "wt.exe");
        assert!(args.contains(&"cmd.exe".to_string()));
        assert!(args.contains(&"D:\\Test Project".to_string()));
    }

    #[test]
    fn test_build_command_codex_resume() {
        let (exe, args) =
            TerminalService::build_command(Path::new("D:\\Test"), "Test", "codex", "resume_picker")
                .unwrap();

        assert_eq!(exe, "wt.exe");
        assert!(args.contains(&"resume".to_string()));
    }

    #[test]
    fn test_build_command_claude_continue() {
        let (exe, args) = TerminalService::build_command(
            Path::new("D:\\Test"),
            "Test",
            "claude",
            "continue_latest",
        )
        .unwrap();

        assert_eq!(exe, "wt.exe");
        assert!(args.contains(&"-c".to_string()));
    }

    #[test]
    fn test_build_command_unknown_tool() {
        let result =
            TerminalService::build_command(Path::new("D:\\Test"), "Test", "unknown", "new");
        assert!(result.is_err());
    }

    #[test]
    fn test_placeholder_replacement() {
        let (_exe, args) = TerminalService::build_command(
            Path::new("D:\\My Project"),
            "MyProject",
            "codex",
            "new",
        )
        .unwrap();

        // 确保占位符被替换
        assert!(!args.iter().any(|a| a.contains("{projectPath}")));
        assert!(!args.iter().any(|a| a.contains("{projectName}")));
        assert!(args.iter().any(|a| a.contains("My Project")));
    }

    #[test]
    fn test_no_shell_injection() {
        // 确保项目路径中的特殊字符不会被解释为 shell 命令
        let malicious_path = Path::new("D:\\Test & del /f /q C:\\*");
        let (exe, args) =
            TerminalService::build_command(malicious_path, "Test", "cmd", "new").unwrap();

        // 路径应该作为单个参数传递，不被分割
        assert!(args.iter().any(|a| a.contains("del /f /q")));
        // 不应该有额外的命令
        assert_eq!(exe, "wt.exe");
    }
}
