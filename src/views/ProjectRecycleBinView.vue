<template>
  <AppLayout>
    <template #title>项目回收站</template>
    <template #actions>
      <el-button @click="$router.push('/')">返回项目列表</el-button>
    </template>

    <el-table :data="projectStore.removedProjects" v-loading="loading" empty-text=" ">
      <el-table-column label="项目名称" min-width="180">
        <template #default="{ row }">
          <span style="font-weight: 500">{{ row.name }}</span>
        </template>
      </el-table-column>
      <el-table-column label="原路径" min-width="300">
        <template #default="{ row }">
          <span class="path-text" :title="row.path">{{ truncatePath(row.path, 60) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="移除时间" width="180">
        <template #default="{ row }">
          {{ formatDateTime(row.removed_at) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button size="small" type="primary" @click="restore(row.id)">恢复</el-button>
          <el-button
            v-if="row.directory_available"
            size="small"
            @click="openExplorer(row.id)"
          >
            在资源管理器中打开
          </el-button>
        </template>
      </el-table-column>

      <template #empty>
        <div class="empty-state">
          <el-icon><Delete /></el-icon>
          <h3>回收站为空</h3>
          <p>没有已移除的项目</p>
          <el-button type="primary" @click="$router.push('/')" style="margin-top: 16px">
            返回项目列表
          </el-button>
        </div>
      </template>
    </el-table>

    <el-alert
      title="项目回收站不会删除磁盘中的项目目录和文件。恢复项目可将其重新纳入管理。"
      type="info"
      :closable="false"
      show-icon
      style="margin-top: 16px"
    />
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import { Delete } from "@element-plus/icons-vue";
import AppLayout from "@/layouts/AppLayout.vue";
import { useProjectStore } from "@/stores/project";
import { filesApi } from "@/api/files";
import { formatDateTime, truncatePath } from "@/utils/format";

const projectStore = useProjectStore();
const loading = ref(false);

async function restore(id: string) {
  try {
    await projectStore.restoreProject(id);
    ElMessage.success("项目已恢复");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "恢复失败");
  }
}

async function openExplorer(projectId: string) {
  try {
    await filesApi.openInExplorer(projectId);
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "打开资源管理器失败");
  }
}

onMounted(async () => {
  loading.value = true;
  await projectStore.loadRemovedProjects();
  loading.value = false;
});
</script>
