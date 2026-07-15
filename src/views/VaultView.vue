<template>
  <AppLayout>
    <template #title>服务器资料库</template>
    <template #actions>
      <el-button :icon="Upload" @click="importDialogVisible = true">导入 TXT</el-button>
      <el-button type="primary" :icon="Plus" @click="newDialogVisible = true">新建</el-button>
    </template>

    <div class="vault-view">
      <!-- 列表面板 -->
      <div class="vault-list-panel">
        <el-input
          v-model="vaultStore.searchQuery"
          placeholder="搜索名称或标签..."
          :prefix-icon="Search"
          clearable
          size="small"
          style="margin-bottom: 8px"
          @input="debouncedLoad"
        />
        <div class="vault-list">
          <div
            v-for="entry in vaultStore.entries"
            :key="entry.id"
            class="vault-item"
            :class="{ selected: vaultStore.selectedId === entry.id }"
            @click="vaultStore.selectEntry(entry.id)"
          >
            <div class="item-name">{{ entry.name }}</div>
            <div class="item-tags">
              <el-tag v-for="tag in entry.tags" :key="tag" size="small" style="margin-right: 4px">
                {{ tag }}
              </el-tag>
            </div>
            <div class="item-time">{{ formatDateTime(entry.updated_at) }}</div>
          </div>
          <div v-if="vaultStore.entries.length === 0" class="empty-state">
            <el-icon><DataLine /></el-icon>
            <h3>尚未保存服务器资料</h3>
            <p>新建资料或导入 TXT 文件</p>
          </div>
        </div>
      </div>

      <!-- 详情面板 -->
      <div class="vault-detail-panel">
        <template v-if="vaultStore.selectedId">
          <!-- 查看模式 -->
          <div v-if="!editing" class="detail-view">
            <div class="detail-header">
              <div>
                <span class="detail-name">{{ selectedEntry?.name }}</span>
                <div class="detail-tags">
                  <el-tag v-for="tag in selectedEntry?.tags" :key="tag" size="small" style="margin-right: 4px">
                    {{ tag }}
                  </el-tag>
                </div>
                <div class="detail-time">最后更新: {{ formatDateTime(selectedEntry?.updated_at ?? null) }}</div>
              </div>
              <div class="detail-actions">
                <el-button size="small" @click="startEdit">编辑</el-button>
                <el-dropdown trigger="click" @command="handleDetailAction">
                  <el-button size="small">⋯</el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item command="remove">移至资料回收站</el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>
            </div>

            <!-- 遮罩内容 -->
            <div class="detail-content">
              <div v-if="!vaultStore.contentRevealed" class="vault-masked">
                <div v-for="i in 5" :key="i" class="mask-line" :style="{ width: 60 + Math.random() * 30 + '%' }"></div>
                <div style="margin-top: 24px; display: flex; gap: 8px">
                  <el-button
                    aria-label="按住显示服务器资料"
                    @pointerdown.prevent="beginTemporaryReveal"
                    @pointerup.prevent="endTemporaryReveal"
                    @pointercancel="endTemporaryReveal"
                    @pointerleave="endTemporaryReveal"
                    @keydown.space.prevent="beginTemporaryReveal"
                    @keyup.space.prevent="endTemporaryReveal"
                    @keydown.enter.prevent="beginTemporaryReveal"
                    @keyup.enter.prevent="endTemporaryReveal"
                    @blur="endTemporaryReveal"
                  >
                    按住显示
                  </el-button>
                  <el-button type="primary" @click="revealContent">显示内容</el-button>
                </div>
                <p style="margin-top: 16px; color: var(--text-secondary); font-size: 12px">
                  此内容使用 Windows 当前用户加密保护。
                </p>
              </div>

              <!-- 已显示内容 -->
              <div v-else>
                <pre class="vault-content">{{ vaultStore.selectedContent?.content }}</pre>
                <div style="margin-top: 12px">
                  <el-button @click="copyContent">复制内容</el-button>
                  <el-button @click="maskContent">隐藏</el-button>
                </div>
              </div>
            </div>
          </div>

          <!-- 编辑模式 -->
          <div v-else class="detail-edit">
            <el-form label-width="60px">
              <el-form-item label="名称">
                <el-input v-model="editName" />
              </el-form-item>
              <el-form-item label="标签">
                <el-input v-model="editTagsStr" placeholder="逗号分隔的标签" />
              </el-form-item>
              <el-form-item label="正文">
                <el-input
                  v-model="editContent"
                  type="textarea"
                  :rows="15"
                  class="mono"
                />
              </el-form-item>
            </el-form>
            <div class="edit-actions">
              <el-button @click="cancelEdit">取消</el-button>
              <el-button type="primary" :loading="saving" @click="saveEdit">保存</el-button>
            </div>
          </div>
        </template>

        <div v-else class="empty-state">
          <el-icon><DataLine /></el-icon>
          <p>选择资料查看详情</p>
        </div>
      </div>
    </div>

    <!-- DPAPI 限制说明 -->
    <el-alert
      v-if="showDpapiNotice"
      title="服务器资料使用当前 Windows 用户加密保护"
      type="info"
      show-icon
      closable
      @close="dismissDpapiNotice"
    >
      <p>将应用数据文件复制到其他电脑或其他 Windows 用户下，通常无法直接解密。</p>
      <p>请不要把本功能视为跨设备密码备份工具。</p>
    </el-alert>

    <!-- 新建对话框 -->
    <el-dialog v-model="newDialogVisible" title="新建资料" width="560px">
      <el-form label-width="60px">
        <el-form-item label="名称">
          <el-input v-model="newName" />
        </el-form-item>
        <el-form-item label="标签">
          <el-input v-model="newTagsStr" placeholder="逗号分隔的标签" />
        </el-form-item>
        <el-form-item label="正文">
          <el-input v-model="newContent" type="textarea" :rows="10" class="mono" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="newDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="createEntry">创建</el-button>
      </template>
    </el-dialog>

    <!-- 导入对话框 -->
    <el-dialog v-model="importDialogVisible" title="导入 TXT" width="480px">
      <el-form label-width="60px">
        <el-form-item label="名称">
          <el-input v-model="importName" placeholder="留空使用文件名" />
        </el-form-item>
        <el-form-item label="文件">
          <el-input v-model="importPath" readonly placeholder="选择 TXT 文件...">
            <template #append>
              <el-button @click="selectFile">选择</el-button>
            </template>
          </el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="importDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="doImport">导入</el-button>
      </template>
    </el-dialog>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { Plus, Upload, Search, DataLine } from "@element-plus/icons-vue";
