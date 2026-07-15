# 第二期完整项目实施方案：产品体验整改与编辑器/IDE启动子系统

> 文档状态：可执行版  
> 适用项目：江天一览 RiverSkyView  
> 适用平台：Windows 10 / Windows 11  
> 技术基线：Tauri 2 + Rust + Vue 3 + TypeScript + SQLite  
> 编制日期：2026-07-15  
> 实施目标：在不破坏第一期既有能力的前提下，完成现有界面缺陷整改、产品命名与导航收敛，并为项目列表和项目详情增加完整、可靠、安全的编辑器/IDE启动能力。

---

## 1. 项目结论

该需求可以实现，而且现有工程已经具备约 70% 的底层基础：Rust 端已有通用进程启动能力，`ToolKind` 已包含 VS Code 和 Cursor，数据库也已有尚未充分利用的 `launch_profiles` 表。

第二期不是单独增加两个编辑器按钮，而是一次完整的可用性升级。本期由四条交付主线组成：现有 P0 界面缺陷整改、产品品牌与导航结构收敛、编辑器/IDE 启动子系统、全量回归和安装包交付。最终达到以下结果：

1. 在项目列表中直接选择 VS Code、Cursor、Visual Studio、JetBrains 系列或自定义 IDE 打开项目。
2. 在项目详情页提供相同入口，行为与项目列表保持一致。
3. 支持全局默认编辑器，以及单个项目自己的默认编辑器。
4. 对 Visual Studio、Rider 等支持解决方案文件的 IDE，自动发现 `.sln`、`.slnx`、`.csproj` 等目标。
5. 多个解决方案并存时让用户选择，并记住项目选择。
6. 自动检测本机已安装的 IDE；未安装项不允许误启动，并给出明确处理建议。
7. 支持用户添加自定义编辑器配置，但不允许拼接 Shell 命令。
8. 所有启动参数继续采用参数数组传递，保证含中文、空格、`&`、`#` 等字符的项目路径安全可用。
9. 修复活动热力图压缩错位、空状态大按钮图标畸变、服务器资料“按住显示”无效三个已确认问题。
10. 收藏从独立侧边栏合并到项目页筛选，减少与项目列表重复的入口。
11. 产品显示名统一更名为“江天一览”，英文名为“RiverSkyView”，副标题为“全域工程可视化工作台”；保留现有 bundle identifier 和数据目录，避免升级后丢失本地数据。

本期功能的准确名称建议统一为“编辑器与 IDE”，不要在界面上使用“编译器”。编译器负责编译代码，VS Code、Cursor、Visual Studio、Rider 等在本需求中属于编辑器或集成开发环境。

---

## 2. 本期范围

### 2.1 必须交付

- 活动热力图布局、尺寸计算和窗口缩放修复。
- 项目/收藏空状态按钮图标尺寸隔离修复。
- 服务器资料“按住显示”交互修复，并覆盖鼠标、触控笔、触摸取消、窗口失焦等状态。
- 产品显示名更名为“江天一览”，英文名为“RiverSkyView”，同步窗口标题、侧栏品牌、安装包显示名和文档称呼。
- 收藏入口并入项目页“全部 / 收藏 / 最近”筛选，移除独立侧边栏菜单但保留收藏数据和能力。
- 项目列表中的“编辑器”分裂按钮。
- 项目详情页中的“编辑器”分裂按钮。
- VS Code、Cursor、Visual Studio 2022 和主流 JetBrains IDE 的内置适配。
- 已安装 IDE 的自动检测和手动刷新。
- 全局默认编辑器设置。
- 项目级默认编辑器设置。
- Visual Studio/Rider 解决方案目标发现、选择和记忆。
- 自定义编辑器配置的新增、修改、禁用和删除。
- 安全启动、错误码、诊断信息和自动化测试。
- Windows 10/11 实机验收和新的 NSIS 安装包。

### 2.2 本期明确不做

- 不实现项目编译、构建、运行、调试任务编排。
- 不代替 IDE 管理插件、SDK、编译器或运行时。
- 不自动下载或安装第三方 IDE。
- 不把任意字符串交给 `cmd.exe`、PowerShell 或其他 Shell 执行。
- 不在项目目录中实时监听 IDE 安装或项目文件变化。
- 不实现跨设备同步编辑器配置。
- 不为每一种小众 IDE 编写专用适配器；未内置的工具通过自定义配置支持。
- 不在本期改变 bundle identifier、SQLite 路径或应用数据目录。
- 不删除收藏能力及收藏字段，只调整信息架构和入口。

### 2.3 第二期工作包与优先级

| 工作包 | 内容 | 优先级 | 阻塞发布 |
|---|---|---|---|
| WP-1 | 热力图、空状态按钮、资料按住显示缺陷整改 | P0 | 是 |
| WP-2 | “江天一览 RiverSkyView”显示品牌替换、收藏入口合并 | P0 | 是 |
| WP-3 | VS Code/Cursor/Visual Studio/Rider 启动闭环 | P0 | 是 |
| WP-4 | 自定义编辑器、其他 JetBrains IDE 与 Android Studio | P1 | 否，但不得影响 P0 |
| WP-5 | 自动化测试、Windows 实机验收、NSIS 交付 | P0 | 是 |

