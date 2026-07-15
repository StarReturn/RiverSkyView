import { invokeCommand } from "./index";
import type {
  EditorDescriptor,
  EditorInstallation,
  EditorProfile,
  EditorProfileInput,
  EditorSettings,
  EditorTarget,
  EditorTestLaunchResult,
  LaunchEditorRequest,
  LaunchEditorResult,
  ProjectEditorPreference,
} from "@/types";

export const editorApi = {
  list: (projectId?: string) =>
    invokeCommand<EditorDescriptor[]>("list_editors", { projectId }),
  refresh: (projectId?: string, editorKey?: string) =>
    invokeCommand<EditorDescriptor[]>("refresh_editor_detection", { projectId, editorKey }),
  listInstallations: () =>
    invokeCommand<EditorInstallation[]>("list_editor_installations"),
  setManualExecutable: (editorKey: string, executable: string) =>
    invokeCommand<EditorInstallation>("set_editor_manual_executable", { editorKey, executable }),
  clearManualExecutable: (editorKey: string) =>
    invokeCommand<EditorInstallation>("clear_editor_manual_executable", { editorKey }),
  verifyExecutable: (editorKey: string, executable?: string) =>
    invokeCommand<EditorInstallation>("verify_editor_executable", { editorKey, executable }),
  testLaunch: (editorKey: string) =>
    invokeCommand<EditorTestLaunchResult>("test_launch_editor", { editorKey }),
  setEnabled: (editorKey: string, enabled: boolean) =>
    invokeCommand<EditorInstallation>("set_editor_enabled", { editorKey, enabled }),
  openLocation: (editorKey: string) =>
    invokeCommand<void>("open_editor_location", { editorKey }),
  resolveTargets: (projectId: string, editorKey: string) =>
    invokeCommand<EditorTarget[]>("resolve_editor_targets", { projectId, editorKey }),
  launch: (request: LaunchEditorRequest) =>
    invokeCommand<LaunchEditorResult>("launch_project_editor", { request }),
  getSettings: () => invokeCommand<EditorSettings>("get_editor_settings"),
  setSettings: (input: EditorSettings) =>
    invokeCommand<EditorSettings>("set_editor_settings", { input }),
  getProjectPreference: (projectId: string) =>
    invokeCommand<ProjectEditorPreference | null>("get_project_editor_preference", { projectId }),
  setProjectPreference: (
    projectId: string,
    input: { editor_key: string; target_relative_path?: string | null; open_mode?: string },
  ) => invokeCommand<ProjectEditorPreference>("set_project_editor_preference", { projectId, input }),
  clearProjectPreference: (projectId: string) =>
    invokeCommand<void>("clear_project_editor_preference", { projectId }),
  listProfiles: (projectId?: string) =>
    invokeCommand<EditorProfile[]>("list_editor_profiles", { projectId }),
  createProfile: (input: EditorProfileInput) =>
    invokeCommand<EditorProfile>("create_editor_profile", { input }),
  updateProfile: (id: string, input: EditorProfileInput) =>
    invokeCommand<EditorProfile>("update_editor_profile", { id, input }),
  deleteProfile: (id: string) =>
    invokeCommand<void>("delete_editor_profile", { id }),
};
