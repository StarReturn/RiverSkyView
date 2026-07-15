import { defineStore } from "pinia";
import { ref } from "vue";
import { projectLogApi } from "@/api/projectLog";
import type { ProjectLog, LogSyncResult, ActivitySummary } from "@/types";

export const useActivityStore = defineStore("activity", () => {
  const logs = ref<ProjectLog[]>([]);
  const activitySummary = ref<ActivitySummary | null>(null);
  const selectedLog = ref<ProjectLog | null>(null);
  const logContent = ref<string | null>(null);
  const loading = ref(false);
  const syncing = ref(false);
  const lastSyncResult = ref<LogSyncResult | null>(null);
  const error = ref<string | null>(null);

  const filterAgent = ref<string | null>(null);
  const filterStatus = ref<string | null>(null);
  const filterDate = ref<string | null>(null);

  async function syncLogs(projectId: string) {
    syncing.value = true;
    error.value = null;
    try {
      const result = await projectLogApi.sync(projectId);
      lastSyncResult.value = result;
      await Promise.all([
        loadLogs(projectId),
        loadActivitySummary(projectId),
      ]);
    } catch (e) {
      error.value = (e as { message?: string })?.message || "同步日志失败";
    } finally {
      syncing.value = false;
    }
  }

  async function loadLogs(projectId: string) {
    loading.value = true;
    try {
      logs.value = await projectLogApi.list(
        projectId,
        filterAgent.value || undefined,
        filterStatus.value || undefined,
        filterDate.value || undefined,
      );
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载日志失败";
    } finally {
      loading.value = false;
    }
  }

  async function loadActivitySummary(projectId: string) {
    try {
      activitySummary.value = await projectLogApi.getActivitySummary(projectId, 365);
    } catch {
      // 静默
    }
  }

  async function selectLog(projectId: string, log: ProjectLog) {
    selectedLog.value = log;
    logContent.value = null;
    try {
      logContent.value = await projectLogApi.getContent(projectId, log.id);
    } catch {
      logContent.value = null;
    }
  }

  function setFilter(agent?: string | null, status?: string | null, date?: string | null) {
    if (agent !== undefined) filterAgent.value = agent;
    if (status !== undefined) filterStatus.value = status;
    if (date !== undefined) filterDate.value = date;
  }

  function clearFilters() {
    filterAgent.value = null;
    filterStatus.value = null;
    filterDate.value = null;
  }

  function reset() {
    logs.value = [];
    activitySummary.value = null;
    selectedLog.value = null;
    logContent.value = null;
    lastSyncResult.value = null;
    error.value = null;
    clearFilters();
  }

  return {
    logs,
    activitySummary,
    selectedLog,
    logContent,
    loading,
    syncing,
    lastSyncResult,
    error,
    filterAgent,
    filterStatus,
    filterDate,
    syncLogs,
    loadLogs,
    loadActivitySummary,
    selectLog,
    setFilter,
    clearFilters,
    reset,
  };
});
