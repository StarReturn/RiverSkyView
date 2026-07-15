<template>
  <AppLayout>
    <template #title>设置</template>

    <div class="settings-view">
      <el-tabs v-model="activeCategory" tab-position="left">
        <!-- 常规 -->
        <el-tab-pane label="常规" name="general">
          <el-form label-width="200px" class="settings-form">
            <el-form-item label="主题">
              <el-select v-model="theme" @change="updateSetting('theme', theme)" style="width: 200px">
                <el-option label="跟随系统" value="system" />
                <el-option label="亮色" value="light" />
                <el-option label="暗色" value="dark" />
              </el-select>
            </el-form-item>
            <el-form-item label="开机启动">
              <el-switch v-model="launchAtLogin" @change="updateSetting('launch_at_login', launchAtLogin)" />
            </el-form-item>
            <el-form-item label="最小化到托盘">
              <el-switch v-model="minimizeToTray" @change="updateSetting('minimize_to_tray', minimizeToTray)" />
            </el-form-item>
            <el-form-item label="全局唤起快捷键">
              <el-input
                v-model="globalShortcut"
                @change="updateSetting('global_shortcut', globalShortcut || '')"
                style="width: 200px"
                placeholder="Ctrl+Alt+P"
              />
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 启动与终端 -->
        <el-tab-pane label="启动与终端" name="terminal">
          <el-form label-width="200px" class="settings-form">
            <el-form-item label="默认 Codex 动作">
              <el-select
                v-model="defaultCodexAction"
                @change="updateSetting('default_codex_action', defaultCodexAction)"
                style="width: 200px"
              >
                <el-option label="新建会话" value="new" />
                <el-option label="恢复会话..." value="resume_picker" />
                <el-option label="继续最近会话" value="continue_latest" />
              </el-select>
            </el-form-item>
            <el-form-item label="默认 Claude 动作">
              <el-select
                v-model="defaultClaudeAction"
                @change="updateSetting('default_claude_action', defaultClaudeAction)"
                style="width: 200px"
              >
                <el-option label="新建会话" value="new" />
                <el-option label="恢复会话..." value="resume_picker" />
                <el-option label="继续最近会话" value="continue_latest" />
              </el-select>
            </el-form-item>
            <el-form-item label="工具可用性">
              <div class="tools-list">
                <div v-for="tool in appStore.toolsAvailability" :key="tool.tool_kind" class="tool-item">
                  <el-tag :type="tool.available ? 'success' : 'danger'" size="small">
                    {{ tool.tool_kind }}
                  </el-tag>
                  <span>{{ tool.available ? '已检测' : '未检测到' }}</span>
                  <span v-if="tool.version" class="mono">{{ tool.version }}</span>
                </div>
              </div>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="编辑器与 IDE" name="editors">
          <EditorSettingsPanel />
        </el-tab-pane>

        <!-- 文件与预览 -->
        <el-tab-pane label="文件与预览" name="files">
          <el-form label-width="200px" class="settings-form">
            <el-form-item label="显示隐藏文件">
              <el-switch v-model="showHiddenFiles" @change="updateSetting('show_hidden_files', showHiddenFiles)" />
            </el-form-item>
            <el-form-item label="Markdown 最大预览大小">
              <el-input-number
                v-model="markdownMaxSize"
                :min="1"
                :max="50"
                @change="updateSetting('markdown_max_size_mb', markdownMaxSize)"
              />
              <span style="margin-left: 8px">MB</span>
            </el-form-item>
            <el-form-item label="图片最大预览大小">
              <el-input-number
                v-model="imageMaxSize"
                :min="1"
                :max="100"
                @change="updateSetting('image_max_size_mb', imageMaxSize)"
              />
              <span style="margin-left: 8px">MB</span>
            </el-form-item>
            <el-form-item label="允许加载远程资源">
              <el-switch v-model="allowRemoteResources" @change="updateSetting('allow_remote_resources', allowRemoteResources)" />
              <span style="margin-left: 8px; color: var(--text-secondary); font-size: 12px">默认关闭，建议保持关闭</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 安全与剪贴板 -->
        <el-tab-pane label="安全与剪贴板" name="security">
          <el-form label-width="200px" class="settings-form">
            <el-form-item label="剪贴板自动清理时间">
              <el-input-number
                v-model="clipboardClearSeconds"
                :min="0"
                :max="300"
                @change="updateSetting('clipboard_clear_seconds', clipboardClearSeconds)"
              />
              <span style="margin-left: 8px">秒</span>
            </el-form-item>
            <el-form-item label="资料显示自动遮罩时间">
              <el-input-number
                v-model="vaultAutoMaskSeconds"
                :min="0"
                :max="600"
                @change="updateSetting('vault_auto_mask_seconds', vaultAutoMaskSeconds)"
              />
              <span style="margin-left: 8px">秒</span>
            </el-form-item>
            <el-form-item label="DPAPI 和备份限制">
              <el-alert type="warning" :closable="false" show-icon>
                <p>服务器资料使用当前 Windows 用户加密保护。</p>
                <p>将应用数据文件复制到其他电脑或其他 Windows 用户下，通常无法直接解密。</p>
                <p>第一版备份只覆盖数据库和非敏感配置；加密资料导出/迁移功能不在本期范围内。</p>
              </el-alert>
            </el-form-item>
            <el-form-item label="应用数据目录">
              <div>
                <code class="mono">{{ settingsStore.dataDir }}</code>
                <el-button size="small" style="margin-left: 8px" @click="openDataDir">打开</el-button>
              </div>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 关于 -->
        <el-tab-pane label="关于" name="about">
          <div class="about-section">
            <img src="/logo.png" alt="江天一览图标" style="width: 96px; height: 96px; border-radius: 18px" @error="(e) => (e.target as HTMLImageElement).style.display = 'none'" />
            <h2>江天一览</h2>
            <p class="brand-en">RiverSkyView</p>
            <p>全域工程可视化工作台</p>
            <p>版本: 1.1.0</p>
            <p>技术: Tauri 2 + Vue 3 + Rust + SQLite + DPAPI</p>
            <el-divider />
            <el-button @click="viewLogs">查看应用日志目录</el-button>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";
