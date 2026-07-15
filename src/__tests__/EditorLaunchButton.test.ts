import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { createMemoryHistory, createRouter } from "vue-router";
import ElementPlus from "element-plus";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

import EditorLaunchButton from "@/components/editor/EditorLaunchButton.vue";

describe("EditorLaunchButton", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "list_editors") {
        return Promise.resolve([{
          key: "builtin:vscode",
          name: "Visual Studio Code",
          family: "vscode",
          available: true,
          version: null,
          executable: "C:\\Code.exe",
          source: "known-path",
          supports_open_mode: true,
          supports_solution_target: false,
          is_custom: false,
        }]);
      }
      if (command === "get_editor_settings") {
        return Promise.resolve({ default_editor_key: "builtin:vscode", open_mode: "default" });
      }
      if (command === "list_editor_profiles") return Promise.resolve([]);
      if (command === "list_editor_installations") return Promise.resolve([]);
      if (command === "launch_project_editor") {
        return Promise.resolve({
          editor_key: "builtin:vscode",
          editor_name: "Visual Studio Code",
          executable: "C:\\Code.exe",
          target_display: "D:\\Demo",
          used_project_default: false,
        });
      }
      return Promise.resolve(null);
    });
  });

  it("launches the configured default editor from the main button", async () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: "/", component: { template: "<div />" } }],
    });
    const wrapper = mount(EditorLaunchButton, {
      props: { projectId: "project-1" },
      global: { plugins: [router, ElementPlus] },
    });
    await flushPromises();
    const button = wrapper.findAll("button").find((item) => item.text().includes("编辑器"));
    expect(button).toBeDefined();
    await button!.trigger("click");
    await flushPromises();
    expect(mockInvoke).toHaveBeenCalledWith("launch_project_editor", {
      request: { project_id: "project-1" },
    });
  });
});
