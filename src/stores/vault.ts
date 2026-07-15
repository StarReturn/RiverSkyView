import { defineStore } from "pinia";
import { ref } from "vue";
import { vaultApi } from "@/api/vault";
import type { VaultListItem, VaultCreateRequest, VaultUpdateRequest, VaultContent } from "@/types";

/**
 * 资料库 store：只保存元数据，明文只在短生命周期变量中持有
 * 明文内容不持久化到 localStorage 或 Pinia 插件
 */
export const useVaultStore = defineStore("vault", () => {
  const entries = ref<VaultListItem[]>([]);
  const removedEntries = ref<VaultListItem[]>([]);
  const selectedId = ref<string | null>(null);
  const selectedContent = ref<VaultContent | null>(null);
  const contentRevealed = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");

  async function loadEntries() {
    loading.value = true;
    error.value = null;
    try {
      if (searchQuery.value.trim()) {
        entries.value = await vaultApi.search(searchQuery.value);
      } else {
        entries.value = await vaultApi.list();
      }
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载资料失败";
    } finally {
      loading.value = false;
    }
  }

  async function loadRemovedEntries() {
    try {
      removedEntries.value = await vaultApi.listRemoved();
    } catch (e) {
      error.value = (e as { message?: string })?.message || "加载回收站失败";
    }
  }

  async function selectEntry(id: string) {
    // 选中只加载元数据，不自动解密
    selectedId.value = id;
    selectedContent.value = null;
    contentRevealed.value = false;
  }

  async function revealContent(id: string) {
    try {
      selectedContent.value = await vaultApi.getContent(id);
      contentRevealed.value = true;
    } catch (e) {
      error.value = (e as { message?: string })?.message || "解密失败";
    }
  }

  function maskContent() {
    contentRevealed.value = false;
    if (selectedContent.value) {
      // 清除明文
      selectedContent.value = null;
    }
  }

  async function createEntry(request: VaultCreateRequest) {
    await vaultApi.create(request);
    await loadEntries();
  }

  async function updateEntry(request: VaultUpdateRequest) {
    await vaultApi.update(request);
    selectedContent.value = null;
    contentRevealed.value = false;
    await loadEntries();
  }

  async function removeEntry(id: string) {
    await vaultApi.remove(id);
    if (selectedId.value === id) {
      selectedId.value = null;
      selectedContent.value = null;
      contentRevealed.value = false;
    }
    await loadEntries();
  }

  async function restoreEntry(id: string) {
    await vaultApi.restore(id);
    await loadRemovedEntries();
    await loadEntries();
  }

  async function permanentDelete(id: string) {
    await vaultApi.permanentDelete(id);
    await loadRemovedEntries();
  }

  function clearPlaintext() {
    selectedContent.value = null;
    contentRevealed.value = false;
  }

  return {
    entries,
    removedEntries,
    selectedId,
    selectedContent,
    contentRevealed,
    loading,
    error,
    searchQuery,
    loadEntries,
    loadRemovedEntries,
    selectEntry,
    revealContent,
    maskContent,
    createEntry,
    updateEntry,
    removeEntry,
    restoreEntry,
    permanentDelete,
    clearPlaintext,
  };
});
