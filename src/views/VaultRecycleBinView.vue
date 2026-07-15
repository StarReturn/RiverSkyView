<template>
  <AppLayout>
    <template #title>资料回收站</template>
    <template #actions>
      <el-button @click="$router.push('/vault')">返回资料库</el-button>
    </template>

    <el-table :data="vaultStore.removedEntries" v-loading="loading" empty-text=" ">
      <el-table-column label="名称" min-width="180">
        <template #default="{ row }">
          <span style="font-weight: 500">{{ row.name }}</span>
        </template>
      </el-table-column>
      <el-table-column label="标签" min-width="150">
        <template #default="{ row }">
          <el-tag v-for="tag in row.tags" :key="tag" size="small" style="margin-right: 4px">
            {{ tag }}
          </el-tag>
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
          <el-button size="small" type="danger" @click="permanentDelete(row)">永久清除</el-button>
        </template>
      </el-table-column>

      <template #empty>
        <div class="empty-state">
          <el-icon><Delete /></el-icon>
          <h3>回收站为空</h3>
          <p>没有已移除的资料</p>
          <el-button type="primary" @click="$router.push('/vault')" style="margin-top: 16px">
            返回资料库
          </el-button>
        </div>
      </template>
    </el-table>

    <el-alert
      title="永久清除将删除应用保存的密文，无法恢复。SSD 和日志型文件系统无法保证物理扇区级安全擦除。"
      type="warning"
      :closable="false"
      show-icon
      style="margin-top: 16px"
    />
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { Delete } from "@element-plus/icons-vue";
import AppLayout from "@/layouts/AppLayout.vue";
import { useVaultStore } from "@/stores/vault";
import { formatDateTime } from "@/utils/format";
import type { VaultListItem } from "@/types";

const vaultStore = useVaultStore();
const loading = ref(false);

async function restore(id: string) {
  try {
    await vaultStore.restoreEntry(id);
    ElMessage.success("资料已恢复");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "恢复失败");
  }
}

async function permanentDelete(entry: VaultListItem) {
  try {
    const { value } = await ElMessageBox.prompt(
      `永久清除将删除「${entry.name}」的密文，无法恢复。\n\n请输入资料名称以确认：`,
      "永久清除服务器资料",
      {
        confirmButtonText: "永久清除",
        cancelButtonText: "取消",
        inputPlaceholder: "输入资料名称确认",
        inputValidator: (val) => val === entry.name || "名称不匹配",
        type: "error",
      },
    );

    if (value === entry.name) {
      await vaultStore.permanentDelete(entry.id);
      ElMessage.success("已永久清除");
    }
  } catch {
    // 用户取消
  }
}

onMounted(async () => {
  loading.value = true;
  await vaultStore.loadRemovedEntries();
  loading.value = false;
});
</script>
