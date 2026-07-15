# 本地项目桌面管理系统 Claude 实施方案

> 本文档用于直接交给 Claude Code 执行。  
> 技术与需求基线：[`技术方案.md`](./技术方案.md)  
> 完成判定：[`验收测试方案.md`](./验收测试方案.md)

## 1. 执行目标

在当前仓库内完成一个可安装、可离线运行的 Windows 10/11 x64 桌面应用，完整实现：

- 项目登记、搜索、重命名、软移除、恢复和批量操作。
- 文件树、资源管理器入口、CMD/CLI 智能体启动。
- Markdown 和图片预览。
- `AGENTS.md`、`CLAUDE.md` 日志规则安全合并。
- `pm_log` 增量同步、活动热力图和时间线。
- 使用 Windows DPAPI 加密的服务器资料库。
- 单实例、托盘、全局快捷键、设置和 NSIS 安装包。

不要实现项目目录实时监听；不要引入 FastAPI、Spring Boot、Electron、MySQL 或 PostgreSQL。

## 2. 开始前必须完成

1. 完整阅读 `docs/技术方案.md`。
2. 完整阅读 `docs/验收测试方案.md`。
3. 检查仓库根目录和相关子目录中的现有 `CLAUDE.md`、`AGENTS.md` 及其他约束文件。
4. 检查当前 Git 状态，保留用户已有修改，不重置、不覆盖无关文件。
5. 检查 Windows 开发环境：Node.js 24 LTS、pnpm、Rust stable、Cargo、Microsoft C++ Build Tools、WebView2。
6. 记录实际工具版本到 README 或开发文档。
7. 如果仓库已有工程，优先在现有结构中实现，不重复初始化或整体重写。

## 3. 不可变更的技术决策

- Vue 3 + TypeScript + Vite。
- Element Plus、Pinia、Vue Router、Apache ECharts。
- Tauri 2 + Rust stable。
- SQLite + SQLx migrations。
- Windows DPAPI CurrentUser 加密资料正文。
- Vue 只通过类型化的 Tauri invoke 封装调用 Rust。
- SQLite 不向前端暴露任意 SQL 接口。
- 所有项目路径操作经过 Rust `PathGuard`。
- 所有进程调用使用可执行文件与参数数组，不拼接 Shell 字符串。
- 项目日志采用 `pm_log/YYYY-MM-DD/HHmmss-agent-shortid.md`。
- 项目活动由日志同步产生，不扫描或监听整个项目。
- 系统不提供项目文件移动和删除功能。

如实施中发现技术方案内部矛盾，先记录矛盾、给出最小修正建议；不要自行换技术栈或扩大范围。

## 4. 工程质量规则

- TypeScript 开启严格模式，不用无理由的 `any`。
- Rust 禁止在正常业务路径使用 `unwrap()`、`expect()` 和静默吞错。
- 统一使用结构化错误码，前端不展示 Rust 调试信息。
- 业务逻辑不得堆积在 Vue 页面或 Tauri command 函数中。
- 文件写入使用临时文件和原子替换；数据库多步写入使用事务。
- 密码正文不得出现在日志、SQLite、localStorage、错误文本、测试快照和示例数据中。
- 对新增核心逻辑同步增加测试，不把测试全部留到最后。
- 每完成一个阶段运行该阶段验证命令，失败即修复后再继续。
- 不修改与任务无关的用户文件，不执行破坏性 Git 命令。
- 不自行提交、推送或创建分支，除非用户另行要求。

## 5. 建议工程结构

```text
.
├── docs/
├── src/
│   ├── api/
│   ├── components/
│   ├── layouts/
│   ├── router/
│   ├── stores/
│   ├── types/
│   ├── utils/
│   └── views/
├── src-tauri/
│   ├── capabilities/
│   ├── migrations/
│   ├── src/
│   │   ├── commands/
│   │   ├── models/
│   │   ├── repositories/
│   │   ├── security/
│   │   ├── services/
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   └── state.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/
├── package.json
├── pnpm-lock.yaml
└── README.md
```

若已有结构不同，可以保留，但职责边界必须等价。

## 6. 分阶段实施计划

### 阶段 0：仓库与环境基线

任务：

- [ ] 检查现有代码、约束文件和工作区状态。
- [ ] 确认项目是否已初始化。
- [ ] 创建或补充 `.nvmrc`，内容指向 Node.js 24 主版本。
- [ ] 在 `package.json` 中固定 pnpm packageManager 版本。
- [ ] 确认 UTF-8、EditorConfig 和换行策略。
- [ ] 创建实施进度文件 `docs/实施进度.md`，按本文阶段记录状态、验证结果和未解决问题。

