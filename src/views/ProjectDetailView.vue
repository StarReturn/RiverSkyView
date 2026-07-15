<template>
  <AppLayout>
    <template #title>
      <div style="display: flex; align-items: center; gap: 8px">
        <el-button text :icon="ArrowLeft" @click="$router.push('/')">全部项目</el-button>
        <span>/ {{ project?.name }}</span>
      </div>
    </template>
    <template #actions>
      <el-button :icon="FolderOpened" @click="openExplorer">资源管理器</el-button>
      <el-button @click="launchCmd">CMD</el-button>
      <EditorLaunchButton :project-id="projectId" />
      <el-dropdown trigger="click" @command="handleCodexAction">
        <el-button>Codex ▾</el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="new">新建会话</el-dropdown-item>
            <el-dropdown-item command="resume_picker">恢复会话...</el-dropdown-item>
            <el-dropdown-item command="continue_latest">继续最近会话</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-dropdown trigger="click" @command="handleClaudeAction">
        <el-button>Claude ▾</el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="new">新建会话</el-dropdown-item>
            <el-dropdown-item command="resume_picker">恢复会话...</el-dropdown-item>
            <el-dropdown-item command="continue_latest">继续最近会话</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </template>

    <div v-if="project" class="project-detail">
      <!-- 项目路径 -->
      <div class="project-path-bar">
        <span class="path-text">{{ project.path }}</span>
      </div>

      <!-- 目录不可用警告 -->
      <el-alert
        v-if="!directoryAvailable"
        title="项目目录当前不可用"
        type="warning"
        :closable="false"
        show-icon
        style="margin-bottom: 16px"
      />

      <!-- 标签页 -->
      <el-tabs v-model="activeTab">
        <el-tab-pane label="文件" name="files">
          <FileWorkspace :project-id="projectId" :project-path="project.path" :disabled="!directoryAvailable" />
        </el-tab-pane>
        <el-tab-pane label="活动" name="activity">
          <ActivityWorkspace :project-id="projectId" />
        </el-tab-pane>
      </el-tabs>
    </div>

    <div v-else-if="loading" class="empty-state">
      <el-icon class="is-loading"><Loading /></el-icon>
      <p>加载中...</p>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import { ElMessage } from "element-plus";
import { ArrowLeft, FolderOpened, Loading } from "@element-plus/icons-vue";
import AppLayout from "@/layouts/AppLayout.vue";
import FileWorkspace from "@/components/file-tree/FileWorkspace.vue";
import ActivityWorkspace from "@/components/activity/ActivityWorkspace.vue";
import EditorLaunchButton from "@/components/editor/EditorLaunchButton.vue";
import { projectApi } from "@/api/project";
import { filesApi } from "@/api/files";
import { terminalApi } from "@/api/terminal";
import type { Project } from "@/types";

const route = useRoute();
const projectId = computed(() => route.params.id as string);

const project = ref<Project | null>(null);
const loading = ref(true);
const activeTab = ref("files");

const directoryAvailable = computed(() => {
  if (!project.value) return false;
  return true; // 简化：后端会校验路径
});

async function loadProject() {
  loading.value = true;
  try {
    project.value = await projectApi.get(projectId.value);
  } catch {
    ElMessage.error("加载项目失败");
  } finally {
    loading.value = false;
  }
}

async function openExplorer() {
  try {
    await filesApi.openInExplorer(projectId.value);
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "打开资源管理器失败");
  }
}

async function launchCmd() {
  try {
    await terminalApi.launch({ project_id: projectId.value, tool_kind: "cmd" });
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "启动 CMD 失败");
  }
}

async function handleCodexAction(action: string) {
  try {
    await terminalApi.launchCodex(projectId.value, action);
    await terminalApi.setProjectDefaultAction(projectId.value, "codex", action);
    ElMessage.success("Codex 已启动");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "启动 Codex 失败");
  }
}

async function handleClaudeAction(action: string) {
  try {
    await terminalApi.launchClaude(projectId.value, action);
    await terminalApi.setProjectDefaultAction(projectId.value, "claude", action);
    ElMessage.success("Claude Code 已启动");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "启动 Claude Code 失败");
  }
}

watch(projectId, loadProject);
onMounted(loadProject);
</script>

<style scoped>
.project-detail {
  display: flex;
  flex-direction: column;
}

.project-path-bar {
  padding: 8px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  margin-bottom: 16px;
}

.project-path-bar .path-text {
  font-size: 13px;
  word-break: break-all;
}
</style>