品牌命名、简介与 Slogan 以 `docs/江天一览 RiverSkyView 完整全套品牌资料.md` 为准。若未来公开商业发布，应另行进行商标和域名检索；本地自用不因此阻塞实施。

---

## 3. 现有工程基线与缺口

### 3.1 可复用能力

当前工程已有以下基础：

- `src-tauri/src/models/terminal.rs` 中的 `ToolKind` 已包含 `Vscode` 和 `Cursor`。
- 现有默认启动定义已能生成 `code {projectPath}` 与 `cursor {projectPath}` 参数。
- `terminal_service.rs` 已使用 `Command::new(...).args(...)` 启动进程，具备安全参数传递基础。
- 工具可用性检测已覆盖 VS Code 和 Cursor。
- `launch_profiles` 表已经包含可执行文件、参数数组、工作目录、启用状态和排序字段。
- 项目列表、项目详情、设置页和 Tauri 命令层已经具备可扩展的结构。

### 3.2 当前缺口

- 项目列表和详情页没有 VS Code/Cursor/IDE 的可见入口。
- 当前工具检测只有“能否找到”，没有安装实例、版本、路径和检测来源等结构化信息。
- 没有全局默认编辑器。
- 没有项目级默认编辑器及目标文件记忆。
- 没有 Visual Studio、Rider 等解决方案文件发现机制。
- `launch_profiles` 尚未形成完整的前后端配置管理闭环。
- 没有统一的编辑器错误码和面向用户的故障提示。

因此，本期采用“复用通用启动底座、建立独立编辑器领域层”的方式，避免继续把不同概念堆进终端启动模块。

---

## 4. 支持矩阵

| 编辑器/IDE | 内置键 | 默认目标 | 默认启动方式 | 本期级别 |
|---|---|---|---|---|
| Visual Studio Code | `builtin:vscode` | 项目目录 | `code <projectPath>` | P0 |
| VS Code Insiders | `builtin:vscode-insiders` | 项目目录 | `code-insiders <projectPath>` | P1 |
| Cursor | `builtin:cursor` | 项目目录 | `cursor <projectPath>` | P0 |
| Visual Studio 2022 | `builtin:visual-studio` | `.sln/.slnx/项目文件` | `devenv.exe <targetPath>` | P0 |
| IntelliJ IDEA | `builtin:idea` | 项目目录 | `idea64.exe <projectPath>` | P1 |
| WebStorm | `builtin:webstorm` | 项目目录 | `webstorm64.exe <projectPath>` | P1 |
| PyCharm | `builtin:pycharm` | 项目目录 | `pycharm64.exe <projectPath>` | P1 |
| Rider | `builtin:rider` | `.sln/.slnx/项目目录` | `rider64.exe <targetPath>` | P0 |
| CLion | `builtin:clion` | 项目目录 | `clion64.exe <projectPath>` | P1 |
| GoLand | `builtin:goland` | 项目目录 | `goland64.exe <projectPath>` | P1 |
| RustRover | `builtin:rustrover` | 项目目录 | `rustrover64.exe <projectPath>` | P1 |
| Android Studio | `builtin:android-studio` | 项目目录 | `studio64.exe <projectPath>` | P1 |
| 自定义编辑器 | `custom:<uuid>` | 项目目录或项目内目标 | 参数模板 | P0 |

P0 项必须通过全部自动化测试和实机验收；P1 项至少完成检测、启动和异常提示验收。

---

## 5. 用户体验设计

### 5.1 项目列表

项目行右侧操作调整为：

```text
[目录] [CMD] [编辑器 ▼] [更多]
```

“编辑器”使用分裂按钮：

- 点击主按钮：使用该项目默认编辑器打开。
- 如果项目未设置默认编辑器：使用全局默认编辑器。
- 如果全局也未设置：首次弹出编辑器选择菜单，选择后询问是否设为默认。
- 点击下拉箭头：展示本机可用编辑器、不可用编辑器状态和“管理编辑器”。
- 选择某个编辑器时，可勾选“以后此项目默认使用此编辑器”。

收藏不再单独占用侧边栏入口。收藏作为项目列表的筛选状态保留，统一在项目页面通过“全部 / 收藏 / 最近”切换。

### 5.2 项目详情页

顶部操作区调整为：

```text
[资源管理器] [CMD] [编辑器 ▼] [Codex ▼] [Claude ▼]
```

编辑器菜单与列表页共用同一个组件、同一套默认值和同一套错误处理，禁止出现两个页面行为不一致的问题。

### 5.3 设置页

设置页新增“编辑器与 IDE”区块：

