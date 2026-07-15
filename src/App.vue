<template>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useAppStore } from "@/stores/app";

const settingsStore = useSettingsStore();
const appStore = useAppStore();

function applyTheme(theme: string) {
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    // 跟随系统
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (prefersDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }
}

watch(() => settingsStore.settings.theme, applyTheme);

onMounted(async () => {
  await settingsStore.load();
  applyTheme(settingsStore.settings.theme);
  appStore.checkTools();
  appStore.setDbStatus("ok");
});
</script>