import { open } from "@tauri-apps/plugin-dialog";
import AppLayout from "@/layouts/AppLayout.vue";
import { useVaultStore } from "@/stores/vault";
import { useSettingsStore } from "@/stores/settings";
import { vaultApi } from "@/api/vault";
import { formatDateTime } from "@/utils/format";
import { ClipboardManager } from "@/utils/clipboard";

const vaultStore = useVaultStore();
const settingsStore = useSettingsStore();

const newDialogVisible = ref(false);
const importDialogVisible = ref(false);
const showDpapiNotice = ref(true);
const editing = ref(false);
const saving = ref(false);
const temporaryRevealActive = ref(false);
let revealGeneration = 0;

const newName = ref("");
const newTagsStr = ref("");
const newContent = ref("");

const importName = ref("");
const importPath = ref("");

const editName = ref("");
const editTagsStr = ref("");
const editContent = ref("");

const selectedEntry = computed(() =>
  vaultStore.entries.find((e) => e.id === vaultStore.selectedId),
);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedLoad() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => vaultStore.loadEntries(), 300);
}

async function revealContent() {
  temporaryRevealActive.value = false;
  revealGeneration += 1;
  if (vaultStore.selectedId) {
    await vaultStore.revealContent(vaultStore.selectedId);
  }
}

function maskContent() {
  temporaryRevealActive.value = false;
  revealGeneration += 1;
  vaultStore.maskContent();
}

async function beginTemporaryReveal() {
  if (!vaultStore.selectedId || temporaryRevealActive.value) return;
  temporaryRevealActive.value = true;
  const generation = ++revealGeneration;
  await vaultStore.revealContent(vaultStore.selectedId);
  if (!temporaryRevealActive.value || generation !== revealGeneration) {
    vaultStore.maskContent();
  }
}

function endTemporaryReveal() {
  if (!temporaryRevealActive.value) return;
  temporaryRevealActive.value = false;
  revealGeneration += 1;
  vaultStore.maskContent();
}

function handleWindowKeyUp(event: KeyboardEvent) {
  if (event.key === " " || event.key === "Enter") endTemporaryReveal();
}

async function copyContent() {
  if (!vaultStore.selectedContent) return;
  await ClipboardManager.write(
    vaultStore.selectedContent.content,
    settingsStore.settings.clipboard_clear_seconds,
  );
  ElMessage.success(`已复制，将在 ${settingsStore.settings.clipboard_clear_seconds} 秒后清理剪贴板`);
}