- 全局默认编辑器。
- 打开窗口方式：由编辑器决定 / 新窗口 / 复用窗口。
- 已检测编辑器列表：名称、版本、路径、检测来源、可用状态。
- “重新检测”按钮。
- 自定义编辑器列表。
- 新增、编辑、禁用、删除自定义编辑器。
- 对自动检测失败的内置 IDE，可手动指定可执行文件。

### 5.4 多解决方案选择

Visual Studio 或 Rider 遇到多个候选目标时弹出选择框：

```text
选择要打开的解决方案

( ) MyApp.sln
( ) backend/Backend.sln
( ) 直接打开项目目录

[取消] [仅本次打开] [打开并记住]
```

已经记住的目标被删除或移出项目目录时，不自动改用不确定目标；应重新解析并要求用户确认。

### 5.5 状态与反馈

- 可用：正常显示，点击即可启动。
- 未安装：灰显，说明“未检测到”，并提供“重新检测”或“手动指定”。
- 路径失效：提示该编辑器位置已变化，清理失效缓存并引导重新检测。
- 启动成功：非打扰式提示“已使用 Rider 打开项目”。
- 启动失败：展示可理解的原因和可执行建议，不直接向普通用户显示 Rust 调试堆栈。

### 5.6 品牌与导航收敛

- 侧栏品牌主标题使用“江天一览”，英文副标使用“RiverSkyView”，关于页使用“全域工程可视化工作台”。
- `tauri.conf.json` 的可见产品名、窗口标题、NSIS 显示名同步更新。
- 为保留升级路径，不修改 bundle identifier、数据库名称、应用数据目录和加密作用域。
- 侧栏移除“收藏”独立菜单；项目页标题保持“项目”，页内提供“全部 / 收藏 / 最近”筛选。
- 旧的收藏路由保留一个版本的兼容重定向，跳转到项目页的收藏筛选，避免托盘、快捷方式或内部链接失效。
- 收藏增删、批量操作、空状态、项目数量统计继续复用同一项目数据源。

### 5.7 已确认界面缺陷整改

#### 热力图压缩错位

- 图表容器使用明确的最小高度和可计算宽度，不依赖内容自然撑高。
- ECharts 初始化必须发生在容器可见且尺寸有效之后。
- 使用 `ResizeObserver` 监听内容区尺寸变化并调用 `chart.resize()`；组件卸载时断开监听并 `dispose()`。
- `calendar.range`、星期标签、月份标签、单元格大小与 52 周数据范围保持一致。
- 空数据仍渲染完整 52 周坐标，不允许退化为左上角七行小方块。
- 切换“文件/活动”标签、窗口缩放、150% DPI 和侧栏变化后必须重新计算。

#### 空状态添加按钮图标畸变

- 禁止空状态按钮继承全局 `svg`、`.icon`、`button svg` 的不受限宽高。
- 为按钮图标使用组件级类，明确 `width`、`height`、`flex-shrink: 0` 和 `viewBox`。
- 文本与图标使用固定间距，不使用会被父容器拉伸的百分比尺寸。
- 项目页与收藏筛选共用同一个空状态组件，避免只修复其中一处。

#### 服务器资料“按住显示”无效

- 使用 Pointer Events 统一处理鼠标、触控笔和触摸输入。
- `pointerdown` 时显示，`pointerup`、`pointercancel`、`pointerleave`、窗口失焦和组件卸载时立即隐藏。
- 支持键盘：按住 Space/Enter 显示，松开隐藏。
- 禁止只监听移动端 `touchstart/touchend`，也禁止仅依赖 CSS `:active` 改变真实文本。
- 明文只存在于组件瞬时内存，不写入 Store 持久化、DOM 属性、日志或剪贴板。
- 连续快速按压、拖出按钮、切换窗口后不允许明文保持显示。

---

## 6. 数据模型设计

### 6.1 复用 `launch_profiles`

现有表用于保存自定义编辑器配置：

- `project_id IS NULL`：全局自定义编辑器。
- `project_id IS NOT NULL`：只对特定项目可见的自定义编辑器。
- `tool_kind = 'editor'`。
- `action_kind = 'open_project'`。
- `executable`：经过校验的绝对可执行文件路径。
- `args_json`：JSON 字符串数组，禁止保存整段 Shell 命令。
- `working_directory`：默认 `{projectPath}`。
- `enabled`：是否显示并允许启动。

不另建与 `launch_profiles` 重复的自定义配置表。

### 6.2 新增项目编辑器偏好表

新增迁移文件 `src-tauri/migrations/0002_editor_preferences.sql`：

```sql
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
```

字段约束：

- `editor_key` 使用稳定命名空间，如 `builtin:vscode`、`custom:<uuid>`。
- `target_relative_path` 必须是相对于项目根目录的路径，严禁保存项目外绝对目标。
- `open_mode` 仅允许 `default`、`new_window`、`reuse_window`。
- 读取到不存在的自定义配置或无效编辑器键时，视为陈旧偏好并回退到全局默认。

