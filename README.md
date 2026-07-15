# 江天一览 RiverSkyView

> 版本：1.1.0
> 平台：Windows 10/11 x64
> 技术：Tauri 2 + Vue 3 + TypeScript + Rust + SQLite + DPAPI

## 简介

江天一览（RiverSkyView）是一款以本地项目为中心的 Windows 全域工程可视化工作台，解决项目分散在不同目录、频繁进入目录、重复打开终端和启动 CLI 智能体的问题。

## 核心功能

- **项目管理**：登记、搜索、重命名、收藏、软移除和恢复本地项目
- **文件树**：按需加载的项目文件树，支持文件复制、路径复制和资源管理器入口
- **文件预览**：只读 Markdown 和图片预览，使用 DOMPurify 净化，阻止远程资源和脚本
- **终端启动**：CMD、PowerShell、Codex、Claude Code、VS Code、Cursor 一键启动
- **CLI 会话恢复**：Codex/Claude Code 支持新建会话、恢复选择器、继续最近会话三种模式
- **项目日志**：自动同步 `pm_log` 目录下的任务日志，GitHub 风格热力图和时间线
- **指令文件**：自动创建和合并 `AGENTS.md` 和 `CLAUDE.md` 管理块，不覆盖已有内容
- **服务器资料库**：使用 Windows DPAPI CurrentUser 加密的服务器资料，默认遮罩，条件性剪贴板清理
- **桌面体验**：单实例运行、系统托盘、全局快捷键（Ctrl+Alt+P）、开机启动

## 快速开始

### 环境要求

- Node.js 20+
- pnpm 10+
- Rust 1.77+ (stable)
- Microsoft C++ Build Tools
- WebView2 Runtime
- Windows Terminal（推荐）

### 开发运行

```bash
pnpm install
pnpm tauri dev
```

### 构建安装包

```bash
pnpm tauri build
```

生成的 NSIS 安装包位于 `src-tauri/target/release/bundle/nsis/`。

### 验证命令

```bash
# 前端
pnpm typecheck
pnpm lint
pnpm test

# Rust
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## 日志协议

任务日志存放在项目的 `pm_log/YYYY-MM-DD/HHmmss-<agent>-<short-id>.md`，包含 YAML Front Matter：

```yaml
---
pm_log_version: 1
agent: codex        # codex | claude | other
started_at: 2026-07-15T14:20:00+08:00
finished_at: 2026-07-15T14:32:05+08:00
status: completed   # completed | failed | blocked
---
```

日志规则由 `AGENTS.md` 和 `CLAUDE.md` 中的管理块自动注入。

## 安全设计

- 所有文件路径通过 Rust `PathGuard` 验证，禁止逃逸项目根目录
- Markdown 和 SVG 视为不可信输入，使用 DOMPurify 净化
- 服务器资料使用 Windows DPAPI CurrentUser 加密，密文不在 SQLite 或日志中
- 所有进程调用使用参数数组，不拼接 Shell 字符串
- 资料明文只临时存在于内存，不持久化到前端状态

## 数据目录

```
%LOCALAPPDATA%\ProjectManager\
├── project-manager.db      # SQLite 数据库
├── encrypted-vault\        # 加密资料密文
├── thumbnails\             # 缩略图缓存
├── logs\                   # 应用日志
└── backups\                # 备份
```

## 限制说明

- 第一阶段仅支持 Windows
- DPAPI 加密资料绑定当前 Windows 用户和设备，不支持跨设备迁移
- 不提供项目文件的移动和删除功能（安全边界）
- 不实时监听项目目录
- SVG 和远程图片默认不加载

