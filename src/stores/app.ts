import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { terminalApi } from "@/api/terminal";
import type { ToolAvailability } from "@/types";

export const useAppStore = defineStore("app", () => {
  const sidebarCollapsed = ref(false);
  const commandPaletteOpen = ref(false);
  const toolsAvailability = ref<ToolAvailability[]>([]);
  const projectCount = ref(0);
  const dbStatus = ref<"ok" | "error" | "initializing">("initializing");
  const lastSyncTime = ref<string | null>(null);

  const availableTools = computed(() => {
    const map: Record<string, boolean> = {};
    for (const t of toolsAvailability.value) {
      map[t.tool_kind] = t.available;
    }
    return map;
  });

  async function checkTools() {
    try {
      toolsAvailability.value = await terminalApi.checkTools();
    } catch {
      // 静默失败
    }
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function openCommandPalette() {
    commandPaletteOpen.value = true;
  }

  function closeCommandPalette() {
    commandPaletteOpen.value = false;
  }

  function setDbStatus(status: "ok" | "error" | "initializing") {
    dbStatus.value = status;
  }

  function setLastSync(time: string) {
    lastSyncTime.value = time;
  }

  function setProjectCount(count: number) {
    projectCount.value = count;
  }

  return {
    sidebarCollapsed,
    commandPaletteOpen,
    toolsAvailability,
    availableTools,
    projectCount,
    dbStatus,
    lastSyncTime,
    checkTools,
    toggleSidebar,
    openCommandPalette,
    closeCommandPalette,
    setDbStatus,
    setLastSync,
    setProjectCount,
  };
});