### 6.3 全局设置键

继续使用 `app_settings` 保存：

- `default_editor_key`
- `editor_open_mode`
- `editor_detection_last_run_at`

全局默认编辑器不存在或不可用时，不静默启动其他程序；应打开选择菜单并提示原默认项不可用。

---

## 7. Rust 后端架构

新增独立编辑器领域模块，建议目录如下：

```text
src-tauri/src/
├─ commands/
│  └─ editor.rs
├─ models/
│  └─ editor.rs
├─ repositories/
│  ├─ editor_preference_repository.rs
│  └─ editor_profile_repository.rs
└─ services/
   ├─ editor_detection_service.rs
   ├─ editor_target_service.rs
   └─ editor_launch_service.rs
```

### 7.1 核心模型

```rust
struct EditorDescriptor {
    key: String,
    name: String,
    family: EditorFamily,
    available: bool,
    version: Option<String>,
    executable: Option<String>,
    source: Option<DetectionSource>,
    supports_open_mode: bool,
    supports_solution_target: bool,
    is_custom: bool,
}

struct EditorTarget {
    relative_path: Option<String>,
    display_name: String,
    kind: EditorTargetKind,
    recommended: bool,
}

struct LaunchEditorRequest {
    project_id: String,
    editor_key: Option<String>,
    target_relative_path: Option<String>,
    open_mode: Option<OpenMode>,
    remember_for_project: bool,
}

struct LaunchEditorResult {
    editor_key: String,
    editor_name: String,
    target_display: String,
    used_project_default: bool,
}
```

### 7.2 适配器注册表

每个内置编辑器使用适配器描述差异：

```rust
trait EditorAdapter {
    fn key(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn detect(&self) -> Result<Vec<DetectedInstallation>, EditorError>;
    fn resolve_targets(&self, project_root: &Path) -> Result<Vec<EditorTarget>, EditorError>;
    fn build_launch_spec(
        &self,
        installation: &DetectedInstallation,
        project_root: &Path,
        target: Option<&Path>,
        open_mode: OpenMode,
    ) -> Result<LaunchSpec, EditorError>;
}
```

所有编辑器通过统一的注册表查找，禁止在 Tauri command 中使用大段 `match` 拼接命令。

### 7.3 检测顺序

编辑器检测按以下优先级执行：

1. 用户手动指定并仍然有效的可执行文件。
2. PATH 中可用命令。
3. Windows App Paths 或明确的已知安装位置。
4. Visual Studio 使用 `vswhere.exe` 获取安装实例。
5. JetBrains Toolbox 与常见独立安装目录。
6. 如果没有命中则标记未检测到，不进行全磁盘递归扫描。

检测要求：

- 只接受存在的普通文件。
- 版本命令设置 2 秒超时，超时不影响主流程。
- 多个安装实例按稳定版优先、版本新优先排序。
- 检测结果仅在内存中缓存；点击“重新检测”必须绕过缓存。
- 日志只记录产品名称、检测来源和必要路径，不记录环境变量全集。

Visual Studio 的 `vswhere.exe` 优先从官方固定位置查找：

```text
%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe
```

### 7.4 项目目标解析

普通文件夹型编辑器直接使用项目根目录。

Visual Studio/Rider 的目标发现规则：

1. 优先检查项目根目录下的 `.slnx`、`.sln`。
2. 再检查根目录下的 `.csproj`、`.fsproj`、`.vbproj`、`.vcxproj`。
3. 如果根目录没有，向下扫描不超过 2 层。
4. 忽略 `.git`、`node_modules`、`target`、`dist`、`build`、`.idea`、`.vs`。
5. 候选数上限 50，防止异常项目阻塞 UI。
6. 唯一解决方案自动推荐。
7. 多个候选项按“项目同名 > 根目录 > `.slnx`/`.sln` > 路径深度 > 字母顺序”排序。
8. 所有候选路径必须经 PathGuard 验证仍在项目根目录内。

### 7.5 安全启动规范

必须坚持以下规则：

- 使用 `std::process::Command::new(executable).args(arguments)`。
- 禁止用字符串拼出 `cmd /C ...`、`powershell -Command ...`。
- `current_dir` 固定为通过 PathGuard 校验的项目根目录。
- 内置适配器只生成经过代码定义的参数。
- 自定义参数只允许以下占位符：`{projectPath}`、`{projectName}`、`{targetPath}`。
- 未知占位符、NUL 字符、无效 JSON、超长参数必须拒绝。
- 自定义程序第一期只允许 `.exe` 和 `.com`；拒绝 `.bat`、`.cmd`、`.ps1`。
- 自定义可执行文件可以位于项目目录外，但必须由用户通过文件选择器明确选择。
- `targetPath` 必须位于项目目录内。
- 启动日志不得包含服务器资料、剪贴板内容或其他敏感明文。

