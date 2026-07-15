import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { editorApi } from "@/api/editor";
import type {
  EditorDescriptor,
  EditorProfile,
  EditorProfileInput,
  EditorSettings,
  LaunchEditorRequest,
} from "@/types";

export const useEditorStore = defineStore("editor", () => {
  const editors = ref<EditorDescriptor[]>([]);
  const profiles = ref<EditorProfile[]>([]);
  const settings = ref<EditorSettings>({ default_editor_key: null, open_mode: "default" });
  const loading = ref(false);
  const launching = ref(false);
  const loaded = ref(false);

  const availableEditors = computed(() => editors.value.filter((editor) => editor.available));

  async function load(projectId?: string, force = false) {
    if (loaded.value && !force && !projectId) return;
    loading.value = true;
    try {
      const [editorList, editorSettings, profileList] = await Promise.all([
        editorApi.list(projectId),
        editorApi.getSettings(),
        editorApi.listProfiles(projectId),
      ]);
      editors.value = editorList;
      settings.value = editorSettings;
      profiles.value = profileList;
      if (!projectId) loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function refresh(projectId?: string) {
    loading.value = true;
    try {
      editors.value = await editorApi.refresh(projectId);
      profiles.value = await editorApi.listProfiles(projectId);
      settings.value = await editorApi.getSettings();
      loaded.value = !projectId;
    } finally {
      loading.value = false;
    }
  }

  async function saveSettings(value: EditorSettings) {
    settings.value = await editorApi.setSettings(value);
  }

  async function launch(request: LaunchEditorRequest) {
    launching.value = true;
    try {
      return await editorApi.launch(request);
    } finally {
      launching.value = false;
    }
  }

  async function createProfile(input: EditorProfileInput) {
    await editorApi.createProfile(input);
    await refresh(input.project_id || undefined);
  }

  async function updateProfile(id: string, input: EditorProfileInput) {
    await editorApi.updateProfile(id, input);
    await refresh(input.project_id || undefined);
  }

  async function deleteProfile(id: string) {
    await editorApi.deleteProfile(id);
    await refresh();
  }

  return {
    editors,
    profiles,
    settings,
    loading,
    launching,
    loaded,
    availableEditors,
    load,
    refresh,
    saveSettings,
    launch,
    createProfile,
    updateProfile,
    deleteProfile,
  };
});

