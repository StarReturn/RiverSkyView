<template>
  <div class="file-workspace">
    <div class="file-tree-panel" :style="{ width: treeWidth + 'px' }">
      <div class="file-tree-header">
        <el-input
          v-model="filterText"
          placeholder="筛选当前目录..."
          size="small"
          :prefix-icon="Search"
          clearable
        />
        <el-button size="small" :icon="Refresh" @click="refresh" circle />
      </div>
      <div class="file-tree-body">
        <LazyFileTree
          :project-id="projectId"
          :project-path="projectPath"
          :filter="filterText"
          :disabled="disabled"
          @select="onFileSelect"
          @contextmenu="onContextMenu"
        />
      </div>
    </div>

    <div class="file-preview-panel">
      <PreviewPane
        :project-id="projectId"
        :file="selectedFile"
      />
    </div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <template v-if="contextMenu.node">
        <div v-if="contextMenu.node.is_dir" class="menu-item" @click="ctxAction('expand')">
          {{ expanded ? '收起' : '展开' }}
        </div>
        <div v-if="contextMenu.node.is_dir" class="menu-item" @click="ctxAction('explorer')">
          在资源管理器中打开
        </div>
        <div v-if="!contextMenu.node.is_dir" class="menu-item" @click="ctxAction('preview')">
          预览
        </div>
        <div v-if="!contextMenu.node.is_dir" class="menu-item" @click="ctxAction('copyFile')">
          复制文件
        </div>
        <div class="menu-item" @click="ctxAction('copyPath')">
          复制绝对路径
        </div>
        <div v-if="!contextMenu.node.is_dir" class="menu-item" @click="ctxAction('reveal')">
          在资源管理器中定位
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { ElMessage } from "element-plus";
import { Search, Refresh } from "@element-plus/icons-vue";
import LazyFileTree from "./LazyFileTree.vue";
import PreviewPane from "@/components/preview/PreviewPane.vue";
import { filesApi } from "@/api/files";
import { ClipboardManager } from "@/utils/clipboard";
import { useSettingsStore } from "@/stores/settings";
import type { FileNode } from "@/types";

const props = defineProps<{
  projectId: string;
  projectPath: string;
  disabled?: boolean;
}>();

const settingsStore = useSettingsStore();
const filterText = ref("");
const treeWidth = ref(settingsStore.settings.file_tree_width || 300);
const selectedFile = ref<FileNode | null>(null);
const expanded = ref(false);
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  node: null as FileNode | null,
});

function onFileSelect(file: FileNode) {
  selectedFile.value = file;
}

function onContextMenu(event: MouseEvent, node: FileNode) {
  event.preventDefault();
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    node,
  };
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

function refresh() {
  // 触发文件树刷新
  filterText.value = filterText.value + " ";
  setTimeout(() => {
    filterText.value = filterText.value.trim();
  }, 0);
}

async function ctxAction(action: string) {
  const node = contextMenu.value.node;
  if (!node) return;
  closeContextMenu();

  try {
    switch (action) {
      case "expand":
        expanded.value = !expanded.value;
        break;
      case "explorer":
        await filesApi.openInExplorer(props.projectId, node.relative_path);
        break;
      case "preview":
        selectedFile.value = node;
        break;
      case "copyFile":
        await filesApi.copyFileToClipboard(props.projectId, node.relative_path);
        ElMessage.success("文件已复制到剪贴板");
        break;
      case "copyPath":
        await ClipboardManager.write(node.absolute_path, settingsStore.settings.clipboard_clear_seconds);
        ElMessage.success(`路径已复制，将在 ${settingsStore.settings.clipboard_clear_seconds} 秒后清理剪贴板`);
        break;
      case "reveal":
        await filesApi.revealInExplorer(props.projectId, node.relative_path);
        break;
    }
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "操作失败");
  }
}

onMounted(() => {
  document.addEventListener("click", closeContextMenu);
});

onUnmounted(() => {
  document.removeEventListener("click", closeContextMenu);
});
</script>

<style scoped>
.file-workspace {
  display: flex;
  gap: 1px;
  height: calc(100vh - 280px);
  min-height: 400px;
  background: var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.file-tree-panel {
  background: var(--bg-surface);
  display: flex;
  flex-direction: column;
  min-width: 240px;
  max-width: 480px;
}

.file-preview-panel {
  flex: 1;
  background: var(--bg-surface);
  overflow: auto;
}

.context-menu {
  position: fixed;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 9999;
  min-width: 180px;
  padding: 4px 0;
}

.menu-item {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
}

.menu-item:hover {
  background: var(--bg-subtle);
}
</style>