建议限制：配置名不超过 80 字符，参数不超过 32 个，单个参数不超过 2048 字符，序列化配置不超过 32 KiB。

---

## 8. Tauri 命令与前端 API

### 8.1 后端命令

新增以下 Tauri commands：

```text
list_editors(project_id?)
refresh_editor_detection()
resolve_editor_targets(project_id, editor_key)
launch_project_editor(request)
get_project_editor_preference(project_id)
set_project_editor_preference(project_id, preference)
clear_project_editor_preference(project_id)
list_editor_profiles(project_id?)
create_editor_profile(input)
update_editor_profile(id, input)
delete_editor_profile(id)
```

`launch_project_editor` 内部流程：

```text
读取项目
  → PathGuard 校验项目根目录
  → 解析显式 editor_key / 项目默认 / 全局默认
  → 校验编辑器可用性
  → 解析并校验目标
  → 构造参数数组
  → 启动进程
  → 按请求保存项目偏好
  → 更新项目最近活动时间
  → 返回结构化结果
```

### 8.2 统一错误码

| 错误码 | 含义 | 前端处理 |
|---|---|---|
| `EDITOR_NOT_CONFIGURED` | 未设置默认编辑器 | 展开选择菜单 |
| `EDITOR_NOT_FOUND` | 编辑器键不存在 | 清理陈旧偏好并提示 |
| `EDITOR_EXECUTABLE_MISSING` | 可执行文件已移动或删除 | 引导重新检测/手动指定 |
| `EDITOR_SELECTION_REQUIRED` | 有多个目标需选择 | 打开目标选择框 |
| `EDITOR_TARGET_INVALID` | 目标不存在或逃逸项目目录 | 清除项目目标记忆并重选 |
| `EDITOR_PROFILE_INVALID` | 自定义配置不合法 | 定位到配置字段 |
| `EDITOR_DETECTION_TIMEOUT` | 版本检测超时 | 允许继续使用已确认路径 |
| `EDITOR_LAUNCH_FAILED` | 系统拒绝或进程启动失败 | 展示系统错误摘要 |
| `PROJECT_NOT_FOUND` | 项目已被删除 | 刷新项目列表 |
| `PROJECT_PATH_MISSING` | 项目目录不存在 | 使用现有缺失项目处理流程 |

错误响应中保留可诊断的 `details`，但 UI 默认只展示本地化后的 `message`。

---

## 9. 前端模块设计

新增：

```text
src/
├─ api/editor.ts
├─ stores/editor.ts
├─ types/editor.ts
└─ components/editor/
   ├─ EditorLaunchButton.vue
   ├─ EditorMenu.vue
   ├─ EditorTargetDialog.vue
   ├─ EditorProfileDialog.vue
   └─ EditorSettingsPanel.vue
```

### 9.1 Store 职责

`editor.ts` Store 负责：

- 编辑器检测结果缓存。
- 全局默认编辑器。
- 项目偏好按项目 ID 缓存。
- 刷新、启动、选择目标、配置管理的加载状态。
- 将统一错误码映射为用户提示。

Store 不保存可执行文件的命令行拼接结果，也不把自定义配置复制到 localStorage；以 Rust/SQLite 为唯一可信来源。

### 9.2 组件行为

`EditorLaunchButton.vue` 必须支持：

- `projectId` 输入。
- 默认启动与下拉选择。
- 防重复点击。
- 启动中状态。
- 编辑器不可用状态。
- 目标选择回调。
- “记住此项目”操作。

项目列表与项目详情必须复用该组件，禁止复制两套事件逻辑。

### 9.3 可访问性与布局

- 主按钮和下拉按钮分别具有明确 `aria-label`。
- 键盘可通过 Tab、Enter、方向键和 Escape 操作菜单。
- 长编辑器路径使用省略号和 Tooltip，不撑破设置页。
- 1280×720、150% Windows 缩放下按钮不得变形或遮挡。
- 空状态大图标必须采用固定尺寸，避免受到全局 `.icon` 或 `svg` 样式污染。

---

## 10. 兼容与迁移策略

- 新迁移只新增表和索引，不修改第一期业务表字段。
- 既有 Codex/Claude 启动配置和项目默认动作保持不变。
- 现有 `launch_terminal` 中 VS Code/Cursor 能力暂时保留，避免旧代码和测试立即失效。
- 新 UI 统一调用 `launch_project_editor`，不再通过终端模块启动编辑器。
- 稳定一个版本后，可在第三期内部重构时将旧 VS Code/Cursor 分支标记弃用。
- 数据库迁移必须包含事务，并验证从 1.0.0 数据库原地升级。
- 回滚旧安装包时，新表不会影响旧版读取；用户偏好仍留在数据库中。

---

## 11. 分阶段实施计划

### 阶段 0：基线确认与保护

任务：

