import { defineStore } from "pinia";
import { ref } from "vue";
import { settingsApi } from "@/api/settings";
import type { AppSettings, SettingUpdate } from "@/types";

const defaultSettings: AppSettings = {
  theme: "system",
  minimize_to_tray: true,
  global_shortcut: "Ctrl+Alt+P",
  launch_at_login: false,
  show_hidden_files: false,
  markdown_max_size_mb: 2,
  image_max_size_mb: 20,
  allow_remote_resources: false,
  clipboard_clear_seconds: 30,
  vault_auto_mask_seconds: 30,
  default_codex_action: "resume_picker",
  default_claude_action: "resume_picker",
  window_width: 1440,
  window_height: 900,
  window_x: null,
  window_y: null,
  window_maximized: false,
  sidebar_collapsed: false,
  file_tree_width: 300,
};

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>({ ...defaultSettings });
  const loading = ref(false);
  const dataDir = ref("");

  async function load() {
    loading.value = true;
    try {
      settings.value = await settingsApi.get();
      dataDir.value = await settingsApi.getDataDir();
    } catch {
      settings.value = { ...defaultSettings };
    } finally {
      loading.value = false;
    }
  }

  async function update(key: keyof AppSettings, value: string | boolean | number) {
    const strValue = String(value);
    const updates: SettingUpdate[] = [{ key: key as string, value: strValue }];
    const result = await settingsApi.updateMany(updates);
    settings.value = result;
  }

  async function updateMany(updates: SettingUpdate[]) {
    const result = await settingsApi.updateMany(updates);
    settings.value = result;
  }

  return {
    settings,
    loading,
    dataDir,
    load,
    update,
    updateMany,
  };
});
