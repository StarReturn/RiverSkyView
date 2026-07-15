import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "projects",
    component: () => import("@/views/ProjectListView.vue"),
    meta: { title: "全部项目" },
  },
  {
    path: "/favorites",
    name: "favorites",
    redirect: { path: "/", query: { filter: "favorites" } },
    meta: { title: "收藏项目（兼容跳转）" },
  },
  {
    path: "/recent",
    name: "recent",
    component: () => import("@/views/ProjectListView.vue"),
    meta: { title: "最近使用", filter: "recent" },
  },
  {
    path: "/project/:id",
    name: "project-detail",
    component: () => import("@/views/ProjectDetailView.vue"),
    meta: { title: "项目详情" },
  },
  {
    path: "/project/recycle-bin",
    name: "project-recycle-bin",
    component: () => import("@/views/ProjectRecycleBinView.vue"),
    meta: { title: "项目回收站" },
  },
  {
    path: "/vault",
    name: "vault",
    component: () => import("@/views/VaultView.vue"),
    meta: { title: "服务器资料库" },
  },
  {
    path: "/vault/recycle-bin",
    name: "vault-recycle-bin",
    component: () => import("@/views/VaultRecycleBinView.vue"),
    meta: { title: "资料回收站" },
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/views/SettingsView.vue"),
    meta: { title: "设置" },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