- 记录当前 `git status`、版本号、数据库版本和安装包路径。
- 执行现有 TypeScript、Rust 和前端测试。
- 对当前数据库做一份测试副本，用于升级回归。
- 将第二期进度写入 `docs/第二期实施进度.md`。

退出条件：第一期基线测试结果已记录；如果存在原有失败，先区分基线问题和本期问题。

### 阶段 1：P0 体验整改、品牌收敛与数据领域模型

任务：

- 按第 5.7 节修复热力图、空状态按钮和资料按住显示。
- 增加针对三项缺陷的前端回归测试，不允许只做 CSS 目测修改。
- 将可见产品名统一为“江天一览 RiverSkyView”，但保持 bundle identifier 和数据目录不变。
- 将收藏合并到项目页筛选，并为旧收藏路由增加兼容重定向。
- 增加 `0002_editor_preferences.sql`。
- 增加编辑器 DTO、枚举、序列化和错误模型。
- 实现偏好与 `launch_profiles` Repository。
- 增加迁移、CRUD、级联删除和陈旧配置测试。

退出条件：三项已知界面问题均有自动化回归测试；品牌和收藏导航完成；数据库可从空库和 1.0.0 数据库升级；项目删除时偏好自动删除。

### 阶段 2：内置适配器与安装检测

任务：

- 建立适配器注册表。
- 实现 VS Code、Cursor、Visual Studio、Rider P0 适配器。
- 实现其余 JetBrains IDE 和 Android Studio P1 适配器。
- 实现 PATH、已知路径、`vswhere`、Toolbox/独立安装检测。
- 增加检测缓存、刷新和超时机制。

退出条件：在未安装、单实例、多实例和失效路径场景下均返回稳定结构，不发生崩溃。

### 阶段 3：项目目标发现

任务：

- 实现解决方案和项目文件的有界扫描。
- 增加忽略目录、排序、上限和 PathGuard 校验。
- 实现项目目标记忆与目标失效处理。

退出条件：唯一目标可自动打开；多个目标返回 `EDITOR_SELECTION_REQUIRED`；路径逃逸被拒绝。

### 阶段 4：安全启动与自定义配置

任务：

- 实现 `EditorLaunchService`。
- 将内置和自定义配置统一转换为 `LaunchSpec`。
- 实现允许的占位符替换。
- 拒绝 Shell 脚本、未知占位符、无效参数和非法目标。
- 增加中文、空格、`&`、`#`、括号路径测试。

退出条件：所有启动均使用参数数组；安全测试能够证明项目路径不会变成额外命令。

### 阶段 5：Tauri 命令与 API

任务：

- 注册本方案定义的全部 commands。
- 实现前端 `api/editor.ts` 和类型。
- 建立统一错误序列化。
- 对命令边界增加集成测试。

退出条件：前端可完成检测、启动、目标解析、偏好和自定义配置的完整调用。

### 阶段 6：设置页

任务：

- 新增“编辑器与 IDE”设置区块。
- 实现全局默认项、打开方式、检测结果和刷新。
- 实现自定义编辑器 CRUD。
- 实现手动选择可执行文件和即时校验。

退出条件：设置重启后保持；删除正在使用的自定义默认项时有明确二次确认和回退处理。

### 阶段 7：项目列表与详情集成

任务：

- 在项目列表加入分裂按钮。
- 在项目详情加入相同组件。
- 实现目标选择框和项目偏好记忆。
- 将收藏从独立侧边菜单合并到项目页筛选。
- 修复因新增按钮造成的窄窗口布局问题。

退出条件：两个入口行为完全一致；两次点击以内可以用指定编辑器打开项目。

### 阶段 8：自动化测试与质量整改

任务：

- 完成 Rust 单元/集成测试。
- 完成 Vue 组件、Store 和 API 测试。
- 修复全部 TypeScript 错误和 Clippy 警告。
- 执行生产构建。

退出条件：本方案第 13 节的质量命令全部成功，`clippy -D warnings` 为零警告。

### 阶段 9：Windows 实机验收

任务：

- 在 Windows 10 与 Windows 11 中各执行 P0 用例。
- 使用真实 VS Code、Cursor、Visual Studio、Rider 验证启动。
- 验证中文、空格和特殊字符项目路径。
- 验证 100%、125%、150% 缩放和窄窗口布局。
- 验证从 1.0.0 数据库升级。

退出条件：全部 P0 通过；P1 失败必须有明确限制说明，不得影响 P0 发布。

### 阶段 10：打包与交付

任务：

- 更新应用版本号，建议 `1.1.0`。
- 生成新的 NSIS x64 安装包。
- 在全新用户环境执行安装、首次启动、升级、卸载验证。
- 生成 `docs/第二期整改与复验报告.md`。
- 记录安装包路径、大小、SHA-256 和测试汇总。

退出条件：安装包可安装、启动、升级且数据不丢失；交付报告包含真实测试证据。

---

## 12. 测试方案

### 12.1 Rust 单元测试

至少覆盖：

