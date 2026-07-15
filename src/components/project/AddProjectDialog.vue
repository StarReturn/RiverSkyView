<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title="添加项目"
    width="640px"
    :close-on-click-modal="false"
  >
    <el-steps :active="step" finish-status="success" simple>
      <el-step title="选择目录" />
      <el-step title="项目信息" />
      <el-step title="确认变更" />
    </el-steps>

    <!-- 步骤 1：选择目录 -->
    <div v-if="step === 0" class="step-content">
      <el-input
        v-model="selectedPath"
        placeholder="选择项目目录..."
        readonly
      >
        <template #append>
          <el-button :icon="FolderOpened" @click="selectDirectory">选择目录</el-button>
        </template>
      </el-input>

      <div v-if="pathError" class="path-error">
        <el-alert :title="pathError" type="error" :closable="false" />
      </div>

      <div v-if="selectedPath" class="path-info">
        <p>路径已选择： <code>{{ selectedPath }}</code></p>
      </div>
    </div>

    <!-- 步骤 2：项目信息 -->
    <div v-if="step === 1" class="step-content">
      <el-form label-width="80px">
        <el-form-item label="项目名称">
          <el-input v-model="projectName" placeholder="项目名称" />
        </el-form-item>
        <el-form-item label="收藏">
          <el-switch v-model="isFavorite" />
        </el-form-item>
      </el-form>
    </div>

    <!-- 步骤 3：确认变更 -->
    <div v-if="step === 2" class="step-content">
      <p>将向项目目录写入以下管理文件：</p>
      <ul class="file-list">
        <li>✓ 创建或更新 AGENTS.md</li>
        <li>✓ 创建或更新 CLAUDE.md</li>
        <li>✓ 创建 pm_log 目录</li>
        <li>✓ 创建 .project-manager.json</li>
      </ul>
      <el-alert
        title="已有文件不会被覆盖，只更新带 project-manager 标记的管理块。"
        type="info"
        :closable="false"
      />
    </div>

    <template #footer>
      <el-button v-if="step > 0" @click="step--">上一步</el-button>
      <el-button v-if="step < 2" type="primary" :disabled="!canProceed" @click="nextStep">
        下一步
      </el-button>
      <el-button
        v-if="step === 2"
        type="primary"
        :loading="submitting"
        @click="submit"
      >
        确认添加
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { ElMessage } from "element-plus";
import { FolderOpened } from "@element-plus/icons-vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useProjectStore } from "@/stores/project";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  "update:visible": [value: boolean];
  added: [];
}>();

const projectStore = useProjectStore();

const step = ref(0);
const selectedPath = ref("");
const pathError = ref("");
const projectName = ref("");
const isFavorite = ref(false);
const submitting = ref(false);

watch(
  () => props.visible,
  (val) => {
    if (val) {
      step.value = 0;
      selectedPath.value = "";
      pathError.value = "";
      projectName.value = "";
      isFavorite.value = false;
      submitting.value = false;
    }
  },
);

const canProceed = computed(() => {
  if (step.value === 0) return selectedPath.value && !pathError.value;
  if (step.value === 1) return projectName.value.trim().length > 0;
  return true;
});

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择项目目录",
    });
    if (selected) {
      selectedPath.value = selected as string;
      pathError.value = "";

      // 默认使用目录名作为项目名
      const parts = selectedPath.value.replace(/\\/g, "/").split("/").filter(Boolean);
      if (parts.length > 0 && !projectName.value) {
        projectName.value = parts[parts.length - 1];
      }
    }
  } catch {
    pathError.value = "选择目录失败";
  }
}

function nextStep() {
  step.value++;
}

async function submit() {
  submitting.value = true;
  try {
    await projectStore.addProject({
      path: selectedPath.value,
      name: projectName.value.trim(),
      is_favorite: isFavorite.value,
    });
    emit("update:visible", false);
    emit("added");
  } catch (e) {
    const err = e as { message?: string; code?: string };
    if (err.code === "PROJECT_REMOVED_EXISTS") {
      ElMessage.warning("该项目曾已移除，请从回收站恢复");
    } else if (err.code === "PROJECT_ALREADY_EXISTS") {
      ElMessage.warning("该项目已添加");
    } else {
      ElMessage.error(err.message || "添加项目失败");
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped>
.step-content {
  padding: 24px 0;
  min-height: 120px;
}

.path-error {
  margin-top: 12px;
}

.path-info {
  margin-top: 12px;
}

.path-info code {
  font-family: "Cascadia Mono", "Consolas", monospace;
  font-size: 13px;
  color: var(--text-secondary);
  word-break: break-all;
}

.file-list {
  list-style: none;
  padding: 12px 0;
  margin-bottom: 12px;
}

.file-list li {
  padding: 4px 0;
  color: var(--success);
}
</style>
