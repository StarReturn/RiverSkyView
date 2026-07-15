import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import ElementPlus from "element-plus";

const mockInvoke = vi.fn();
const mockOpen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: unknown[]) => mockOpen(...args),
}));

import EditorSettingsPanel from "@/components/editor/EditorSettingsPanel.vue";

const installation = {
  editor_key: "builtin:vscode",
  name: "Visual Studio Code",
  family: "vscode",
  manual_executable: "D:\\Portable\\Code.exe",
  detected_executable: "C:\\Program Files\\Microsoft VS Code\\Code.exe",
  active_executable: "D:\\Portable\\Code.exe",
  active_source: "manual",
  available: true,
  enabled: true,
  verification_status: "valid",
  detected_source: "known-path",
  version: null,
  last_detected_at: null,
  last_verified_at: null,
  last_error: null,
};

describe("EditorSettingsPanel", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
    mockOpen.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "list_editors") {
        return Promise.resolve([{
          key: "builtin:vscode",
          name: "Visual Studio Code",
          family: "vscode",
          available: true,
          version: null,
          executable: installation.active_executable,
          source: "manual",
          supports_open_mode: true,
          supports_solution_target: false,
          is_custom: false,
        }]);
      }
      if (command === "list_editor_installations") return Promise.resolve([installation]);
      if (command === "get_editor_settings") {
        return Promise.resolve({ default_editor_key: "builtin:vscode", open_mode: "default" });
      }
      if (command === "list_editor_profiles") return Promise.resolve([]);
      if (command === "set_editor_manual_executable") return Promise.resolve(installation);
      return Promise.resolve(null);
    });
  });

  it("shows that a manual executable overrides the detected path", async () => {
    const wrapper = mount(EditorSettingsPanel, {
      global: { plugins: [ElementPlus] },
    });
    await flushPromises();
    expect(wrapper.text()).toContain("手动指定");
    expect(wrapper.text()).toContain("D:\\Portable\\Code.exe");
    expect(wrapper.text()).toContain("自动检测：");
  });

  it("lets the user replace a built-in editor executable", async () => {
    mockOpen.mockResolvedValue("E:\\Apps\\Code.exe");
    const wrapper = mount(EditorSettingsPanel, {
      global: { plugins: [ElementPlus] },
    });
    await flushPromises();
    const choose = wrapper.findAll("button").find((button) => button.text().includes("选择路径"));
    expect(choose).toBeDefined();
    await choose!.trigger("click");
    await flushPromises();
    expect(mockInvoke).toHaveBeenCalledWith("set_editor_manual_executable", {
      editorKey: "builtin:vscode",
      executable: "E:\\Apps\\Code.exe",
    });
  });
});