- 每个内置适配器的参数构造。
- VS Code `default/new_window/reuse_window` 参数差异。
- Visual Studio 与 Rider 目标优先级。
- 多解决方案返回选择要求。
- 忽略目录和扫描深度限制。
- 符号链接和 `..` 路径逃逸拒绝。
- 自定义占位符的正常替换和非法占位符拒绝。
- `.bat/.cmd/.ps1` 拒绝。
- 含中文、空格、`&`、`#`、单引号和括号的路径保持为单个参数。
- 编辑器路径失效、检测超时和启动失败错误映射。
- 项目偏好 CRUD、删除项目级联和陈旧自定义键回退。

### 12.2 前端测试

至少覆盖：

- 热力图空数据仍保留完整 52 周布局。
- 标签切换和容器变化触发图表 resize，卸载时正确释放实例。
- 空状态图标的计算尺寸固定，不受全局 SVG 样式影响。
- 服务器资料在 pointerdown/keydown 时显示，在所有结束、取消和失焦事件后隐藏。
- 资料明文不进入持久化 Store、DOM 属性和日志。
- 旧收藏路由正确跳转到项目页收藏筛选，收藏数据保持不变。
- 有默认编辑器时主按钮直接启动。
- 无默认编辑器时展开选择。
- 项目默认覆盖全局默认。
- 不可用编辑器灰显且不可启动。
- 防止连续点击启动多个进程。
- `EDITOR_SELECTION_REQUIRED` 打开目标选择框。
- “仅本次”和“打开并记住”行为不同。
- 自定义配置表单校验。
- 删除默认配置的确认和状态更新。
- 列表页与详情页共享同一组件。
- 1280×720 和窄宽度下按钮布局不畸变。

### 12.3 Windows 手工验收核心用例

| 编号 | 用例 | 预期 |
|---|---|---|
| E-001 | 项目列表用 VS Code 打开 | VS Code 工作区为所选项目目录 |
| E-002 | 项目详情用 Cursor 打开 | Cursor 工作区为所选项目目录 |
| E-003 | 设 Rider 为项目默认 | 重启应用后主按钮仍使用 Rider |
| E-004 | 全局默认 VS Code、项目默认 VS | 对应项目优先用 Visual Studio |
| E-005 | 单个 `.sln` | 直接打开该解决方案 |
| E-006 | 多个 `.sln` | 弹出选择，不随意猜测 |
| E-007 | 已记忆 `.sln` 被删除 | 要求重选，不启动错误目标 |
| E-008 | IDE 卸载后启动 | 明确提示并提供重新检测 |
| E-009 | 中文和空格路径 | 正常启动且路径未被拆分 |
| E-010 | 路径含 `& # ()` | 正常启动且无命令注入 |
| E-011 | 自定义 `.exe` | 按参数数组正常打开项目 |
| E-012 | 自定义 `.cmd` | 保存或启动时被拒绝 |
| E-013 | 150% 缩放 | 按钮、菜单和弹窗不变形 |
| E-014 | 从 1.0.0 升级 | 原项目、日志和资料数据保留 |
| E-015 | 删除自定义默认编辑器 | 清理偏好并要求重新选择 |

---

## 13. 质量门槛

交付前必须依次执行并记录真实结果：

