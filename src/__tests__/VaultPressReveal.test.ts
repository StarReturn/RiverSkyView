import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import ElementPlus from "element-plus";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn(), readText: vi.fn() }));

import VaultView from "@/views/VaultView.vue";
import { useVaultStore } from "@/stores/vault";

describe("Vault press reveal", () => {
  beforeEach(() => {
    const pinia = createPinia();
    setActivePinia(pinia);
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "list_vault_entries") {
        return Promise.resolve([{
          id: "vault-1",
          name: "测试资料",
          source_filename: null,
          tags: [],
          created_at: "2026-07-15T00:00:00Z",
          updated_at: "2026-07-15T00:00:00Z",
          removed_at: null,
        }]);
      }
      if (command === "get_vault_content") {
        return Promise.resolve({
          id: "vault-1",
          name: "测试资料",
          content: "secret",
          tags: [],
          updated_at: "2026-07-15T00:00:00Z",
        });
      }
      return Promise.resolve(null);
    });
  });

  it("reveals only while pointer is held and masks on global pointerup", async () => {
    const wrapper = mount(VaultView, {
      global: {
        plugins: [ElementPlus],
        stubs: {
          AppLayout: { template: "<div><slot name='actions'/><slot/></div>" },
        },
      },
    });
    await flushPromises();
    const store = useVaultStore();
    await store.selectEntry("vault-1");
    await flushPromises();
    const holdButton = wrapper.findAll("button").find((item) => item.text().includes("按住显示"));
    expect(holdButton).toBeDefined();
    await holdButton!.trigger("pointerdown");
    await flushPromises();
    expect(store.contentRevealed).toBe(true);
    window.dispatchEvent(new window.Event("pointerup"));
    await flushPromises();
    expect(store.contentRevealed).toBe(false);
    expect(store.selectedContent).toBeNull();
  });
});
