<template>
  <AppLayout>
    <template #title>{{ pageTitle }}</template>
    <template #actions>
      <el-button v-if="!batchMode" @click="batchMode = true">批量操作</el-button>
      <el-button v-if="batchMode" @click="exitBatchMode">退出批量</el-button>
      <el-button type="primary" :icon="Plus" @click="openAddDialog">
        添加项目
      </el-button>
    </template>

    <div class="project-list-view">
      <!-- 工具栏 -->
      <div class="toolbar">
        <el-input
          v-model="projectStore.searchQuery"
          placeholder="搜索项目名称或路径..."
          :prefix-icon="Search"
          clearable
          style="width: 320px"
        />
        <el-radio-group v-model="projectStore.filterMode" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="favorites">收藏</el-radio-button>
          <el-radio-button value="recent">最近</el-radio-button>
        </el-radio-group>
      </div>

      <!-- 项目表格 -->
      <el-table
        ref="tableRef"
        :data="projectStore.filteredProjects"
        v-loading="projectStore.loading"
        style="width: 100%"
        @selection-change="onSelectionChange"
        empty-text=" "
      >
        <el-table-column v-if="batchMode" type="selection" width="40" />
        <el-table-column width="50">
          <template #default="{ row }">
            <el-icon
              :style="{ color: row.is_favorite ? '#F59E0B' : '#C0C4CC', cursor: 'pointer' }"
              @click.stop="toggleFavorite(row)"
            >
              <Star v-if="row.is_favorite" />
              <StarFilled v-else />
            </el-icon>
          </template>
        </el-table-column>
        <el-table-column label="项目名称" min-width="180">
          <template #default="{ row }">
            <span style="cursor: pointer; font-weight: 500" @click.stop="goDetail(row.id)">
              {{ row.name }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="项目路径" min-width="300">
          <template #default="{ row }">
            <span class="path-text" :title="row.path">{{ truncatePath(row.path, 60) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="最近活动" width="140">
          <template #default="{ row }">
            <span :title="formatDateTime(row.last_activity_at)">
              {{ formatRelativeTime(row.last_activity_at) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="80">
          <template #default="{ row }">
            <el-tag v-if="!row.directory_available" type="warning" size="small">不可用</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="370" fixed="right">
          <template #default="{ row }">
            <div class="row-actions">
              <el-button size="small" @click.stop="openExplorer(row.id)">目录</el-button>
              <el-button size="small" @click.stop="launchCmd(row.id)">CMD</el-button>
              <EditorLaunchButton :project-id="row.id" />
              <el-dropdown trigger="click" @command="(cmd: string) => handleAction(cmd, row)">
                <el-button size="small" @click.stop>
                  更多<el-icon class="el-icon--right"><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="detail">打开项目详情</el-dropdown-item>
                    <el-dropdown-item command="explorer">打开资源管理器</el-dropdown-item>
                    <el-dropdown-item command="cmd">打开 CMD</el-dropdown-item>
                    <el-dropdown-item command="codex">启动 Codex</el-dropdown-item>
                    <el-dropdown-item command="claude">启动 Claude Code</el-dropdown-item>
                    <el-dropdown-item command="rename">修改名称</el-dropdown-item>
                    <el-dropdown-item command="check">检查日志规则</el-dropdown-item>
                    <el-dropdown-item command="remove" divided>从系统移除</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </template>
        </el-table-column>

        <template #empty>
          <div class="empty-state">
            <el-icon><FolderOpened /></el-icon>
            <h3>尚未添加项目</h3>
            <p>选择一个本地项目目录，之后可以从这里快速打开终端和智能体。</p>
            <el-button class="empty-add-button" type="primary" :icon="Plus" @click="openAddDialog" style="margin-top: 16px">
              添加第一个项目
            </el-button>
          </div>
        </template>
      </el-table>

      <!-- 批量操作栏 -->
      <div v-if="batchMode && projectStore.selectedIds.size > 0" class="batch-bar">
        <span>已选择 {{ projectStore.selectedIds.size }} 项</span>
        <el-button @click="clearBatchSelection">取消选择</el-button>
        <el-button type="danger" @click="confirmBatchRemove">从系统移除</el-button>
      </div>
    </div>

    <!-- 添加项目对话框 -->
    <AddProjectDialog v-model:visible="addDialogVisible" @added="onProjectAdded" />

    <!-- 重命名对话框 -->
    <el-dialog v-model="renameDialogVisible" title="修改项目名称" width="420px">
      <el-input v-model="renameValue" placeholder="项目名称" />
      <template #footer>
        <el-button @click="renameDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="doRename">确认</el-button>
      </template>
    </el-dialog>

    <!-- 检查日志规则结果对话框 -->
    <el-dialog v-model="checkDialogVisible" title="日志规则检查结果" width="520px">
      <div v-if="checkResult" class="check-result">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="AGENTS.md">
            <el-tag :type="checkStatusTag(checkResult.agents_md)" size="small">
              {{ checkResult.agents_md }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="CLAUDE.md">
            <el-tag :type="checkStatusTag(checkResult.claude_md)" size="small">
              {{ checkResult.claude_md }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="pm_log 目录">
            <el-tag :type="checkResult.pm_log_exists ? 'success' : 'warning'" size="small">
              {{ checkResult.pm_log_exists ? "存在" : "不存在" }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label=".project-manager.json">
            <el-tag :type="checkResult.identifier_exists ? 'success' : 'warning'" size="small">
              {{ checkResult.identifier_exists ? "存在" : "不存在" }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
      </div>
      <template #footer>
        <el-button @click="checkDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import type { TableInstance } from "element-plus";
import {
  Plus, Search, Star, StarFilled, ArrowDown, FolderOpened,
} from "@element-plus/icons-vue";
import AppLayout from "@/layouts/AppLayout.vue";
import AddProjectDialog from "@/components/project/AddProjectDialog.vue";
import EditorLaunchButton from "@/components/editor/EditorLaunchButton.vue";
import { useProjectStore } from "@/stores/project";
import { terminalApi } from "@/api/terminal";
import { filesApi } from "@/api/files";
import { projectApi } from "@/api/project";
import { formatRelativeTime, formatDateTime, truncatePath } from "@/utils/format";
import type { ProjectListItem } from "@/types";

const router = useRouter();
const route = useRoute();
const projectStore = useProjectStore();

const tableRef = ref<TableInstance>();
const batchMode = ref(false);
const addDialogVisible = ref(false);
const renameDialogVisible = ref(false);
const renameValue = ref("");
const renameTargetId = ref("");
const checkDialogVisible = ref(false);
const checkResult = ref<Record<string, any> | null>(null);

const activeRouteFilter = computed<"all" | "favorites" | "recent">(() => {
  const queryFilter = route.query.filter;
  if (queryFilter === "favorites" || queryFilter === "recent") return queryFilter;
  const metaFilter = route.meta.filter;
  if (metaFilter === "favorites" || metaFilter === "recent") return metaFilter;
  return "all";
});

const pageTitle = computed(() => {
  const filter = activeRouteFilter.value;
  if (filter === "favorites") return "收藏项目";
  if (filter === "recent") return "最近使用";
  return "全部项目";
});

function goDetail(id: string) {
  router.push(`/project/${id}`);
}

function toggleFavorite(row: ProjectListItem) {
  projectStore.toggleFavorite(row.id, !row.is_favorite);
}

async function openExplorer(projectId: string) {
  try {
    await filesApi.openInExplorer(projectId);
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "打开资源管理器失败");
  }
}

async function launchCmd(projectId: string) {
  try {
    await terminalApi.launch({ project_id: projectId, tool_kind: "cmd" });
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "启动 CMD 失败");
  }
}

async function handleAction(cmd: string, row: ProjectListItem) {
  switch (cmd) {
    case "detail":
      goDetail(row.id);
      break;
    case "explorer":
      await openExplorer(row.id);
      break;
    case "cmd":
      await launchCmd(row.id);
      break;
    case "codex":
      try {
        await terminalApi.launchCodex(row.id);
        ElMessage.success("Codex 已启动");
      } catch (e) {
        ElMessage.error((e as { message?: string })?.message || "启动 Codex 失败");
      }
      break;
    case "claude":
      try {
        await terminalApi.launchClaude(row.id);
        ElMessage.success("Claude Code 已启动");
      } catch (e) {
        ElMessage.error((e as { message?: string })?.message || "启动 Claude Code 失败");
      }
      break;
    case "rename":
      renameTargetId.value = row.id;
      renameValue.value = row.name;
      renameDialogVisible.value = true;
      break;
    case "check":
      await checkInstructionFiles(row.id);
      break;
    case "remove":
      await confirmRemove(row);
      break;
  }
}

async function checkInstructionFiles(projectId: string) {
  try {
    const result = await projectApi.checkInstructionFiles(projectId);
    checkResult.value = result as Record<string, any>;
    checkDialogVisible.value = true;
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "检查日志规则失败");
  }
}

function checkStatusTag(status: string): "success" | "warning" | "danger" | "info" {
  if (status === "not_exists") return "warning";
  if (status.startsWith("corrupted")) return "danger";
  if (status.startsWith("same_version")) return "success";
  if (status.startsWith("older_version")) return "warning";
  if (status === "no_block") return "warning";
  return "info";
}

async function confirmRemove(row: ProjectListItem) {
  try {
    await ElMessageBox.confirm(
      `确定从系统中移除「${row.name}」吗？\n\n此操作不会删除磁盘中的项目目录和文件，只从本应用中移除管理记录。`,
      "从系统移除项目",
      {
        confirmButtonText: "从系统移除",
        cancelButtonText: "取消",
        type: "warning",
      },
    );
    await projectStore.removeProject(row.id);
    ElMessage.success("已从系统移除");
  } catch {
    // 用户取消
  }
}

// --- 批量操作 ---

function onSelectionChange(selection: ProjectListItem[]) {
  // 同步 el-table 选择到 store
  projectStore.selectedIds = new Set(selection.map((p) => p.id));
}

function clearBatchSelection() {
  tableRef.value?.clearSelection();
  projectStore.clearSelection();
}

function exitBatchMode() {
  batchMode.value = false;
  clearBatchSelection();
}

async function confirmBatchRemove() {
  if (projectStore.selectedIds.size === 0) return;
  try {
    await ElMessageBox.confirm(
      `确定从系统中移除 ${projectStore.selectedIds.size} 个项目吗？\n\n此操作不会删除磁盘中的项目目录和文件。`,
      "批量移除项目",
      {
        confirmButtonText: "从系统移除",
        cancelButtonText: "取消",
        type: "warning",
      },
    );
    await projectStore.batchRemoveProjects();
    ElMessage.success("已批量移除");
    exitBatchMode();
  } catch {
    // 用户取消
  }
}

function openAddDialog() {
  addDialogVisible.value = true;
}

function onProjectAdded() {
  ElMessage.success("项目添加成功");
}

async function doRename() {
  if (!renameValue.value.trim()) {
    ElMessage.warning("项目名称不能为空");
    return;
  }
  try {
    await projectStore.renameProject(renameTargetId.value, renameValue.value.trim());
    renameDialogVisible.value = false;
    ElMessage.success("已重命名");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "重命名失败");
  }
}

// 当数据变化时，如果在批量模式，需要恢复选中状态
watch(() => projectStore.filteredProjects, () => {
  if (batchMode.value && projectStore.selectedIds.size > 0) {
    setTimeout(() => {
      projectStore.filteredProjects.forEach((p) => {
        if (projectStore.selectedIds.has(p.id)) {
          tableRef.value?.toggleRowSelection(p, true);
        }
      });
    }, 0);
  }
});

watch(activeRouteFilter, (filter) => {
  projectStore.filterMode = filter;
}, { immediate: true });

onMounted(() => undefined);
</script>

<style scoped>
.project-list-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.row-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.batch-bar {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.check-result {
  padding: 8px 0;
}

.empty-add-button :deep(.el-icon) {
  width: 16px;
  height: 16px;
  margin: 0 6px 0 0;
  font-size: 16px;
  flex: 0 0 16px;
}
</style>
