import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { projectApi } from "@/api/project";
import type { ProjectListItem, AddProjectRequest, AddProjectResult } from "@/types";
import { useAppStore } from "./app";

export const useProjectStore = defineStore("project", () => {
  const projects = ref<ProjectListItem[]>([]);
  const removedProjects = ref<ProjectListItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const filterMode = ref<"all" | "favorites" | "recent">("all");
  const selectedIds = ref<Set<string>>(new Set());

  const filteredProjects = computed(() => {
    let list = projects.value;

    if (filterMode.value === "favorites") {
      list = list.filter((p) => p.is_favorite);
    } else if (filterMode.value === "recent") {
      list = list
        .filter((p) => p.last_opened_at || p.last_activity_at)
        .slice()
        .sort((a, b) => {
          const aTime = a.last_opened_at || a.last_activity_at || "";
          const bTime = b.last_opened_at || b.last_activity_at || "";
          return bTime.localeCompare(aTime);
        });
    }

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      list = list.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.path.toLowerCase().includes(q),
      );
    }

    return list;
  });

  const selectedProjects = computed(() =>
    filteredProjects.value.filter((p) => selectedIds.value.has(p.id)),
  );

  async function loadProjects() {
    loading.value = true;
    error.value = null;
    try {
      const [list, count] = await Promise.all([
        projectApi.list(),
        projectApi.count(),
      ]);
      projects.value = list;
      const appStore = useAppStore();
      appStore.setProjectCount(count);
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载项目失败";
    } finally {
      loading.value = false;
    }
  }

  async function loadRemovedProjects() {
    try {
      removedProjects.value = await projectApi.listRemoved();
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载回收站失败";
    }
  }

  async function addProject(request: AddProjectRequest): Promise<AddProjectResult> {
    const result = await projectApi.add(request);
    await loadProjects();
    return result;
  }

  async function renameProject(projectId: string, newName: string) {
    await projectApi.rename({ project_id: projectId, new_name: newName });
    await loadProjects();
  }

  async function toggleFavorite(projectId: string, favorite: boolean) {
    await projectApi.setFavorite(projectId, favorite);
    const p = projects.value.find((p) => p.id === projectId);
    if (p) p.is_favorite = favorite;
  }

  async function removeProject(projectId: string) {
    await projectApi.remove(projectId);
    await loadProjects();
  }

  async function batchRemoveProjects() {
    const ids = Array.from(selectedIds.value);
    if (ids.length === 0) return;
    await projectApi.batchRemove(ids);
    selectedIds.value.clear();
    await loadProjects();
  }

  async function restoreProject(projectId: string) {
    await projectApi.restore(projectId);
    await loadRemovedProjects();
    await loadProjects();
  }

  function toggleSelection(id: string) {
    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id);
    } else {
      selectedIds.value.add(id);
    }
    selectedIds.value = new Set(selectedIds.value);
  }

  function selectAll() {
    selectedIds.value = new Set(filteredProjects.value.map((p) => p.id));
  }

  function clearSelection() {
    selectedIds.value.clear();
    selectedIds.value = new Set();
  }

  return {
    projects,
    removedProjects,
    loading,
    error,
    searchQuery,
    filterMode,
    selectedIds,
    filteredProjects,
    selectedProjects,
    loadProjects,
    loadRemovedProjects,
    addProject,
    renameProject,
    toggleFavorite,
    removeProject,
    batchRemoveProjects,
    restoreProject,
    toggleSelection,
    selectAll,
    clearSelection,
  };
});
