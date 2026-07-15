<template>
  <div class="lazy-file-tree">
    <div
      v-for="node in nodes"
      :key="node.relative_path"
      class="tree-node-wrapper"
    >
      <div
        class="tree-node"
        :class="{ selected: selectedPath === node.relative_path }"
        :style="{ paddingLeft: depth * 16 + 12 + 'px' }"
        @click="onNodeClick(node)"
        @contextmenu="onContextMenu($event, node)"
      >
        <el-icon v-if="node.is_dir" class="expand-icon">
          <ArrowRight v-if="!isExpanded(node.relative_path)" />
          <ArrowDown v-else />
        </el-icon>
        <span v-else class="expand-icon-spacer" />

        <el-icon v-if="node.is_dir" class="file-icon"><Folder /></el-icon>
        <el-icon v-else class="file-icon"><Document /></el-icon>

        <span class="name" :title="node.name">{{ node.name }}</span>

        <el-icon v-if="node.error" class="error-icon" :title="node.error">
          <WarningFilled />
        </el-icon>
      </div>

      <!-- 递归子目录 -->
      <template v-if="node.is_dir && isExpanded(node.relative_path)">
        <LazyFileTree
          v-if="childNodes[node.relative_path]"
          :project-id="projectId"
          :project-path="projectPath"
          :relative-dir="node.relative_path"
          :depth="depth + 1"
          :filter="filter"
          :selected-path="selectedPath"
          @select="$emit('select', $event)"
          @contextmenu="onChildContextmenu"
        />
        <div v-else class="tree-node loading" :style="{ paddingLeft: (depth + 1) * 16 + 12 + 'px' }">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>加载中...</span>
        </div>
      </template>
    </div>

    <div v-if="nodes.length === 0 && !loading" class="empty-tree">
      <span>{{ filter ? '无匹配文件' : '空目录' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import {
  ArrowRight, ArrowDown, Folder, Document, WarningFilled, Loading,
} from "@element-plus/icons-vue";
import { filesApi } from "@/api/files";
import type { FileNode } from "@/types";

const props = withDefaults(defineProps<{
  projectId: string;
  projectPath: string;
  relativeDir?: string;
  depth?: number;
  filter?: string;
  selectedPath?: string;
  disabled?: boolean;
}>(), {
  relativeDir: "",
  depth: 0,
  filter: "",
  selectedPath: "",
  disabled: false,
});

const emit = defineEmits<{
  select: [file: FileNode];
  contextmenu: [event: MouseEvent, node: FileNode];
}>();

// 使用自身组件递归
defineOptions({ name: "LazyFileTree" });

const nodes = ref<FileNode[]>([]);
const childNodes = ref<Record<string, FileNode[]>>({});
const expandedPaths = ref<Set<string>>(new Set());
const loading = ref(false);

function isExpanded(path: string): boolean {
  return expandedPaths.value.has(path);
}

async function loadNodes() {
  if (props.disabled) return;
  loading.value = true;
  try {
    const result = await filesApi.listDirectory(props.projectId, props.relativeDir);
    if (props.filter) {
      nodes.value = result.filter(
        (n) => n.is_dir || n.name.toLowerCase().includes(props.filter.toLowerCase()),
      );
    } else {
      nodes.value = result;
    }
  } catch {
    nodes.value = [];
  } finally {
    loading.value = false;
  }
}

async function onNodeClick(node: FileNode) {
  if (node.is_dir) {
    const path = node.relative_path;
    if (isExpanded(path)) {
      expandedPaths.value.delete(path);
    } else {
      expandedPaths.value.add(path);
      if (!childNodes.value[path]) {
        try {
          childNodes.value[path] = await filesApi.listDirectory(props.projectId, path);
        } catch {
          childNodes.value[path] = [];
        }
      }
    }
    expandedPaths.value = new Set(expandedPaths.value);
  } else {
    emit("select", node);
  }
}

function onContextMenu(event: MouseEvent, node: FileNode) {
  emit("contextmenu", event, node);
}

function onChildContextmenu(event: MouseEvent, node: FileNode) {
  emit("contextmenu", event, node);
}

watch(() => props.filter, () => {
  if (props.depth === 0) loadNodes();
});

watch(() => props.relativeDir, loadNodes);

onMounted(loadNodes);
</script>

<style scoped>
.lazy-file-tree {
  user-select: none;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  cursor: pointer;
  white-space: nowrap;
  font-size: 13px;
}

.tree-node:hover {
  background: var(--bg-subtle);
}

.tree-node.selected {
  background: rgba(37, 99, 235, 0.1);
}

.expand-icon, .expand-icon-spacer {
  min-width: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}

.file-icon {
  font-size: 14px;
  color: var(--text-secondary);
}

.tree-node .name {
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.error-icon {
  color: var(--warning);
  font-size: 12px;
}

.tree-node.loading {
  color: var(--text-secondary);
  font-size: 12px;
  padding: 4px 12px;
}

.empty-tree {
  padding: 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 12px;
}
</style>