import { ElMessage } from "element-plus";
import AppLayout from "@/layouts/AppLayout.vue";
import { useSettingsStore } from "@/stores/settings";
import { useAppStore } from "@/stores/app";
import { settingsApi } from "@/api/settings";
import EditorSettingsPanel from "@/components/editor/EditorSettingsPanel.vue";

const settingsStore = useSettingsStore();
const appStore = useAppStore();
const route = useRoute();

const activeCategory = ref("general");

const theme = ref("system");
const launchAtLogin = ref(false);
const minimizeToTray = ref(true);
const globalShortcut = ref("Ctrl+Alt+P");
const defaultCodexAction = ref("resume_picker");
const defaultClaudeAction = ref("resume_picker");
const showHiddenFiles = ref(false);
const markdownMaxSize = ref(2);
const imageMaxSize = ref(20);
const allowRemoteResources = ref(false);
const clipboardClearSeconds = ref(30);
const vaultAutoMaskSeconds = ref(30);

async function updateSetting(key: string, value: string | boolean | number) {
  try {
    await settingsStore.update(key as keyof typeof settingsStore.settings, value);
    ElMessage.success({ message: "已保存", duration: 1500 });
  } catch (e) {
    const err = e as { message?: string; code?: string };
    if (err.code === "SETTINGS_INVALID_VALUE") {
      ElMessage.error(err.message || "设置值无效");
    } else {
      ElMessage.error(err.message || "保存设置失败");
    }
  }
}

async function openDataDir() {
  try {
    await settingsApi.openDataDir();
  } catch {
    ElMessage.error("打开数据目录失败");
  }
}

async function viewLogs() {
  try {
    // 日志目录在数据目录下的 logs 子目录
    await settingsApi.openDataDir();
  } catch {
    ElMessage.error("打开日志目录失败");
  }
}

onMounted(() => {
  if (route.query.tab === "editors") activeCategory.value = "editors";
  const s = settingsStore.settings;
  theme.value = s.theme;
  launchAtLogin.value = s.launch_at_login;
  minimizeToTray.value = s.minimize_to_tray;
  globalShortcut.value = s.global_shortcut || "";
  defaultCodexAction.value = s.default_codex_action;
  defaultClaudeAction.value = s.default_claude_action;
  showHiddenFiles.value = s.show_hidden_files;
  markdownMaxSize.value = s.markdown_max_size_mb;
  imageMaxSize.value = s.image_max_size_mb;
  allowRemoteResources.value = s.allow_remote_resources;
  clipboardClearSeconds.value = s.clipboard_clear_seconds;
  vaultAutoMaskSeconds.value = s.vault_auto_mask_seconds;
});
</script>

<style scoped>
.settings-view {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 24px;
  min-height: 500px;
}

.settings-form {
  max-width: 600px;
}

.tools-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.about-section {
  text-align: center;
  padding: 32px;
}

.about-section h2 {
  margin: 16px 0 8px;
}

.about-section p {
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.about-section .brand-en {
  color: var(--primary);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.4px;
}
</style>