```powershell
pnpm typecheck
pnpm lint
pnpm test
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

发布门槛：

- TypeScript 0 错误。
- Rust 测试全部通过。
- Clippy `-D warnings` 0 警告。
- 前端测试全部通过。
- P0 手工验收全部通过。
- 生产安装包成功生成。
- 不允许用“代码已写完”代替实际运行测试。

---

## 14. 验收标准

满足以下条件才可以宣布第二期完成：

1. 用户可从项目列表和详情页打开任意项目。
2. 已安装编辑器能被正确检测，未安装编辑器不会误报可用。
3. 全局默认与项目默认优先级正确且重启保持。
4. Visual Studio/Rider 在多解决方案项目中不会随意选择。
5. 自定义编辑器配置可用且不存在 Shell 注入路径。
6. 特殊字符项目路径作为单个参数传递。
7. 旧版数据库可无损升级。
8. Codex、Claude、CMD、日志、资料库等第一期功能无回归。
9. 设置页、项目列表和详情页在 150% 缩放下无按钮变形。
10. 自动化质量门槛和 Windows P0 验收全部通过。
11. 热力图、空状态按钮、资料按住显示三个已确认问题均已修复并具备回归测试。
12. 应用中文显示名为“江天一览”、英文名为“RiverSkyView”，旧版升级后项目、日志、资料库和设置均不丢失。
13. 收藏只占用项目页筛选入口，旧收藏链接仍可兼容跳转。

---

## 15. 风险与控制

### 风险一：IDE 安装路径差异大

控制：采用“手动配置 > PATH > 官方检测工具/已知位置”的多级策略，并提供手动选择可执行文件作为最终兜底。

### 风险二：Visual Studio 项目目标存在歧义

控制：只在唯一且明确时自动选择；多个候选由用户决定并记忆，不基于不可解释的猜测启动。

### 风险三：自定义参数引入命令注入

控制：只允许可执行文件和参数数组；不经过 Shell；限制占位符、参数数量、长度和程序后缀。

### 风险四：工具检测拖慢启动

控制：首次进入相关界面时异步检测，设置超时，结果内存缓存，用户主动刷新才强制重扫。

### 风险五：新按钮加剧现有界面布局问题

控制：使用复用分裂按钮组件，明确最小宽度和响应式隐藏顺序，并纳入 1280×720/150% 验收。

### 风险六：旧数据中的默认项失效

控制：读取时验证编辑器键和可执行文件；失效后保留诊断信息，但不静默启动其他程序。

---

## 16. 交付物清单

- Rust 编辑器领域模型、检测、目标解析和启动服务。
- 热力图、空状态按钮和资料按住显示整改代码及测试。
- “江天一览 RiverSkyView”可见品牌配置与收藏导航合并。
- 数据库迁移 `0002_editor_preferences.sql`。
- Tauri 编辑器 commands。
- 前端 API、Store、共享组件、目标选择框和设置面板。
- 项目列表与项目详情集成。
- 收藏入口合并到项目筛选。
- Rust 与前端自动化测试。
- Windows 10/11 P0 验收记录。
- `docs/第二期实施进度.md`。
- `docs/第二期整改与复验报告.md`。
- 版本号为 `1.1.0` 的 NSIS 安装包。
- 安装包 SHA-256、大小、构建时间与产物路径。

本地自用场景不要求代码签名。Windows 可能显示未知发布者提示，这不作为本期阻塞项。

---

## 17. 供 Claude Code 直接执行的总指令

```text
你现在负责实施《docs/第二期-编辑器IDE启动能力完整实施方案.md》全部内容。

执行要求：
1. 从阶段 0 连续执行到阶段 10，不要只生成示例代码或停在设计阶段。
2. 开始前阅读现有技术方案、验收方案、整改报告、数据库迁移和相关源码。
3. 保留用户现有改动，不使用 git reset --hard、git clean 或其他破坏性命令。
4. 每完成一个阶段，更新 docs/第二期实施进度.md，记录完成项、测试证据、问题和下一步。
5. 遇到普通技术选择，依据实施方案作最小保守决定并继续，不要频繁中断询问。
6. 遇到真正阻塞，记录阻塞编号、阶段、原因、已尝试方案、复现命令和解除条件。
7. 所有进程启动必须使用可执行文件加参数数组，不得通过 Shell 拼接命令。
8. 不得降低测试标准、删除失败测试或用跳过测试代替修复。
9. 阶段 8 必须运行全部质量命令，并将真实结果写入进度文档。
10. 阶段 10 必须生成 NSIS 安装包和 docs/第二期整改与复验报告.md。
11. 最终报告必须给出变更摘要、测试数量、Windows 验收结果、剩余限制、安装包绝对路径、文件大小和 SHA-256。

只有满足第 14 节全部验收标准，才可以声明第二期完成。
```

---

## 18. 官方实现依据

- VS Code 官方命令行说明：`code <folder>`，并支持 `--new-window`、`--reuse-window` 等选项。  
  https://code.visualstudio.com/docs/configure/command-line
- VS Code Windows 安装与 PATH 说明：  
  https://code.visualstudio.com/docs/setup/windows
- Visual Studio 官方 `devenv` 命令行说明：可将解决方案或项目路径作为启动目标。  
  https://learn.microsoft.com/en-us/visualstudio/ide/reference/devenv-command-line-switches?view=visualstudio
- Visual Studio 实例发现与 `vswhere`：  
  https://learn.microsoft.com/en-us/visualstudio/install/tools-for-managing-visual-studio-instances?view=visualstudio
- JetBrains IntelliJ IDEA 命令行打开项目：  
  https://www.jetbrains.com/help/idea/opening-files-from-command-line.html
- JetBrains Rider 命令行打开项目和解决方案：  
  https://www.jetbrains.com/help/rider/Opening_Files_from_Command_Line.html
- Android Studio Windows 安装与可执行程序说明：  
  https://developer.android.com/studio/install.html

---

## 19. 最终建议

第二期应以“常用操作足够快、复杂情况不误判、安全边界不后退”为原则。最重要的产品结果不是支持多少 IDE 图标，而是用户在项目列表中选择一个项目后，可以稳定地用正确工具和正确目标打开，并且第二次操作更快。

因此发布优先级应保持：

```text
安全启动与默认项
  > VS Code / Cursor / Visual Studio / Rider 完整体验
  > 自定义编辑器
  > 其余 IDE 自动检测覆盖率
```

如果工期出现压力，不得删减安全参数传递、项目目标校验、默认项持久化和 P0 测试；可以将少数 P1 IDE 的自动检测优化延后到 1.1.x。
