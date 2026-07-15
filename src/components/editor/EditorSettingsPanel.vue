<template>
  <div class="editor-settings-panel">
    <div class="settings-toolbar">
      <div>
        <h3>编辑器与 IDE</h3>
        <p>手动路径优先于自动检测；重新检测不会覆盖你的选择。</p>
      </div>
      <el-button :loading="editorStore.loading" @click="refresh">重新检测</el-button>
    </div>

    <el-form label-width="160px" class="editor-form">
      <el-form-item label="全局默认编辑器">
        <el-select v-model="defaultEditorKey" clearable placeholder="每次选择" style="width: 320px" @change="saveGeneral">
          <el-option
            v-for="editor in editorStore.editors"
            :key="editor.key"
            :label="editor.available ? editor.name : `${editor.name}（未检测到）`"
            :value="editor.key"
            :disabled="!editor.available"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="窗口打开方式">
        <el-select v-model="openMode" style="width: 320px" @change="saveGeneral">
          <el-option label="由编辑器决定" value="default" />
          <el-option label="新窗口" value="new_window" />
          <el-option label="复用窗口" value="reuse_window" />
        </el-select>
      </el-form-item>
    </el-form>

    <el-alert type="info" :closable="false" show-icon>
      自动检测只作为辅助。检测有误或使用绿色版时，请直接点击“选择路径”；手动路径失效后系统不会偷偷切换到其他版本。
    </el-alert>

    <el-table :data="editorStore.installations" size="small" border>
      <el-table-column label="开发工具" min-width="170">
        <template #default="{ row }">
          <div class="tool-name">
            <strong>{{ row.name }}</strong>
            <small v-if="row.version">{{ row.version }}</small>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="105">
        <template #default="{ row }">
          <el-tag :type="statusType(row.verification_status, row.available)" size="small">
            {{ statusLabel(row.verification_status, row.available) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="来源" width="105">
        <template #default="{ row }">
          {{ row.active_source === 'manual' ? '手动指定' : (row.detected_source ? '自动检测' : '未配置') }}
        </template>
      </el-table-column>
      <el-table-column label="当前生效路径" min-width="260">
        <template #default="{ row }">
          <div class="path-cell">
            <span class="path-text" :title="row.active_executable || ''">{{ row.active_executable || '尚未配置' }}</span>
            <small v-if="row.last_error" class="path-error">{{ row.last_error }}</small>
            <small
              v-else-if="row.active_source === 'manual' && row.detected_executable && row.detected_executable !== row.manual_executable"
              class="detected-path"
              :title="row.detected_executable"
            >
              自动检测：{{ row.detected_executable }}
            </small>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="启用" width="70" align="center">
        <template #default="{ row }">
          <el-switch
            v-model="row.enabled"
            :loading="isActing(row.editor_key)"
            @change="toggleEnabled(row.editor_key, Boolean($event))"
          />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="310" fixed="right">
        <template #default="{ row }">
          <div class="row-actions">
            <el-button text size="small" :loading="isActing(row.editor_key)" @click="chooseBuiltinExecutable(row.editor_key)">
              选择路径
            </el-button>
            <el-button text size="small" :disabled="!row.available" :loading="isActing(row.editor_key)" @click="testBuiltin(row.editor_key)">
              测试
            </el-button>
            <el-dropdown trigger="click" @command="(command: string) => handleInstallationCommand(command, row)">
              <el-button text size="small">更多</el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="refresh">重新检测</el-dropdown-item>
                  <el-dropdown-item command="verify" :disabled="!row.active_executable">验证路径</el-dropdown-item>
                  <el-dropdown-item command="location" :disabled="!row.available">打开程序位置</el-dropdown-item>
                  <el-dropdown-item command="auto" :disabled="row.active_source !== 'manual'" divided>
                    恢复自动检测
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </template>
      </el-table-column>
    </el-table>

    <div class="custom-header">
      <div>
        <h3>自定义编辑器</h3>
        <p>仅允许直接启动 .exe 或 .com，不执行 CMD、PowerShell 或脚本。</p>
      </div>
      <el-button type="primary" @click="openCreate">新增自定义编辑器</el-button>
    </div>

    <el-table :data="editorStore.profiles" size="small" border empty-text="尚未添加自定义编辑器">
      <el-table-column label="名称" prop="name" width="180" />
      <el-table-column label="可执行文件" min-width="300">
        <template #default="{ row }"><span class="path-text">{{ row.executable }}</span></template>
      </el-table-column>
      <el-table-column label="参数" min-width="180">
        <template #default="{ row }">{{ formatArgs(row.args_json) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="140">
        <template #default="{ row }">
          <el-button text size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text size="small" type="danger" @click="removeProfile(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑自定义编辑器' : '新增自定义编辑器'" width="620px">
      <el-form label-width="110px">
        <el-form-item label="名称" required>
          <el-input v-model="profileName" maxlength="80" />
        </el-form-item>
        <el-form-item label="可执行文件" required>
          <el-input v-model="profileExecutable" readonly placeholder="选择 .exe 或 .com">
            <template #append><el-button @click="chooseExecutable">选择</el-button></template>
          </el-input>
        </el-form-item>
        <el-form-item label="参数">
          <el-input v-model="profileArgsText" type="textarea" :rows="5" />
          <div class="form-help">每行一个参数。允许：{projectPath}、{projectName}、{targetPath}</div>
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="profileEnabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveProfile">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { open } from "@tauri-apps/plugin-dialog";
import { useEditorStore } from "@/stores/editor";
import { editorApi } from "@/api/editor";
import type { AppError, EditorInstallation, EditorProfile, EditorVerificationStatus } from "@/types";

const editorStore = useEditorStore();
const defaultEditorKey = ref<string | null>(null);
const openMode = ref<"default" | "new_window" | "reuse_window">("default");
const dialogVisible = ref(false);
const editingId = ref("");
const profileName = ref("");
const profileExecutable = ref("");
const profileArgsText = ref("{projectPath}");
const profileEnabled = ref(true);

function syncSettings() {
  defaultEditorKey.value = editorStore.settings.default_editor_key;
  openMode.value = editorStore.settings.open_mode;
}

async function refresh() {
  try {
    await editorStore.refresh();
    syncSettings();
    ElMessage.success("编辑器检测已刷新");
  } catch (error) {
    ElMessage.error((error as AppError).message || "刷新编辑器检测失败");
  }
}

function isActing(editorKey: string) {
  return editorStore.installationActionKey === editorKey;
}

function statusType(status: EditorVerificationStatus, available: boolean) {
  if (available && status === "valid") return "success";
  if (status === "launch_failed") return "warning";
  if (status === "missing" || status === "invalid") return "danger";
  return "info";
}

function statusLabel(status: EditorVerificationStatus, available: boolean) {
  if (available && status === "valid") return "可用";
  if (status === "launch_failed") return "启动失败";
  if (status === "invalid") return "路径无效";
  if (status === "missing") return "未配置";
  return "待验证";
}

async function chooseBuiltinExecutable(editorKey: string) {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "可执行程序", extensions: ["exe", "com"] }],
    });
    if (typeof selected !== "string") return;
    await editorStore.setManualExecutable(editorKey, selected);
    ElMessage.success("手动路径已保存，将优先使用该路径");
  } catch (error) {
    ElMessage.error((error as AppError).message || "保存开发工具路径失败");
  }
}

