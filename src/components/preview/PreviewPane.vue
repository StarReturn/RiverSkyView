<template>
  <div class="preview-pane">
    <!-- 无文件选中 -->
    <div v-if="!file" class="empty-state">
      <el-icon><Document /></el-icon>
      <p>选择文件预览</p>
    </div>

    <!-- 加载中 -->
    <div v-else-if="loading" class="empty-state">
      <el-icon class="is-loading"><Loading /></el-icon>
      <p>加载中...</p>
    </div>

    <!-- 文件预览 -->
    <template v-else-if="preview">
      <!-- 预览头部 -->
      <div class="preview-header">
        <span class="file-name">{{ file.name }}</span>
        <span class="file-path mono">{{ file.relative_path }}</span>
        <el-button size="small" text @click="copyPath">复制路径</el-button>
        <el-tag size="small" type="info">只读</el-tag>
      </div>

      <!-- Markdown 预览 -->
      <div v-if="preview.file_type === 'Markdown'" class="markdown-preview" v-html="renderedMarkdown">
      </div>

      <!-- 图片预览 -->
      <div v-else-if="preview.file_type === 'Image' || preview.file_type === 'Svg'" class="image-preview">
        <div class="image-toolbar">
          <el-button size="small" @click="zoomOut">缩小</el-button>
          <span>{{ Math.round(zoom * 100) }}%</span>
          <el-button size="small" @click="zoomIn">放大</el-button>
          <el-button size="small" @click="zoomReset">适应</el-button>
          <el-button size="small" @click="zoomOriginal">原始</el-button>
          <el-button size="small" @click="rotateLeft">左旋</el-button>
          <el-button size="small" @click="rotateRight">右旋</el-button>
          <el-button size="small" @click="reset">重置</el-button>
        </div>
        <img
          :src="imageSrc"
          :style="{ transform: `scale(${zoom}) rotate(${rotation}deg)` }"
          alt="预览"
        />
      </div>

      <!-- 文本预览 -->
      <pre v-else-if="preview.file_type === 'Text'" class="text-preview">{{ preview.content }}</pre>

      <!-- 超大文件 -->
      <div v-else-if="preview.file_type === 'TooLarge'" class="empty-state">
        <el-icon><WarningFilled /></el-icon>
        <p>{{ preview.error }}</p>
      </div>

      <!-- 二进制文件 -->
      <div v-else-if="preview.file_type === 'Binary'" class="empty-state">
        <el-icon><Document /></el-icon>
        <p>二进制文件，不支持预览</p>
        <p v-if="preview.error">{{ preview.error }}</p>
      </div>

      <!-- 错误 -->
      <div v-else class="empty-state">
        <el-icon><WarningFilled /></el-icon>
        <p>{{ preview.error || '无法预览此文件' }}</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { ElMessage } from "element-plus";
import {
  Document, Loading, WarningFilled,
} from "@element-plus/icons-vue";
import { filesApi } from "@/api/files";
import { renderMarkdown, sanitizeSvg } from "@/utils/sanitize";
import { ClipboardManager } from "@/utils/clipboard";
import { useSettingsStore } from "@/stores/settings";
import type { FileNode, FilePreview } from "@/types";

const props = defineProps<{
  projectId: string;
  file: FileNode | null;
}>();

const settingsStore = useSettingsStore();
const preview = ref<FilePreview | null>(null);
const loading = ref(false);
const zoom = ref(1);
const rotation = ref(0);

const renderedMarkdown = computed(() => {
  if (!preview.value?.content) return "";
  return renderMarkdown(preview.value.content);
});

const imageSrc = computed(() => {
  if (!preview.value?.content || !preview.value.encoding) return "";
  if (preview.value.file_type === "Svg") {
    const sanitized = sanitizeSvg(atob(preview.value.content));
    return `data:image/svg+xml;base64,${btoa(sanitized)}`;
  }
  return `data:${preview.value.encoding};base64,${preview.value.content}`;
});

async function loadPreview() {
  if (!props.file) {
    preview.value = null;
    return;
  }

  loading.value = true;
  zoom.value = 1;
  rotation.value = 0;

  try {
    preview.value = await filesApi.readFileForPreview(
      props.projectId,
      props.file.relative_path,
    );
  } catch (e) {
    preview.value = {
      file_type: "Binary",
      content: null,
      size: 0,
      encoding: null,
      error: (e as { message?: string })?.message || "预览失败",
    };
  } finally {
    loading.value = false;
  }
}

async function copyPath() {
  if (!props.file) return;
  await ClipboardManager.write(props.file.absolute_path, settingsStore.settings.clipboard_clear_seconds);
  ElMessage.success("路径已复制");
}

function zoomIn() { zoom.value = Math.min(zoom.value + 0.25, 5); }
function zoomOut() { zoom.value = Math.max(zoom.value - 0.25, 0.1); }
function zoomReset() { zoom.value = 1; rotation.value = 0; }
function zoomOriginal() { zoom.value = 1; }
function rotateLeft() { rotation.value -= 90; }
function rotateRight() { rotation.value += 90; }
function reset() { zoom.value = 1; rotation.value = 0; }

watch(() => props.file, loadPreview);
</script>

<style scoped>
.preview-pane {
  height: 100%;
  overflow: auto;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
  position: sticky;
  top: 0;
  z-index: 1;
}

.file-name {
  font-weight: 600;
}

.file-path {
  font-size: 12px;
  color: var(--text-secondary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.text-preview {
  padding: 24px;
  font-family: "Cascadia Mono", "Consolas", monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
