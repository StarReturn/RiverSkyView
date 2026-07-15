use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

use crate::models::editor::{EditorDescriptor, EditorProfile};

#[derive(Debug, Clone)]
pub struct BuiltinEditorSpec {
    pub key: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub commands: &'static [&'static str],
    pub supports_open_mode: bool,
    pub supports_solution_target: bool,
}

pub struct EditorDetectionService;

impl EditorDetectionService {
    pub fn specs() -> Vec<BuiltinEditorSpec> {
        vec![
            BuiltinEditorSpec {
                key: "builtin:vscode",
                name: "Visual Studio Code",
                family: "vscode",
                commands: &["code", "Code.exe"],
                supports_open_mode: true,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:vscode-insiders",
                name: "VS Code Insiders",
                family: "vscode",
                commands: &["code-insiders", "Code - Insiders.exe"],
                supports_open_mode: true,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:cursor",
                name: "Cursor",
                family: "cursor",
                commands: &["cursor", "Cursor.exe"],
                supports_open_mode: true,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:visual-studio",
                name: "Visual Studio 2022",
                family: "visual-studio",
                commands: &["devenv.exe"],
                supports_open_mode: false,
                supports_solution_target: true,
            },
            BuiltinEditorSpec {
                key: "builtin:idea",
                name: "IntelliJ IDEA",
                family: "jetbrains",
                commands: &["idea64.exe", "idea"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:webstorm",
                name: "WebStorm",
                family: "jetbrains",
                commands: &["webstorm64.exe", "webstorm"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:pycharm",
                name: "PyCharm",
                family: "jetbrains",
                commands: &["pycharm64.exe", "pycharm"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:rider",
                name: "JetBrains Rider",
                family: "jetbrains",
                commands: &["rider64.exe", "rider"],
                supports_open_mode: false,
                supports_solution_target: true,
            },
            BuiltinEditorSpec {
                key: "builtin:clion",
                name: "CLion",
                family: "jetbrains",
                commands: &["clion64.exe", "clion"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:goland",
                name: "GoLand",
                family: "jetbrains",
                commands: &["goland64.exe", "goland"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:rustrover",
                name: "RustRover",
                family: "jetbrains",
                commands: &["rustrover64.exe", "rustrover"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
            BuiltinEditorSpec {
                key: "builtin:android-studio",
                name: "Android Studio",
                family: "android-studio",
                commands: &["studio64.exe", "studio"],
                supports_open_mode: false,
                supports_solution_target: false,
            },
        ]
    }

    pub fn list_builtins() -> Vec<EditorDescriptor> {
        Self::specs().iter().map(Self::detect_builtin).collect()
    }

    pub fn detect_builtin(spec: &BuiltinEditorSpec) -> EditorDescriptor {
        let detected = Self::find_executable(spec);
        EditorDescriptor {
            key: spec.key.to_string(),
            name: spec.name.to_string(),
            family: spec.family.to_string(),
            available: detected.is_some(),
            version: None,
            executable: detected
                .as_ref()
                .map(|(path, _)| path.to_string_lossy().to_string()),
            source: detected.map(|(_, source)| source),
            supports_open_mode: spec.supports_open_mode,
            supports_solution_target: spec.supports_solution_target,
            is_custom: false,
        }
    }

    pub fn find_builtin(key: &str) -> Option<EditorDescriptor> {
        Self::specs()
            .into_iter()
            .find(|spec| spec.key == key)
            .map(|spec| Self::detect_builtin(&spec))
    }

    pub fn custom_descriptor(profile: &EditorProfile) -> EditorDescriptor {
        let executable = Self::resolve_executable(&profile.executable);
        EditorDescriptor {
            key: profile.editor_key(),
            name: profile.name.clone(),
            family: "custom".to_string(),
            available: profile.enabled && executable.is_some(),
            version: None,
            executable: executable.as_ref().map(|p| p.to_string_lossy().to_string()),
            source: executable.map(|_| "custom".to_string()),
            supports_open_mode: false,
            supports_solution_target: true,
            is_custom: true,
        }
    }

    pub fn resolve_executable(value: &str) -> Option<PathBuf> {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
        which::which(value).ok()
    }

    fn find_executable(spec: &BuiltinEditorSpec) -> Option<(PathBuf, String)> {
        if spec.key == "builtin:visual-studio" {
            if let Some(path) = Self::find_visual_studio() {
                return Some((path, "vswhere".to_string()));
            }
        }

        for path in Self::known_paths(spec.key) {
            if path.is_file() {
                return Some((path, "known-path".to_string()));
            }
        }

        if spec.family == "jetbrains" {
            if let Some(exe_name) = spec.commands.first() {
                if let Some(path) = Self::find_jetbrains(exe_name) {
                    return Some((path, "jetbrains-toolbox".to_string()));
                }
            }
        }

        for command in spec.commands {
            if let Ok(path) = which::which(command) {
                if Self::is_native_executable(&path) {
                    return Some((path, "path".to_string()));
                }
            }
        }
        None
    }

    fn env_path(variable: &str, suffix: &str) -> Option<PathBuf> {
        std::env::var_os(variable).map(|base| PathBuf::from(base).join(suffix))
    }

    fn known_paths(key: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut push = |variable: &str, suffix: &str| {
            if let Some(path) = Self::env_path(variable, suffix) {
                paths.push(path);
            }
        };
        match key {
            "builtin:vscode" => {
                push("LOCALAPPDATA", r"Programs\Microsoft VS Code\Code.exe");
                push("ProgramFiles", r"Microsoft VS Code\Code.exe");
            }
            "builtin:vscode-insiders" => {
                push(
                    "LOCALAPPDATA",
                    r"Programs\Microsoft VS Code Insiders\Code - Insiders.exe",
                );
                push(
                    "ProgramFiles",
                    r"Microsoft VS Code Insiders\Code - Insiders.exe",
                );
            }
            "builtin:cursor" => {
                push("LOCALAPPDATA", r"Programs\cursor\Cursor.exe");
                push("LOCALAPPDATA", r"Programs\Cursor\Cursor.exe");
                push("ProgramFiles", r"Cursor\Cursor.exe");
            }
            "builtin:android-studio" => {
                push("ProgramFiles", r"Android\Android Studio\bin\studio64.exe");
                push("LOCALAPPDATA", r"Programs\Android Studio\bin\studio64.exe");
            }
            _ => {}
        }
        paths
    }

    fn find_visual_studio() -> Option<PathBuf> {
        let vswhere = Self::env_path(
            "ProgramFiles(x86)",
            r"Microsoft Visual Studio\Installer\vswhere.exe",
        )?;
        if !vswhere.is_file() {
            return None;
        }
        let output = Command::new(vswhere)
            .args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.Component.MSBuild",
                "-property",
                "installationPath",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if root.is_empty() {
            return None;
        }
        let path = PathBuf::from(root).join(r"Common7\IDE\devenv.exe");
        path.is_file().then_some(path)
    }

    fn find_jetbrains(exe_name: &str) -> Option<PathBuf> {
        let mut roots = Vec::new();
        if let Some(path) = Self::env_path("LOCALAPPDATA", r"JetBrains\Toolbox\apps") {
            roots.push(path);
        }
        if let Some(path) = Self::env_path("LOCALAPPDATA", r"Programs") {
            roots.push(path);
        }
        if let Some(path) = Self::env_path("ProgramFiles", "JetBrains") {
            roots.push(path);
        }

        let mut matches = Vec::new();
        for root in roots.into_iter().filter(|root| root.is_dir()) {
            for entry in WalkDir::new(root)
                .max_depth(5)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
            {
                if entry.file_type().is_file()
                    && entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(exe_name)
                {
                    matches.push(entry.into_path());
                }
            }
        }
        matches.sort_by(|a, b| b.to_string_lossy().cmp(&a.to_string_lossy()));
        matches.into_iter().next()
    }

    pub fn is_allowed_custom_executable(path: &Path) -> bool {
        Self::is_native_executable(path)
    }

    fn is_native_executable(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase),
            Some(ext) if ext == "exe" || ext == "com"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_keys_are_unique() {
        let specs = EditorDetectionService::specs();
        let mut keys: Vec<_> = specs.iter().map(|spec| spec.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), specs.len());
    }

    #[test]
    fn rejects_shell_script_extensions() {
        assert!(!EditorDetectionService::is_allowed_custom_executable(
            Path::new("tool.cmd")
        ));
        assert!(!EditorDetectionService::is_allowed_custom_executable(
            Path::new("tool.ps1")
        ));
        assert!(!EditorDetectionService::is_allowed_custom_executable(
            Path::new("tool.bat")
        ));
        assert!(EditorDetectionService::is_allowed_custom_executable(
            Path::new("tool.exe")
        ));
    }
}