async function testBuiltin(editorKey: string) {
  try {
    const result = await editorStore.testLaunch(editorKey);
    ElMessage.success(result.message);
  } catch (error) {
    ElMessage.error((error as AppError).message || "测试启动失败");
  }
}

async function toggleEnabled(editorKey: string, enabled: boolean) {
  try {
    await editorStore.setEnabled(editorKey, enabled);
    ElMessage.success(enabled ? "开发工具已启用" : "开发工具已禁用");
  } catch (error) {
    await editorStore.load(undefined, true);
    ElMessage.error((error as AppError).message || "更新启用状态失败");
  }
}

async function handleInstallationCommand(command: string, row: EditorInstallation) {
  try {
    if (command === "refresh") {
      await editorStore.refreshDetection(row.editor_key);
      ElMessage.success(row.name + " 检测结果已刷新");
    } else if (command === "verify") {
      await editorStore.verifyExecutable(row.editor_key);
      ElMessage.success("路径验证通过");
    } else if (command === "location") {
      await editorApi.openLocation(row.editor_key);
    } else if (command === "auto") {
      await ElMessageBox.confirm(
        "恢复后将不再固定使用当前手动路径，是否继续？",
        "恢复自动检测",
        { type: "warning", confirmButtonText: "恢复自动", cancelButtonText: "取消" },
      );
      await editorStore.clearManualExecutable(row.editor_key);
      ElMessage.success("已恢复自动检测");
    }
  } catch (error) {
    if ((error as Error)?.message === "cancel") return;
    ElMessage.error((error as AppError).message || "开发工具操作失败");
  }
}

