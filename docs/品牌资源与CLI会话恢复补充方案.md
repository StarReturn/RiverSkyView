# 品牌资源与 CLI 会话恢复补充方案

> 本文档补充《技术方案》《CLAUDE实施方案》《验收测试方案》和《前端界面设计方案》。如有冲突，以本文关于图标和 CLI 会话恢复的规则为准。

## 1. 品牌资源

原始资源已归档到：

```text
assets/branding/logo.png
assets/branding/logo.ico
```

检查结果：

- `logo.png`：1254×1254，24-bit RGB，无透明通道。
- `logo.ico`：包含 16、24、32、48、64、72、80、96、128、256 共 10 个 32-bit 图层。
- 两个文件当前均为不透明背景；源图中的棋盘格已经写入像素，并非真正透明。

## 2. 图标应用规则

### Windows 程序和安装包

将 `assets/branding/logo.ico` 复制为：

```text
src-tauri/icons/icon.ico
```

并在 `src-tauri/tauri.conf.json` 的 `bundle.icon` 中包含：

```json
{
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico"
    ]
  }
}
```

ICO 用于：

- 应用 EXE 图标。
- NSIS 安装包图标。
- Windows 开始菜单和桌面快捷方式。
- 任务栏和窗口图标。

### 应用内品牌图

`logo.png` 用于：

- 左侧导航展开状态顶部，建议显示 28×28 或 32×32。
- 关于页面，建议显示 96×96。
- 首次启动或空项目状态，建议显示 80–120 px。
- 安装程序欢迎页的品牌图。

当前 PNG 无透明通道且带有已烘焙棋盘格，因此只能临时用于白色背景。正式发布前应准备至少 1024×1024 的透明 RGBA PNG 或透明 SVG，再运行：

```powershell
pnpm tauri icon assets/branding/logo-transparent.png
```

该命令生成 `src-tauri/icons` 所需的 Windows 和其他平台尺寸。不要直接用现有不透明 PNG 重新生成最终图标，否则棋盘格会进入所有尺寸。

## 3. Codex 与 Claude Code 启动模式

每个项目为 Codex 和 Claude Code 分别提供一个分裂按钮：

```text
[Codex ▾]  [Claude ▾]
```

按钮主体执行该项目上次选择的模式；下拉菜单固定提供：

- 新建会话。
- 恢复会话……
- 继续最近会话。

命令映射：

| 智能体 | 新建会话 | 恢复会话选择器 | 继续当前项目最近会话 |
|---|---|---|---|
| Codex | `codex` | `codex resume` | `codex resume --last` |
| Claude Code | `claude` | `claude -r` | `claude -c` |

直接恢复指定会话：

```text
codex resume <SESSION_ID或会话名称>
claude -r <SESSION_ID或会话名称>
```

第一版不解析 Codex 或 Claude Code 的私有会话文件。用户点击“恢复会话……”时，应用在目标项目目录启动官方交互式选择器。

## 4. Windows Terminal 参数

所有命令使用参数数组传递，禁止拼接 Shell 字符串。

Codex 恢复选择器：

```text
executable: wt.exe
args:
  - -w
  - "0"
  - new-tab
  - -d
  - <project-path>
  - --title
  - Codex · <project-name>
  - codex
  - resume
```

Codex 继续最近会话：

```text
["-w", "0", "new-tab", "-d", <project-path>, "codex", "resume", "--last"]
```

Claude Code 恢复选择器：

```text
["-w", "0", "new-tab", "-d", <project-path>, "claude", "-r"]
```

Claude Code 继续最近会话：

```text
["-w", "0", "new-tab", "-d", <project-path>, "claude", "-c"]
```

如果 `wt.exe` 不存在，使用项目目录作为 `current_dir` 直接启动相应 CLI 或通过 `cmd.exe /K` 回退；仍然不得把项目路径拼进单一命令字符串。

## 5. 界面行为

项目列表操作区：

```text
[目录] [CMD] [Codex ▾] [Claude ▾] [更多]
```

项目详情头部：

```text
[资源管理器] [CMD] [Codex ▾] [Claude ▾] [更多]
```

下拉菜单文案必须区分：

