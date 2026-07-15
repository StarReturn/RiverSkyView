import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { editorApi } from "@/api/editor";
import type {
  EditorDescriptor,
  EditorInstallation,
  EditorProfile,
  EditorProfileInput,
  EditorSettings,
  LaunchEditorRequest,
} from "@/types";

export const useEditorStore = defineStore("editor", () => {
  const editors = ref<EditorDescriptor[]>([]);
  const profiles = ref<EditorProfile[]>([]);
  const installations = ref<EditorInstallation[]>([]);
  const settings = ref<EditorSettings>({ default_editor_key: null, open_mode: "default" });
  const loading = ref(false);
  const launching = ref(false);
  const installationActionKey = ref<string | null>(null);
  const loaded = ref(false);

  const availableEditors = computed(() => editors.value.filter((editor) => editor.available));

  async function load(projectId?: string, force = false) {
    if (loaded.value && !force && !projectId) return;
    loading.value = true;
    try {
      const [editorList, editorSettings, profileList, installationList] = await Promise.all([
        editorApi.list(projectId),
        editorApi.getSettings(),
        editorApi.listProfiles(projectId),
        editorApi.listInstallations(),
      ]);
      editors.value = editorList;
      settings.value = editorSettings;
      profiles.value = profileList;
      installations.value = installationList;
      if (!projectId) loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function refresh(projectId?: string, editorKey?: string) {
    loading.value = true;
    try {
      const [editorList, profileList, editorSettings, installationList] = await Promise.all([
        editorApi.refresh(projectId, editorKey),
        editorApi.listProfiles(projectId),
        editorApi.getSettings(),
        editorApi.listInstallations(),
      ]);
      editors.value = editorList;
      profiles.value = profileList;
      settings.value = editorSettings;
      installations.value = installationList;
      loaded.value = !projectId;
    } finally {
      loading.value = false;
    }
  }

  async function runInstallationAction<T>(editorKey: string, action: () => Promise<T>) {
    installationActionKey.value = editorKey;
    try {
      const result = await action();
      const [editorList, installationList] = await Promise.all([
        editorApi.list(),
        editorApi.listInstallations(),
      ]);
      editors.value = editorList;
      installations.value = installationList;
      return result;
    } finally {
      installationActionKey.value = null;
    }
  }

  const setManualExecutable = (editorKey: string, executable: string) =>
    runInstallationAction(editorKey, () => editorApi.setManualExecutable(editorKey, executable));

  const clearManualExecutable = (editorKey: string) =>
    runInstallationAction(editorKey, () => editorApi.clearManualExecutable(editorKey));

  const verifyExecutable = (editorKey: string) =>
    runInstallationAction(editorKey, () => editorApi.verifyExecutable(editorKey));

  const testLaunch = (editorKey: string) =>
    runInstallationAction(editorKey, () => editorApi.testLaunch(editorKey));

  const setEnabled = (editorKey: string, enabled: boolean) =>
    runInstallationAction(editorKey, () => editorApi.setEnabled(editorKey, enabled));

  const refreshDetection = (editorKey?: string) =>
    editorKey
      ? runInstallationAction(editorKey, () => editorApi.refresh(undefined, editorKey))
      : refresh();

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
    installations,
    settings,
    loading,
    launching,
    installationActionKey,
    loaded,
    availableEditors,
    load,
    refresh,
    saveSettings,
    launch,
    createProfile,
    updateProfile,
    deleteProfile,
    setManualExecutable,
    clearManualExecutable,
    verifyExecutable,
    testLaunch,
    setEnabled,
    refreshDetection,
  };
});