async function saveGeneral() {
  try {
    await editorStore.saveSettings({
      default_editor_key: defaultEditorKey.value || null,
      open_mode: openMode.value,
    });
    ElMessage.success("编辑器设置已保存");
  } catch (error) {
    ElMessage.error((error as AppError).message || "保存编辑器设置失败");
  }
}

function resetForm() {
  editingId.value = "";
  profileName.value = "";
  profileExecutable.value = "";
  profileArgsText.value = "{projectPath}";
  profileEnabled.value = true;
}

function openCreate() {
  resetForm();
  dialogVisible.value = true;
}

function openEdit(profile: EditorProfile) {
  editingId.value = profile.id;
  profileName.value = profile.name;
  profileExecutable.value = profile.executable;
  profileArgsText.value = JSON.parse(profile.args_json || "[]").join("\n") || "{projectPath}";
  profileEnabled.value = profile.enabled;
  dialogVisible.value = true;
}

async function chooseExecutable() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "可执行程序", extensions: ["exe", "com"] }],
  });
  if (typeof selected === "string") profileExecutable.value = selected;
}

async function saveProfile() {
  const input = {
    project_id: null,
    name: profileName.value.trim(),
    executable: profileExecutable.value,
    args: profileArgsText.value.split(/\r?\n/).map((value) => value.trim()).filter(Boolean),
    working_directory: "{projectPath}",
    enabled: profileEnabled.value,
  };
  try {
    if (editingId.value) await editorStore.updateProfile(editingId.value, input);
    else await editorStore.createProfile(input);
    dialogVisible.value = false;
    ElMessage.success("自定义编辑器已保存");
  } catch (error) {
    ElMessage.error((error as AppError).message || "保存自定义编辑器失败");
  }
}

async function removeProfile(id: string) {
  try {
    await ElMessageBox.confirm("删除后，使用该配置的项目默认项也会被清除。", "删除自定义编辑器", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
    await editorStore.deleteProfile(id);
    syncSettings();
    ElMessage.success("自定义编辑器已删除");
  } catch {
    // 用户取消或错误提示由 API 处理
  }
}

function formatArgs(argsJson: string) {
  try {
    return (JSON.parse(argsJson) as string[]).join(" ") || "{projectPath}";
  } catch {
    return "参数格式异常";
  }
}

onMounted(async () => {
  await editorStore.load();
  syncSettings();
});
</script>

<style scoped>
.editor-settings-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.settings-toolbar,
.custom-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.settings-toolbar p,
.custom-header p,
.form-help {
  color: var(--text-secondary);
  font-size: 12px;
}

.editor-form {
  max-width: 600px;
}

.form-help {
  margin-top: 6px;
}

.tool-name,
.path-cell {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.tool-name small,
.detected-path {
  overflow: hidden;
  color: var(--text-secondary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-text {
  overflow: hidden;
  font-family: Consolas, monospace;
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-error {
  color: var(--danger);
  font-size: 11px;
}

.row-actions {
  display: flex;
  align-items: center;
  white-space: nowrap;
}
</style>