- “新建会话”：不加载旧对话。
- “恢复会话……”：打开官方会话选择器。
- “继续最近会话”：不显示选择器，直接继续当前项目最近会话。

首次使用时默认主体动作设为“恢复会话……”，之后按项目、按智能体分别记忆最后选择。设置页允许把全局默认改成任一模式。

如果找不到 `codex.exe` 或 `claude.exe`：

- 按钮显示不可用状态。
- Tooltip 说明未检测到 CLI。
- 下拉菜单提供“打开启动设置”。
- 不弹出空白终端窗口。

## 6. 数据设计补充

`launch_profiles` 增加或明确以下逻辑字段：

```text
tool_kind: cmd | powershell | codex | claude | vscode | cursor | custom
action_kind: new | resume_picker | continue_latest | custom
```

每个项目设置中保存：

```text
default_codex_action
default_claude_action
```

只保存启动模式，不保存或复制 CLI 会话正文。会话 ID 只有用户明确创建“指定会话快捷方式”时才允许保存，且应作为普通配置而非敏感资料处理。

## 7. 实施任务补充

- [ ] 将归档 ICO 接入 Tauri Windows bundle。
- [ ] 为侧栏、空状态和关于页接入品牌图。
- [ ] 正式发布前取得透明 RGBA/SVG 源图并重新生成全套图标。
- [ ] 检测 `codex --version`、`codex resume --help`、`claude --version` 和 `claude --help`。
- [ ] 实现 Codex 新建、恢复选择、继续最近三种模式。
- [ ] 实现 Claude Code 新建、恢复选择、继续最近三种模式。
- [ ] 实现项目级默认动作记忆。
- [ ] 不读取两个 CLI 的私有会话历史目录。
- [ ] 所有路径和参数通过进程参数数组传递。

## 8. 验收补充

| ID | 优先级 | 测试步骤 | 预期结果 |
|---|---|---|---|
| BRD-001 | P1 | 安装应用并检查 EXE、快捷方式、任务栏 | 均显示提供的应用图标 |
| BRD-002 | P1 | 检查 ICO 图层 | 至少包含 16、24、32、48、64、256 |
| BRD-003 | P2 | 在 100%、150%、200% DPI 下检查图标 | 图标清晰，不显示错误裁切 |
| BRD-004 | P1 | 检查侧栏、空状态、关于页 | 品牌图尺寸和位置符合设计，无拉伸 |
| BRD-005 | P1 | 使用暗色主题检查现有 PNG | 若仍有棋盘格/白底不协调，正式发布验收不得通过 |
| SES-001 | P1 | 点击 Codex 新建会话 | 在目标项目目录启动新会话 |
| SES-002 | P1 | 点击 Codex 恢复会话 | 打开 `codex resume` 官方选择器，默认筛选当前目录 |
| SES-003 | P1 | 点击 Codex 继续最近 | 执行 `codex resume --last`，不显示选择器 |
| SES-004 | P1 | 点击 Claude 新建会话 | 在目标项目目录启动新会话 |
| SES-005 | P1 | 点击 Claude 恢复会话 | 执行 `claude -r` 并打开官方选择器 |
| SES-006 | P1 | 点击 Claude 继续最近 | 执行 `claude -c`，不显示选择器 |
| SES-007 | P1 | 不同项目分别选择默认动作并重启 | 每个项目、每个智能体的默认动作正确保留 |
| SES-008 | P0 | 从含空格、中文、`&`、括号的路径恢复 | 在正确目录恢复，不发生命令注入 |
| SES-009 | P1 | CLI 未安装时点击对应按钮 | 不启动空白窗口，显示安装/路径提示 |

## 9. 当前环境核对记录

2026-07-15 本机核对：

```text
codex-cli 0.144.1
Codex: codex resume / codex resume --last

Claude Code 2.0.76
Claude: claude -r / claude -c
```

实现时不得只依赖此版本号；应以目标机器实际 `--help` 能力和配置的可执行路径为准。

## 10. 官方参考

- [Tauri App Icons](https://v2.tauri.app/develop/icons/)
- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage)
- [Claude Code session management](https://code.claude.com/docs/en/sessions)
