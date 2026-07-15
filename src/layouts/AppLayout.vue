<template>
  <div class="app-layout">
    <aside class="app-sidebar" :class="{ collapsed: appStore.sidebarCollapsed }">
      <div class="logo">
        <img src="/logo.png" alt="江天一览图标" @error="onLogoError" />
        <div class="brand-copy">
          <span class="brand-name">江天一览</span>
          <small>RiverSkyView</small>
        </div>
      </div>
      <nav class="sidebar-nav">
        <div
          v-for="item in navItems"
          :key="item.route"
          class="sidebar-item"
          :class="{ active: isActive(item.route) }"
          @click="navigate(item.route)"
        >
          <el-icon><component :is="item.icon" /></el-icon>
          <span>{{ item.label }}</span>
          <span v-if="item.badge" class="sidebar-badge">{{ item.badge }}</span>
        </div>
      </nav>
    </aside>

    <div class="app-main">
      <header class="page-header">
        <div class="title">
          <slot name="title">{{ pageTitle }}</slot>
        </div>
        <div class="actions">
          <slot name="actions" />
        </div>
      </header>

      <main class="page-content">
        <slot />
      </main>

      <footer class="app-statusbar">
        <span>数据库: {{ statusText }}</span>
        <span>·</span>
        <span>{{ appStore.projectCount }} 个项目</span>
        <span v-if="appStore.lastSyncTime">·</span>
        <span v-if="appStore.lastSyncTime">最近同步: {{ appStore.lastSyncTime }}</span>
        <span style="margin-left: auto; cursor: pointer" @click="goSettings">v1.1.1</span>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useProjectStore } from "@/stores/project";
import {
  FolderOpened, Clock, DataLine, Delete, Setting,
} from "@element-plus/icons-vue";

const route = useRoute();
const router = useRouter();
const appStore = useAppStore();
const projectStore = useProjectStore();

const navItems = computed(() => [
  { route: "/", label: "项目", icon: FolderOpened },
  { route: "/recent", label: "最近使用", icon: Clock },
  { route: "/vault", label: "服务器资料", icon: DataLine },
  {
    route: "/project/recycle-bin",
    label: "回收站",
    icon: Delete,
    badge: projectStore.removedProjects.length || undefined,
  },
  { route: "/settings", label: "设置", icon: Setting },
]);

const pageTitle = computed(() => (route.meta.title as string) || "");
const statusText = computed(() => {
  switch (appStore.dbStatus) {
    case "ok": return "正常";
    case "error": return "异常";
    case "initializing": return "初始化中";
    default: return "未知";
  }
});

function isActive(routePath: string): boolean {
  if (routePath === "/") return route.path === "/";
  return route.path.startsWith(routePath);
}

function navigate(routePath: string) {
  router.push(routePath);
}

function goSettings() {
  router.push("/settings");
}

function onLogoError(e: Event) {
  (e.target as HTMLImageElement).style.display = "none";
}

// eslint-disable-next-line no-unused-vars
type KeyHandler = (event: KeyboardEvent) => void;
let keydownHandler: KeyHandler | null = null;

onMounted(async () => {
  await projectStore.loadProjects();
  await projectStore.loadRemovedProjects();

  keydownHandler = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      appStore.openCommandPalette();
    }
  };
  window.addEventListener("keydown", keydownHandler);
});

onUnmounted(() => {
  if (keydownHandler) {
    window.removeEventListener("keydown", keydownHandler);
  }
});
</script>
