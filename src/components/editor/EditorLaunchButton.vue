<template>
  <div class="editor-launch-button" @click.stop>
    <el-button-group>
      <el-button size="small" :loading="editorStore.launching" @click="launchDefault">
        编辑器
      </el-button>
      <el-popover v-model:visible="menuVisible" placement="bottom-end" :width="340" trigger="click">
        <template #reference>
          <el-button size="small" aria-label="选择编辑器">
            <el-icon><ArrowDown /></el-icon>
          </el-button>
        </template>

        <div class="editor-menu" v-loading="editorStore.loading">
          <div class="menu-title">选择编辑器或 IDE</div>
          <div v-if="editorStore.editors.length === 0" class="menu-empty">尚未检测到编辑器</div>
          <div v-for="editor in editorStore.editors" :key="editor.key" class="editor-row">
            <div class="editor-info">
              <span class="status-dot" :class="{ available: editor.available }"></span>
              <div class="editor-text">
                <strong>{{ editor.name }}</strong>
                <small>{{ editor.available ? (editor.executable || '已检测') : '未检测到' }}</small>
              </div>
            </div>
            <div class="editor-actions">
              <el-button text size="small" :disabled="!editor.available" @click="openEditor(editor.key)">
                打开
              </el-button>
              <el-button text size="small" :disabled="!editor.available" @click="setProjectDefault(editor.key)">
                设为默认
              </el-button>
            </div>
          </div>
          <el-divider />
          <el-button text size="small" @click="goSettings">管理编辑器与 IDE</el-button>
        </div>
      </el-popover>
    </el-button-group>

    <el-dialog v-model="targetDialogVisible" title="选择要打开的项目目标" width="560px" append-to-body>
      <el-radio-group v-model="selectedTarget" class="target-list">
        <el-radio
          v-for="target in pendingTargets"
          :key="target.relative_path || '__folder__'"
          :value="target.relative_path || '__folder__'"
          border
        >
          {{ target.display_name }}
          <el-tag v-if="target.recommended" size="small" type="success">推荐</el-tag>
        </el-radio>
      </el-radio-group>
      <el-checkbox v-model="rememberTarget">以后此项目默认使用该编辑器和目标</el-checkbox>
      <template #footer>
        <el-button @click="targetDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="editorStore.launching" @click="confirmTargetLaunch">
          打开
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ArrowDown } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { editorApi } from "@/api/editor";
import { useEditorStore } from "@/stores/editor";
import type { AppError, EditorTarget } from "@/types";

const props = defineProps<{ projectId: string }>();
const router = useRouter();
const editorStore = useEditorStore();

const menuVisible = ref(false);
const targetDialogVisible = ref(false);
const pendingTargets = ref<EditorTarget[]>([]);
const pendingEditorKey = ref("");
const selectedTarget = ref("__folder__");
const rememberTarget = ref(false);

async function launchDefault() {
  try {
    const result = await editorStore.launch({ project_id: props.projectId });
    ElMessage.success(`已使用 ${result.editor_name} 打开项目`);
  } catch (error) {
    handleLaunchError(error as AppError);
  }
}

async function openEditor(editorKey: string) {
  menuVisible.value = false;
  try {
    const targets = await editorApi.resolveTargets(props.projectId, editorKey);
    const fileTargets = targets.filter((target) => target.relative_path);
    if (fileTargets.length > 1) {
      showTargetDialog(editorKey, targets);
      return;
    }
    const target = fileTargets[0]?.relative_path || null;
    const result = await editorStore.launch({
      project_id: props.projectId,
      editor_key: editorKey,
      target_relative_path: target,
    });
    ElMessage.success(`已使用 ${result.editor_name} 打开项目`);
  } catch (error) {
    handleLaunchError(error as AppError, editorKey);
  }
}

async function setProjectDefault(editorKey: string) {
  try {
    await editorApi.setProjectPreference(props.projectId, {
      editor_key: editorKey,
      open_mode: editorStore.settings.open_mode,
    });
    menuVisible.value = false;
    ElMessage.success("已设为此项目的默认编辑器");
  } catch (error) {
    ElMessage.error((error as AppError).message || "设置项目默认编辑器失败");
  }
}

function showTargetDialog(editorKey: string, targets: EditorTarget[]) {
  pendingEditorKey.value = editorKey;
  pendingTargets.value = targets;
  const recommended = targets.find((target) => target.recommended);
  selectedTarget.value = recommended?.relative_path || "__folder__";
  rememberTarget.value = false;
  targetDialogVisible.value = true;
}

async function confirmTargetLaunch() {
  try {
    const result = await editorStore.launch({
      project_id: props.projectId,
      editor_key: pendingEditorKey.value,
      target_relative_path: selectedTarget.value === "__folder__" ? null : selectedTarget.value,
      remember_for_project: rememberTarget.value,
    });
    targetDialogVisible.value = false;
    ElMessage.success(`已使用 ${result.editor_name} 打开项目`);
  } catch (error) {
    ElMessage.error((error as AppError).message || "启动编辑器失败");
  }
}

function handleLaunchError(error: AppError, fallbackEditorKey = "") {
  if (error.code === "EDITOR_NOT_CONFIGURED" || error.code === "EDITOR_NOT_FOUND") {
    menuVisible.value = true;
    ElMessage.warning(error.message || "请先选择编辑器");
    return;
  }
  if (error.code === "EDITOR_SELECTION_REQUIRED") {
    const details = error.details as { targets?: EditorTarget[]; editor_key?: string } | null;
    if (details?.targets?.length) {
      showTargetDialog(details.editor_key || fallbackEditorKey, details.targets);
      return;
    }
  }
  ElMessage.error(error.message || "启动编辑器失败");
}

function goSettings() {
  menuVisible.value = false;
  router.push({ path: "/settings", query: { tab: "editors" } });
}

onMounted(() => editorStore.load());
</script>

<style scoped>
.editor-launch-button {
  display: inline-flex;
}

.editor-menu {
  max-height: 440px;
  overflow: auto;
}

.menu-title {
  margin-bottom: 8px;
  font-weight: 600;
}

.menu-empty {
  padding: 20px 0;
  color: var(--text-secondary);
  text-align: center;
}

.editor-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.editor-info {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  flex: 0 0 8px;
  border-radius: 50%;
  background: var(--border);
}

.status-dot.available {
  background: var(--success);
}

.editor-text {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.editor-text small {
  max-width: 180px;
  overflow: hidden;
  color: var(--text-secondary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-actions {
  display: flex;
  flex: 0 0 auto;
}

.target-list {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.target-list :deep(.el-radio) {
  width: 100%;
  margin: 0;
}
</style>