完成条件：

- 环境检查结果可复现。
- 用户原有文件和改动未被覆盖。
- 后续所有阶段有进度记录位置。

### 阶段 1：Tauri/Vue 基础工程

任务：

- [ ] 初始化或完善 Vue 3 + TypeScript + Vite 工程。
- [ ] 集成 Tauri 2。
- [ ] 安装并配置 Element Plus、Pinia、Vue Router、ECharts、markdown-it、DOMPurify。
- [ ] 配置 ESLint、Prettier 和 Vitest。
- [ ] 建立应用布局和路由：项目、项目回收站、资料库、资料回收站、设置。
- [ ] 建立前端 `api` 层，不允许页面直接散落 `invoke()`。
- [ ] 建立 Rust 模块骨架、共享状态和统一错误结构。
- [ ] 配置开发态日志，确保敏感字段默认脱敏。

验证：

```powershell
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
pnpm tauri dev
```

完成条件：

- 应用窗口正常启动。
- 六个页面路由均可访问且无控制台错误。
- Rust 示例命令通过统一 API 返回成功结果。
- 类型检查、Lint 和基础测试通过。

### 阶段 2：SQLite 与基础数据层

任务：

- [ ] 配置 SQLx SQLite，数据库位于应用数据目录。
- [ ] 创建第一版 migration：`projects`、`launch_profiles`、`project_logs`、`vault_entries`、`app_settings`。
- [ ] 启用外键和 WAL。
- [ ] 实现 repository 层和事务边界。
- [ ] 实现数据库初始化与迁移失败提示。
- [ ] 为软删除、恢复、判重和日志索引写 Rust 测试。
- [ ] 前端不得获得任意 SQL 执行命令。

验证：

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
pnpm typecheck
```

完成条件：

- 全新启动可以自动创建并迁移数据库。
- 重复启动迁移幂等。
- migration 失败不会静默创建不完整数据结构。
- repository 测试通过。

### 阶段 3：项目登记与智能体指令文件

任务：

- [ ] 使用 Tauri 目录选择对话框选择项目目录。
- [ ] 实现项目名称校验和路径规范化。
- [ ] 实现 `canonical_path` 判重和软移除项目恢复提示。
- [ ] 创建 `.project-manager.json`，使用数据库项目 UUID。
- [ ] 实现 `AGENTS.md` 管理块模板。
- [ ] 实现 `CLAUDE.md` 管理块模板。
- [ ] 实现管理块创建、追加、升级、幂等、损坏检测。
- [ ] 创建 `pm_log` 目录。
- [ ] 使用临时文件和原子替换更新指令文件。
- [ ] 任一步失败时回滚本次数据库新增，并清理由本次操作创建且可安全确认的临时文件。
- [ ] 添加项目前展示将创建/修改的文件清单。
- [ ] 实现项目列表、搜索、重命名、收藏、批量软移除和恢复。

必须测试：

- [ ] 空目录。
- [ ] 已有 AGENTS.md。
- [ ] 已有 CLAUDE.md。
- [ ] 已有同版本管理块。
- [ ] 已有旧版本管理块。
- [ ] 管理块只有开始标记或只有结束标记。
- [ ] 中文、空格和括号路径。
- [ ] 只读目录。
- [ ] 同一路径不同大小写和尾部斜杠。

完成条件：

- 添加项目行为满足技术方案中的合并算法。
- 重复执行不会重复追加规则。
- 原有指令内容逐字保留，管理块外无变化。
- 失败不会留下半完成数据库项目。

### 阶段 4：文件树与安全路径层

任务：

- [ ] 实现 `PathGuard`，所有项目文件命令必须调用。
- [ ] 实现按需读取直接子项，不递归预扫描。
- [ ] 实现目录优先、自然排序和隐藏文件设置。
- [ ] 处理无权限、已消失、路径过长和不安全链接。
- [ ] 实现文件树展开、收起、刷新和加载状态。
- [ ] 实现复制绝对路径。
- [ ] 实现文件复制到 Windows 剪贴板。
- [ ] 实现资源管理器打开目录和定位文件。
- [ ] 确保界面没有移动或删除项目文件入口。

必须测试：

- [ ] `..` 逃逸路径。
- [ ] 指向项目外部的链接/目录联接。
- [ ] 请求发出后文件被删除。
- [ ] 五千个直接子项目录。
- [ ] 中文和特殊字符文件名。

完成条件：

- 项目外路径无法通过伪造前端参数访问。
- 大目录加载有反馈，不锁死整个窗口。
- 复制和 Explorer 操作在 Windows 上实际可用。

### 阶段 5：Markdown 与图片预览

任务：

- [ ] 实现文件类型、大小和存在性校验。
- [ ] 实现 Markdown 文本读取和编码错误提示。
- [ ] 使用 markdown-it 渲染并用 DOMPurify 净化。
- [ ] 默认阻止远程资源自动请求。
- [ ] 实现项目内相对图片安全解析。
- [ ] 实现图片预览、缩放、适应、原始比例、旋转和重置。
- [ ] 安全处理 SVG。
- [ ] 对超大文件、损坏图片和未知二进制提供明确提示。

安全测试至少包含：

```html
<script>alert(1)</script>
<img src=x onerror=alert(1)>
<a href="javascript:alert(1)">click</a>
```

完成条件：

- Markdown 中脚本、事件处理器和危险协议不执行。
- 图片查看操作完整。
- 超限文件不会导致应用崩溃或长期无响应。

### 阶段 6：终端与启动配置

任务：

- [ ] 实现 Windows Terminal CMD 启动。
- [ ] 实现 `wt.exe` 不存在时的 `cmd.exe` 回退。
- [ ] 实现 PowerShell、Codex、Claude Code、VS Code、Cursor 默认配置。
- [ ] 实现全局启动配置和项目级覆盖。
- [ ] 参数模板只允许明确占位符，如 `{projectPath}`。
- [ ] 可执行文件与参数数组分别存储和调用。
- [ ] 记录“打开终端/智能体”这一应用操作，但不伪造项目工作日志。
- [ ] 从本应用启动的子进程结束时请求一次项目日志刷新；不能可靠获知外部终端标签页结束时，允许用户手动刷新。

必须测试：

- [ ] 已有 Windows Terminal 窗口时新增标签页。
- [ ] 没有窗口时创建窗口。
- [ ] `wt.exe` 不可用时回退。
- [ ] 路径含空格、中文、`&`、括号和 `#`。
- [ ] 恶意参数不能通过路径注入第二条命令。

