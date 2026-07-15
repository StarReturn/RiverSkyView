import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount, flushPromises } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { createRouter, createMemoryHistory } from "vue-router";
import ElementPlus from "element-plus";

// Mock Tauri invoke before importing components
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock clipboard manager
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(),
  readText: vi.fn(),
}));

// Mock dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

import ProjectListView from "@/views/ProjectListView.vue";
import AppLayout from "@/layouts/AppLayout.vue";

describe("ProjectListView smoke test", () => {
  let pinia: ReturnType<typeof createPinia>;
  let router: ReturnType<typeof createRouter>;

  beforeEach(() => {
    pinia = createPinia();
    setActivePinia(pinia);

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: "/", name: "projects", component: ProjectListView },
        { path: "/project/:id", name: "project-detail", component: { template: "<div/>" } },
      ],
    });

    // Mock Tauri invoke to return empty project list
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_projects") return Promise.resolve([]);
      if (cmd === "count_projects") return Promise.resolve(0);
      if (cmd === "list_removed_projects") return Promise.resolve([]);
      if (cmd === "check_tools_availability") return Promise.resolve([]);
      return Promise.resolve([]);
    });
  });

  it('renders "添加项目" button in the header', async () => {
    const wrapper = mount(ProjectListView, {
      global: {
        plugins: [pinia, router, ElementPlus],
        stubs: {
          AddProjectDialog: true,
        },
      },
    });

    await flushPromises();
    await flushPromises();

    // Check that "添加项目" text appears somewhere in the component
    const html = wrapper.html();
    expect(html).toContain("添加项目");
  });

  it('renders "添加第一个项目" in empty state when no projects exist', async () => {
    const wrapper = mount(ProjectListView, {
      global: {
        plugins: [pinia, router, ElementPlus],
        stubs: {
          AddProjectDialog: true,
        },
      },
    });

    // Wait for async operations to complete
    await flushPromises();
    await flushPromises();
    await flushPromises();

    const html = wrapper.html();
    // The empty state should show "添加第一个项目"
    expect(html).toContain("添加第一个项目");
  });

  it("renders search input", async () => {
    const wrapper = mount(ProjectListView, {
      global: {
        plugins: [pinia, router, ElementPlus],
        stubs: {
          AddProjectDialog: true,
        },
      },
    });

    await flushPromises();

    const html = wrapper.html();
    expect(html).toContain("搜索项目名称或路径");
  });

  it("renders filter radio buttons", async () => {
    const wrapper = mount(ProjectListView, {
      global: {
        plugins: [pinia, router, ElementPlus],
        stubs: {
          AddProjectDialog: true,
        },
      },
    });

    await flushPromises();

    const html = wrapper.html();
    expect(html).toContain("全部");
    expect(html).toContain("收藏");
    expect(html).toContain("最近");
  });
});

describe("AppLayout integration test", () => {
  it("renders sidebar with navigation items", () => {
    const pinia = createPinia();
    setActivePinia(pinia);

    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: "/", name: "projects", component: { template: "<div>content</div>" } }],
    });

    const wrapper = mount(AppLayout, {
      global: {
        plugins: [pinia, router, ElementPlus],
      },
      slots: {
        default: "<div>page content</div>",
      },
    });

    const html = wrapper.html();
    // Sidebar should contain navigation labels
    expect(html).toContain("项目");
    expect(html).toContain("设置");
    expect(html).toContain("江天一览");
    expect(html).toContain("RiverSkyView");
    expect(html).not.toContain(">收藏<");
    // Slot content should be rendered
    expect(html).toContain("page content");
  });
});
