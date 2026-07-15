// 项目类型
export interface Project {
  id: string;
  name: string;
  path: string;
  canonical_path: string;
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
  last_opened_at: string | null;
  last_activity_at: string | null;
  removed_at: string | null;
}

export interface ProjectListItem {
  id: string;
  name: string;
  path: string;
  is_favorite: boolean;
  created_at: string;
  last_opened_at: string | null;
  last_activity_at: string | null;
  removed_at: string | null;
  directory_available: boolean;
}

export interface AddProjectRequest {
  path: string;
  name?: string;
  is_favorite?: boolean;
}

export interface AddProjectResult {
  project: Project;
  agents_md_created: boolean;
  agents_md_updated: boolean;
  claude_md_created: boolean;
  claude_md_updated: boolean;
  pm_log_created: boolean;
  id_file_created: boolean;
}

export interface RenameProjectRequest {
  project_id: string;
  new_name: string;
}

// 文件类型
export interface FileNode {
  name: string;
  relative_path: string;
  absolute_path: string;
  is_dir: boolean;
  size: number | null;
  modified: string | null;
  is_hidden: boolean;
  error: string | null;
}

export type FileType =
  | "Markdown"
  | "Image"
  | "Svg"
  | "Text"
  | "Binary"
  | "Directory"
  | "NotFound"
  | "AccessDenied"
  | "TooLarge";

export interface FilePreview {
  file_type: FileType;
  content: string | null;
  size: number;
  encoding: string | null;
  error: string | null;
}

// 终端类型
export interface LaunchRequest {
  project_id: string;
  tool_kind: string;
  action_kind?: string;
}

export interface LaunchResult {
  success: boolean;
  message: string;
  executable: string;
  args: string[];
  working_directory: string;
}

export interface ToolAvailability {
  tool_kind: string;
  available: boolean;
  executable: string;
  version: string | null;
}

// 编辑器与 IDE
export interface EditorDescriptor {
  key: string;
  name: string;
  family: string;
  available: boolean;
  version: string | null;
  executable: string | null;
  source: string | null;
  supports_open_mode: boolean;
  supports_solution_target: boolean;
  is_custom: boolean;
}

export interface EditorTarget {
  relative_path: string | null;
  display_name: string;
  kind: string;
  recommended: boolean;
}

export interface EditorSettings {
  default_editor_key: string | null;
  open_mode: "default" | "new_window" | "reuse_window";
}

export interface ProjectEditorPreference {
  project_id: string;
  editor_key: string;
  target_relative_path: string | null;
  open_mode: "default" | "new_window" | "reuse_window";
  updated_at: string;
}

export interface EditorProfile {
  id: string;
  project_id: string | null;
  name: string;
  executable: string;
  args_json: string;
  working_directory: string | null;
  sort_order: number;
  enabled: boolean;
}

export interface EditorProfileInput {
  project_id?: string | null;
  name: string;
  executable: string;
  args: string[];
  working_directory?: string | null;
  sort_order?: number;
  enabled?: boolean;
}

export interface LaunchEditorRequest {
  project_id: string;
  editor_key?: string;
  target_relative_path?: string | null;
  open_mode?: "default" | "new_window" | "reuse_window";
  remember_for_project?: boolean;
}

export interface LaunchEditorResult {
  editor_key: string;
  editor_name: string;
  executable: string;
  target_display: string;
  used_project_default: boolean;
}

// 日志类型
export interface ProjectLog {
  id: string;
  project_id: string;
  relative_path: string;
  content_hash: string;
  agent: string;
  status: string;
  title: string | null;
  started_at: string | null;
  finished_at: string;
  time_inferred: boolean;
  parse_status: string;
  parse_error: string | null;
  indexed_at: string;
}

export interface LogSyncResult {
  project_id: string;
  scanned: number;
  added: number;
  updated: number;
  removed: number;
  errors: number;
  last_synced_at: string;
}

export interface HeatmapCell {
  date: string;
  count: number;
}

export interface ActivitySummary {
  total_tasks: number;
  completed: number;
  failed: number;
  blocked: number;
  heatmap: HeatmapCell[];
  period_start: string;
  period_end: string;
}

// 资料库类型
export interface VaultEntry {
  id: string;
  name: string;
  source_filename: string | null;
  encrypted_path: string;
  tags_json: string;
  created_at: string;
  updated_at: string;
  removed_at: string | null;
}

export interface VaultListItem {
  id: string;
  name: string;
  source_filename: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
  removed_at: string | null;
}

export interface VaultCreateRequest {
  name: string;
  content: string;
  tags?: string[];
  source_filename?: string;
}

export interface VaultUpdateRequest {
  id: string;
  name?: string;
  content?: string;
  tags?: string[];
}

export interface VaultImportRequest {
  name: string;
  source_file_path: string;
  tags?: string[];
}

export interface VaultContent {
  id: string;
  name: string;
  content: string;
  tags: string[];
  updated_at: string;
}

// 设置类型
export interface AppSettings {
  theme: string;
  minimize_to_tray: boolean;
  global_shortcut: string | null;
  launch_at_login: boolean;
  show_hidden_files: boolean;
  markdown_max_size_mb: number;
  image_max_size_mb: number;
  allow_remote_resources: boolean;
  clipboard_clear_seconds: number;
  vault_auto_mask_seconds: number;
  default_codex_action: string;
  default_claude_action: string;
  window_width: number | null;
  window_height: number | null;
  window_x: number | null;
  window_y: number | null;
  window_maximized: boolean;
  sidebar_collapsed: boolean;
  file_tree_width: number | null;
}

export interface SettingUpdate {
  key: string;
  value: string;
}

// 错误类型
export interface AppError {
  code: string;
  message: string;
  details: unknown;
}

// 异步状态
export type AsyncState = "idle" | "loading" | "success" | "error";
