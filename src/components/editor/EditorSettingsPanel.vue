<template>
  <div class="editor-settings-panel">
    <div class="settings-toolbar">
      <div>
        <h3>编辑器与 IDE</h3>
        <p>管理全局默认编辑器、窗口方式和自定义程序。</p>
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

    <el-table :data="editorStore.editors" size="small" border>
      <el-table-column label="编辑器" prop="name" min-width="170" />
      <el-table-column label="状态" width="110">
        <template #default="{ row }">
          <el-tag :type="row.available ? 'success' : 'info'" size="small">
            {{ row.available ? '已检测' : '未检测到' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="来源" prop="source" width="140" />
      <el-table-column label="可执行文件" min-width="280">
        <template #default="{ row }"><span class="path-text">{{ row.executable || '—' }}</span></template>
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
import type { AppError, EditorProfile } from "@/types";

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
</style>

