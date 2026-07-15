<template>
  <div class="activity-workspace">
    <!-- 活动摘要 -->
    <div class="activity-summary" v-if="activityStore.activitySummary">
      <div class="summary-stats">
        <span>过去一年</span>
        <span><strong>{{ activityStore.activitySummary.total_tasks }}</strong> 个任务</span>
        <span class="stat-completed"><strong>{{ activityStore.activitySummary.completed }}</strong> 完成</span>
        <span class="stat-failed"><strong>{{ activityStore.activitySummary.failed }}</strong> 失败</span>
        <span class="stat-blocked"><strong>{{ activityStore.activitySummary.blocked }}</strong> 阻塞</span>
      </div>
      <div class="last-sync">
        上次同步: {{ activityStore.lastSyncResult ? formatDateTime(activityStore.lastSyncResult.last_synced_at) : '未同步' }}
      </div>
    </div>

    <!-- 热力图 -->
    <div class="heatmap-container" ref="heatmapContainer"></div>

    <!-- 筛选栏 -->
    <div class="activity-filters">
      <el-select v-model="agentFilter" placeholder="全部智能体" clearable size="small" style="width: 140px">
        <el-option label="全部智能体" value="" />
        <el-option label="Codex" value="codex" />
        <el-option label="Claude" value="claude" />
        <el-option label="其他" value="other" />
      </el-select>
      <el-select v-model="statusFilter" placeholder="全部状态" clearable size="small" style="width: 140px">
        <el-option label="全部状态" value="" />
        <el-option label="完成" value="completed" />
        <el-option label="失败" value="failed" />
        <el-option label="阻塞" value="blocked" />
      </el-select>
      <el-button size="small" :icon="Refresh" :loading="activityStore.syncing" @click="syncLogs">
        刷新日志
      </el-button>
      <el-tag v-if="activityStore.filterDate" closable @close="clearDateFilter" size="small">
        {{ activityStore.filterDate }}
      </el-tag>
    </div>

    <!-- 时间线 + 日志详情 -->
    <div class="timeline-detail">
      <div class="timeline-panel">
        <div class="panel-title">时间线</div>
        <div class="timeline-body">
          <div
            v-for="log in activityStore.logs"
            :key="log.id"
            class="timeline-item"
            :class="{ selected: activityStore.selectedLog?.id === log.id }"
            @click="selectLog(log)"
          >
            <span class="status-dot" :class="log.status"></span>
            <span class="time">{{ formatTime(log.finished_at) }}</span>
            <span class="agent">{{ log.agent }}</span>
            <span class="title">{{ log.title || log.relative_path }}</span>
            <el-tag v-if="log.parse_status === 'invalid'" type="warning" size="small">格式异常</el-tag>
            <el-tag v-if="log.time_inferred" type="info" size="small">推断时间</el-tag>
          </div>
          <div v-if="activityStore.logs.length === 0" class="empty-state">
            <p>暂无项目日志</p>
          </div>
        </div>
      </div>

      <div class="log-detail-panel">
        <div class="panel-title">
          {{ activityStore.selectedLog ? (activityStore.selectedLog.title || '日志详情') : '日志详情' }}
        </div>
        <div class="log-detail-body">
          <div v-if="activityStore.logContent" class="markdown-preview" v-html="renderedLogContent">
          </div>
          <div v-else class="empty-state">
            <p>选择日志查看详情</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from "vue";
import * as echarts from "echarts";
import { Refresh } from "@element-plus/icons-vue";
import { useActivityStore } from "@/stores/activity";
import { renderMarkdown } from "@/utils/sanitize";
import { formatDateTime } from "@/utils/format";
import { buildYearHeatmapData, toLocalDateString } from "@/utils/heatmap";
import type { ProjectLog } from "@/types";

const props = defineProps<{ projectId: string }>();
const activityStore = useActivityStore();

const heatmapContainer = ref<HTMLElement | null>(null);
let chart: echarts.ECharts | null = null;
let resizeObserver: ResizeObserver | null = null;

const agentFilter = ref("");
const statusFilter = ref("");

const renderedLogContent = computed(() => {
  if (!activityStore.logContent) return "";
  return renderMarkdown(activityStore.logContent);
});

async function syncLogs() {
  await activityStore.syncLogs(props.projectId);
  await nextTick();
  renderHeatmap();
}

async function selectLog(log: ProjectLog) {
  await activityStore.selectLog(props.projectId, log);
}

