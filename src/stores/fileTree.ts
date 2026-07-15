import { defineStore } from "pinia";
import { ref } from "vue";
import { filesApi } from "@/api/files";
import type { FileNode } from "@/types";

export const useFileTreeStore = defineStore("fileTree", () => {
  const nodes = ref<Map<string, FileNode[]>>(new Map());
  const expandedPaths = ref<Set<string>>(new Set());
  const loadingPaths = ref<Set<string>>(new Set());
  const selectedFile = ref<FileNode | null>(null);
  const error = ref<string | null>(null);

  async function loadDirectory(projectId: string, relativeDir: string) {
    const key = relativeDir || ".";
    if (loadingPaths.value.has(key)) return;

    loadingPaths.value.add(key);
    error.value = null;

    try {
      const children = await filesApi.listDirectory(projectId, relativeDir);
      nodes.value.set(key, children);
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载目录失败";
      nodes.value.set(key, []);
    } finally {
      loadingPaths.value.delete(key);
    }
  }

  function toggleExpand(path: string) {
    if (expandedPaths.value.has(path)) {
      expandedPaths.value.delete(path);
    } else {
      expandedPaths.value.add(path);
    }
    expandedPaths.value = new Set(expandedPaths.value);
  }

  function selectFile(node: FileNode) {
    selectedFile.value = node;
  }

  function clearSelection() {
    selectedFile.value = null;
  }

  function reset() {
    nodes.value.clear();
    expandedPaths.value.clear();
    loadingPaths.value.clear();
    selectedFile.value = null;
    error.value = null;
  }

  return {
    nodes,
    expandedPaths,
    loadingPaths,
    selectedFile,
    error,
    loadDirectory,
    toggleExpand,
    selectFile,
    clearSelection,
    reset,
  };
});
