import { describe, it, expect } from "vitest";
import router from "@/router";

describe("Router configuration", () => {
  it("has flat routes (no parent AppLayout wrapper)", () => {
    const routes = router.getRoutes();

    // 确保路由不是嵌套在 AppLayout 下
    const hasParentLayout = routes.some(
      (r) => r.components?.default && r.children && r.children.length > 0,
    );

    // 路由应该是扁平的，不应该有子路由嵌套在 AppLayout 下
    expect(hasParentLayout).toBe(false);
  });

  it("has route for projects (home page)", () => {
    const route = router.getRoutes().find((r) => r.path === "/");
    expect(route).toBeDefined();
    expect(route?.name).toBe("projects");
  });

  it("has route for favorites", () => {
    const route = router.getRoutes().find((r) => r.path === "/favorites");
    expect(route).toBeDefined();
    expect(route?.name).toBe("favorites");
    expect(route?.redirect).toEqual({ path: "/", query: { filter: "favorites" } });
  });

  it("has route for recent", () => {
    const route = router.getRoutes().find((r) => r.path === "/recent");
    expect(route).toBeDefined();
    expect(route?.name).toBe("recent");
  });

  it("has route for project detail", () => {
    const route = router.getRoutes().find((r) => r.path === "/project/:id");
    expect(route).toBeDefined();
    expect(route?.name).toBe("project-detail");
  });

  it("has route for project recycle bin", () => {
    const route = router.getRoutes().find((r) => r.path === "/project/recycle-bin");
    expect(route).toBeDefined();
    expect(route?.name).toBe("project-recycle-bin");
  });

  it("has route for vault", () => {
    const route = router.getRoutes().find((r) => r.path === "/vault");
    expect(route).toBeDefined();
    expect(route?.name).toBe("vault");
  });

  it("has route for vault recycle bin", () => {
    const route = router.getRoutes().find((r) => r.path === "/vault/recycle-bin");
    expect(route).toBeDefined();
    expect(route?.name).toBe("vault-recycle-bin");
  });

  it("has route for settings", () => {
    const route = router.getRoutes().find((r) => r.path === "/settings");
    expect(route).toBeDefined();
    expect(route?.name).toBe("settings");
  });

  it("all routes have title meta", () => {
    const routes = router.getRoutes();
    for (const route of routes) {
      expect(route.meta?.title).toBeDefined();
      expect(typeof route.meta?.title).toBe("string");
    }
  });
});