完成条件：

- 项目目录作为真实工作目录打开。
- 无字符串拼接 Shell 注入路径。
- 每个启动配置错误均有可理解提示。

### 阶段 7：项目日志同步、热力图和时间线

任务：

- [ ] 实现只扫描 `pm_log` 的同步服务。
- [ ] 解析规范文件名、YAML Front Matter 和 Markdown 标题。
- [ ] 使用相对路径、修改时间、大小和哈希增量同步。
- [ ] 对缺少完成时间的日志使用文件修改时间并标记推断。
- [ ] 对格式错误日志建立异常索引，不中断其他日志。
- [ ] 更新项目 `last_activity_at`。
- [ ] 实现 ECharts 年度活动热力图。
- [ ] 实现日期、智能体、状态筛选。
- [ ] 实现日志时间线和净化后的详情预览。
- [ ] 在应用启动、进入项目详情和手动刷新时同步。
- [ ] 不添加项目目录实时监听。

必须测试：

- [ ] 同一天多个 Codex/Claude 日志。
- [ ] 两个智能体文件名时间相同但短 ID 不同。
- [ ] 无 Front Matter。
- [ ] 无效日期、未知状态、未知智能体。
- [ ] 日志被外部删除或修改。
- [ ] 10,000 个日志文件的增量刷新。

完成条件：

- 热力图数量与有效日志一致。
- 最近活动时间与最新日志一致。
- 异常日志可见、可定位，不影响正常日志。
- 未访问项目期间应用不持续扫描项目。

### 阶段 8：DPAPI 服务器资料库

任务：

- [ ] 使用 `windows` crate 封装 DPAPI CurrentUser 加密和解密。
- [ ] 封装敏感缓冲区清理，尽量缩短明文生命周期。
- [ ] 实现 TXT 导入，但不修改或删除源文件。
- [ ] 实现新建、遮罩预览、显隐、编辑和保存。
- [ ] 实现名称搜索和标签。
- [ ] 实现软移除、恢复和二次确认永久清除。
- [ ] 实现密文临时写入、成功后原子替换。
- [ ] 实现复制后 30 秒条件性清空剪贴板。
- [ ] 应用最小化、锁定或关闭资料详情时清除前端明文状态。
- [ ] 确保诊断日志、错误信息、SQLite 和前端持久化中无正文。
- [ ] 明确展示 DPAPI 备份/迁移限制。

必须测试：