function startEdit() {
  if (!selectedEntry.value) return;
  editing.value = true;
  editName.value = selectedEntry.value.name;
  editTagsStr.value = selectedEntry.value.tags.join(", ");

  // 临时获取内容用于编辑
  vaultStore.revealContent(selectedEntry.value.id).then(() => {
    editContent.value = vaultStore.selectedContent?.content || "";
  });
}

function cancelEdit() {
  editing.value = false;
  vaultStore.maskContent();
}

async function saveEdit() {
  if (!vaultStore.selectedId) return;
  saving.value = true;
  try {
    await vaultStore.updateEntry({
      id: vaultStore.selectedId,
      name: editName.value,
      content: editContent.value,
      tags: editTagsStr.value.split(",").map((t) => t.trim()).filter(Boolean),
    });
    editing.value = false;
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "保存失败");
  } finally {
    saving.value = false;
  }
}

async function createEntry() {
  try {
    await vaultStore.createEntry({
      name: newName.value,
      content: newContent.value,
      tags: newTagsStr.value.split(",").map((t) => t.trim()).filter(Boolean),
    });
    newDialogVisible.value = false;
    newName.value = "";
    newTagsStr.value = "";
    newContent.value = "";
    ElMessage.success("已创建");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "创建失败");
  }
}

async function selectFile() {
  const selected = await open({
    filters: [{ name: "文本文件", extensions: ["txt"] }],
    multiple: false,
  });
  if (selected) {
    importPath.value = selected as string;
  }
}

async function doImport() {
  if (!importPath.value) {
    ElMessage.warning("请选择文件");
    return;
  }
  try {
    await vaultApi.importTxt({
      name: importName.value,
      source_file_path: importPath.value,
    });
    await vaultStore.loadEntries();
    importDialogVisible.value = false;
    importName.value = "";
    importPath.value = "";
    ElMessage.success("已导入");
  } catch (e) {
    ElMessage.error((e as { message?: string })?.message || "导入失败");
  }
}

async function handleDetailAction(cmd: string) {
  if (cmd === "remove" && vaultStore.selectedId) {
    try {
      await ElMessageBox.confirm("确定移至资料回收站？", "移至回收站", {
        confirmButtonText: "移至回收站",
        cancelButtonText: "取消",
        type: "warning",
      });
      await vaultStore.removeEntry(vaultStore.selectedId);
      ElMessage.success("已移至回收站");
    } catch {
      // 用户取消
    }
  }
}

function dismissDpapiNotice() {
  showDpapiNotice.value = false;
}

watch(() => vaultStore.selectedId, () => endTemporaryReveal());

onMounted(() => {
  vaultStore.loadEntries();
  window.addEventListener("pointerup", endTemporaryReveal);
  window.addEventListener("pointercancel", endTemporaryReveal);
  window.addEventListener("blur", endTemporaryReveal);
  window.addEventListener("keyup", handleWindowKeyUp);
});

onBeforeUnmount(() => {
  window.removeEventListener("pointerup", endTemporaryReveal);
  window.removeEventListener("pointercancel", endTemporaryReveal);
  window.removeEventListener("blur", endTemporaryReveal);
  window.removeEventListener("keyup", handleWindowKeyUp);
  maskContent();
});
</script>

<style scoped>
.vault-view {
  display: flex;
  gap: 1px;
  height: calc(100vh - 200px);
  min-height: 400px;
  background: var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.vault-list-panel {
  width: 320px;
  background: var(--bg-surface);
  padding: 8px;
  display: flex;
  flex-direction: column;
}

.vault-list {
  flex: 1;
  overflow: auto;
}

.vault-item {
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
}

.vault-item:hover {
  background: var(--bg-subtle);
}

.vault-item.selected {
  background: rgba(37, 99, 235, 0.1);
}

.item-name {
  font-weight: 500;
  margin-bottom: 4px;
}

.item-tags {
  margin-bottom: 4px;
}

.item-time {
  font-size: 12px;
  color: var(--text-secondary);
}

.vault-detail-panel {
  flex: 1;
  background: var(--bg-surface);
  padding: 16px;
  overflow: auto;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.detail-name {
  font-size: 18px;
  font-weight: 600;
}

.detail-tags {
  margin-top: 4px;
}

.detail-time {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.vault-masked {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px;
}

.mask-line {
  background: var(--text-secondary);
  height: 14px;
  border-radius: 4px;
  margin: 4px 0;
  opacity: 0.3;
}

.vault-content {
  font-family: "Cascadia Mono", "Consolas", monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  padding: 12px;
  background: var(--bg-subtle);
  border-radius: 6px;
  min-height: 200px;
}

.edit-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
}
</style>