function clearDateFilter() {
  activityStore.setFilter(undefined, undefined, null);
  activityStore.loadLogs(props.projectId);
}

watch(agentFilter, (val) => {
  activityStore.setFilter(val || null);
  activityStore.loadLogs(props.projectId);
});

watch(statusFilter, (val) => {
  activityStore.setFilter(undefined, val || null);
  activityStore.loadLogs(props.projectId);
});

function formatTime(timeStr: string): string {
  const d = new Date(timeStr);
  if (isNaN(d.getTime())) return timeStr;
  const today = new Date();
  if (d.toDateString() === today.toDateString()) {
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  }
  return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

function renderHeatmap() {
  if (!heatmapContainer.value || !activityStore.activitySummary) return;

  const width = heatmapContainer.value.clientWidth;
  if (width < 320 || heatmapContainer.value.clientHeight < 120) return;

  if (!chart) {
    chart = echarts.init(heatmapContainer.value);
  }

  const heatmapData = activityStore.activitySummary.heatmap;
  const { today, rangeStart, data } = buildYearHeatmapData(heatmapData);

  const maxCount = Math.max(...heatmapData.map((c) => c.count), 1);
  const cellSize = Math.max(10, Math.min(16, Math.floor((width - 100) / 53) - 2));
  const styles = getComputedStyle(heatmapContainer.value);
  const surface = styles.getPropertyValue("--bg-surface").trim() || "#fff";
  const secondary = styles.getPropertyValue("--text-secondary").trim() || "#64748b";

  chart.setOption({
    tooltip: {
      formatter: (params: { data: [string, number] }) => {
        const [date, count] = params.data;
        return `${date}<br/>${count} 个任务`;
      },
    },
    visualMap: {
      min: 0,
      max: maxCount,
      calculable: false,
      orient: "horizontal",
      left: 44,
      bottom: 8,
      itemWidth: 14,
      itemHeight: 10,
      textStyle: { color: secondary },
      inRange: { color: ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"] },
    },
    calendar: {
      top: 38,
      left: 52,
      right: 20,
      bottom: 52,
      cellSize: [cellSize, cellSize],
      range: [toLocalDateString(rangeStart), toLocalDateString(today)],
      itemStyle: { borderWidth: 2, borderColor: surface },
      yearLabel: { show: false },
      dayLabel: { firstDay: 1, nameMap: "cn", color: secondary },
      monthLabel: { nameMap: "cn", color: secondary },
      splitLine: { show: false },
    },
    series: [{
      type: "heatmap",
      coordinateSystem: "calendar",
      data,
    }],
  }, true);
  chart.resize();
}

onMounted(async () => {
  await activityStore.syncLogs(props.projectId);
  await nextTick();
  renderHeatmap();
  if (heatmapContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      window.requestAnimationFrame(renderHeatmap);
    });
    resizeObserver.observe(heatmapContainer.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  chart?.dispose();
  chart = null;
});
</script>

<style scoped>
.activity-workspace {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.activity-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.summary-stats {
  display: flex;
  gap: 16px;
  font-size: 14px;
}

.summary-stats strong {
  font-weight: 600;
}

.stat-completed { color: var(--success); }
.stat-failed { color: var(--danger); }
.stat-blocked { color: var(--warning); }

.last-sync {
  font-size: 12px;
  color: var(--text-secondary);
}

.heatmap-container {
  width: 100%;
  height: 230px;
  min-height: 230px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.activity-filters {
  display: flex;
  gap: 8px;
  align-items: center;
}

.timeline-detail {
  display: flex;
  gap: 1px;
  height: 400px;
  background: var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.timeline-panel {
  width: 40%;
  background: var(--bg-surface);
  display: flex;
  flex-direction: column;
}

.log-detail-panel {
  flex: 1;
  background: var(--bg-surface);
  display: flex;
  flex-direction: column;
}

.panel-title {
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
}

.timeline-body, .log-detail-body {
  flex: 1;
  overflow: auto;
}

.timeline-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--border);
}

.timeline-item:hover {
  background: var(--bg-subtle);
}

.timeline-item.selected {
  background: rgba(37, 99, 235, 0.1);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  min-width: 8px;
}

.status-dot.completed { background: var(--success); }
.status-dot.failed { background: var(--danger); }
.status-dot.blocked { background: var(--warning); }

.timeline-item .time {
  font-family: "Cascadia Mono", "Consolas", monospace;
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 50px;
}

.timeline-item .agent {
  font-size: 12px;
  color: var(--primary);
  min-width: 50px;
}

.timeline-item .title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