- [ ] 加密后磁盘和 SQLite 搜索不到测试明文。
- [ ] 当前用户加解密回环。
- [ ] 密文被篡改后拒绝解密，原文件不覆盖。
- [ ] 编辑保存中断时旧密文仍可用。
- [ ] 软移除和恢复。
- [ ] 永久清除二次确认。
- [ ] 复制后用户已改写剪贴板时，不清除用户新内容。
- [ ] 错误日志中不存在测试秘密。

完成条件：

- 正文只在需要时临时进入内存。
- SQLite 和应用日志没有明文。
- 所有失败路径都不会把密文替换为空内容。

### 阶段 9：托盘、单实例、设置与体验完善

任务：

- [ ] 配置单实例，再次启动激活已有窗口。
- [ ] 实现托盘菜单和最小化到托盘。
- [ ] 实现全局快捷键，并处理快捷键占用错误。
- [ ] 实现窗口状态保存和屏幕外恢复。
- [ ] 实现可选开机启动，默认关闭。
- [ ] 完成设置页：隐藏文件、预览上限、剪贴板超时、全局快捷键、开机启动。
- [ ] 增加空状态、加载状态、错误状态和确认对话框。
- [ ] 检查键盘导航、焦点和常用操作可达性。

完成条件：

- 单实例、托盘、快捷键和窗口恢复在真实 Windows 环境验证通过。
- 设置重启后保留。
- 关闭/退出语义明确，不留下孤立后台进程。

### 阶段 10：测试、文档与安装包

任务：

- [ ] 补齐 Rust、前端和集成测试。
- [ ] 按 `docs/验收测试方案.md` 完整执行验收。
- [ ] 修复所有 P0/P1 缺陷。
- [ ] 确认没有项目实时监听进程或依赖。
- [ ] 确认无敏感测试数据进入仓库。
- [ ] 完成 README：安装、运行、开发、日志协议、资料库限制。
- [ ] 生成应用图标和版本信息。
- [ ] 配置并构建 NSIS 安装程序。
- [ ] 在干净 Windows 用户环境执行安装、首次启动、升级和卸载测试。
- [ ] 将实际验收结果填写到验收文档的执行记录中。

最终验证命令至少包括：

```powershell
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

完成条件：

- 所有命令成功。
- 生成可安装的 NSIS `.exe`。
- P0/P1 用例全部通过。
- 安装包在目标 Windows 版本实际运行。
- 用户数据在应用升级时保留。

## 7. 管理块模板要求

实现时将模板作为 Rust 内嵌资源或独立受版本控制模板，不在多处复制字符串。

### 7.1 AGENTS.md 管理块

```markdown
<!-- project-manager:log-rules:start v1 -->

## Project task logging

After completing or becoming blocked on a task that changes this project:

1. Create one new Markdown log under `pm_log/YYYY-MM-DD/`.
2. Name it `HHmmss-codex-<short-id>.md` using local time and a collision-resistant short ID.
3. Include YAML front matter with `pm_log_version`, `agent`, `started_at`, `finished_at`, and `status`.
4. Use `agent: codex` and one of `completed`, `failed`, or `blocked` for `status`.
5. Record the task goal, completed changes, changed files, verification results, and remaining issues.
6. Write the log after verification and before the final response.
7. Do not create a log for pure discussion or read-only analysis without project changes.
8. Never include passwords, tokens, cookies, private keys, environment variable values, or other secrets.
9. Never rewrite or delete existing project log files.

<!-- project-manager:log-rules:end -->
```

### 7.2 CLAUDE.md 管理块

内容与上述结构一致，但文件名使用 `HHmmss-claude-<short-id>.md`，Front Matter 使用 `agent: claude`。

## 8. 数据迁移与失败恢复要求

- 数据库 migration 必须随代码版本提交，不允许运行时临时拼接表结构。
- 新增项目时先完成路径检查，再准备文件变更，最后在事务中提交数据库；明确设计补偿动作。
- 指令文件更新前创建同目录临时文件，原子替换前保留原内容；任何失败不得截断原文件。
- 资料编辑先写新密文并验证可解密，再替换旧密文，最后更新元数据。
- 应用崩溃后应能识别和清理本应用遗留的临时文件，不删除无法确认归属的文件。
- 数据库损坏时提供错误和数据目录位置，不自动静默重建覆盖。

## 9. 完成报告格式

全部阶段完成后，向用户提供：

1. 完成的功能摘要。
2. 关键架构和安全实现说明。
3. 自动化测试命令与结果。
4. 验收测试通过/失败统计。
5. 安装包绝对路径。
6. 已知限制和未完成事项。
7. 发生过的数据迁移或兼容性决定。

如果仍有 P0/P1 用例未通过，不得宣称项目已完成；应明确列出失败用例、原因和复现步骤。
