<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { fetchMetrics } from '@/api/api'
import type { Metrics } from '@/api/model'
import { WorkerState } from '@/api/model'
import { formatBytes, formatDuration } from '@/utils/format'
import LogViewer from '@/components/LogViewer.vue'

const metrics = ref<Metrics | null>(null)
let refreshInterval: ReturnType<typeof setInterval> | null = null

// 刷新间隔选项
const refreshIntervals = [
  { title: '3 秒', value: 3000 },
  { title: '5 秒', value: 5000 },
  { title: '30 秒', value: 30000 }
]
const selectedInterval = ref(3000) // 默认 3 秒

// 获取指标数据
const loadMetrics = async () => {
  try {
    metrics.value = await fetchMetrics()
  } catch (error) {
    console.error('获取系统指标失败:', error)
  }
}

// 计算 Worker 状态统计
const workerStats = computed(() => {
  if (!metrics.value?.scheduler.workers) return []

  const stats = new Map<WorkerState, number>()
  metrics.value.scheduler.workers.forEach(worker => {
    stats.set(worker.state, (stats.get(worker.state) || 0) + 1)
  })

  return [
    { state: WorkerState.Collecting, count: stats.get(WorkerState.Collecting) || 0, color: 'info' },
    { state: WorkerState.Idle, count: stats.get(WorkerState.Idle) || 0, color: 'grey' }
  ]
})

// 重置定时器
const resetInterval = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
  refreshInterval = setInterval(loadMetrics, selectedInterval.value)
}

// 监听刷新间隔变化
watch(selectedInterval, () => {
  resetInterval()
})

// 定时刷新指标数据
onMounted(() => {
  loadMetrics()
  resetInterval()
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<template>
  <v-container fluid class="dashboard pa-6">
    <!-- 顶部标题栏 -->
    <v-row class="mb-6">
      <v-col cols="12" class="d-flex justify-space-between align-center">
        <div class="d-flex align-center">
          <v-icon icon="mdi-view-dashboard" size="32" class="mr-4" color="primary" />
          <h1 class="text-h4 font-weight-medium mb-0">系统仪表盘</h1>
        </div>
        <div class="d-flex align-center">
          <v-icon icon="mdi-refresh" class="mr-2" color="primary" />
          <v-select
            v-model="selectedInterval"
            :items="refreshIntervals"
            item-title="title"
            item-value="value"
            label="刷新间隔"
            variant="outlined"
            hide-details
            density="compact"
            class="refresh-select"
          />
        </div>
      </v-col>
    </v-row>

    <!-- 状态卡片行 -->
    <v-row class="mb-6">
      <!-- 系统资源卡片 -->
      <v-col cols="12" md="4">
        <v-card elevation="2" class="h-100">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon icon="mdi-memory" size="28" color="primary" class="mr-2" />
            </template>
            <v-card-title class="text-h6 font-weight-medium">系统资源</v-card-title>
          </v-card-item>
          <v-divider />
          <v-card-text class="pt-4" v-if="metrics?.process">
            <v-row dense>
              <v-col cols="12" class="mb-3">
                <div class="d-flex align-center justify-space-between">
                  <div class="d-flex align-center">
                    <v-icon icon="mdi-database" color="info" class="mr-3" />
                    <span class="text-subtitle-1">内存使用</span>
                  </div>
                  <span class="text-h6 font-weight-bold">{{ formatBytes(metrics.process.used) }}</span>
                </div>
              </v-col>
              <v-col cols="12">
                <div class="d-flex align-center justify-space-between">
                  <div class="d-flex align-center">
                    <v-icon icon="mdi-clock-outline" color="success" class="mr-3" />
                    <span class="text-subtitle-1">运行时间</span>
                  </div>
                  <span class="text-h6 font-weight-bold">{{ formatDuration(metrics.process.run_time_sec) }}</span>
                </div>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- 下载器状态卡片 -->
      <v-col cols="12" md="4">
        <v-card elevation="2" class="h-100">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon icon="mdi-download" size="28" color="primary" class="mr-2" />
            </template>
            <v-card-title class="text-h6 font-weight-medium">下载器状态</v-card-title>
          </v-card-item>
          <v-divider />
          <v-card-text class="pt-4" v-if="metrics?.downloader">
            <div class="d-flex align-center justify-space-between">
              <div class="d-flex align-center">
                <v-icon icon="mdi-download-network" color="warning" class="mr-3" />
                <span class="text-subtitle-1">当前任务数</span>
              </div>
              <span class="text-h5 font-weight-bold">{{ metrics.downloader.num_of_tasks }}</span>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- Worker 状态卡片 -->
      <v-col cols="12" md="4">
        <v-card elevation="2" class="h-100">
          <v-card-item>
            <template v-slot:prepend>
              <v-icon icon="mdi-cogs" size="28" color="primary" class="mr-2" />
            </template>
            <v-card-title class="text-h6 font-weight-medium">系统组件状态</v-card-title>
          </v-card-item>
          <v-divider />
          <v-card-text class="pt-4" v-if="metrics?.scheduler.workers.length">
            <v-row dense>
              <v-col v-for="stat in workerStats" :key="stat.state" cols="6" class="mb-3">
                <v-card
                  :color="stat.color"
                  variant="outlined"
                  class="worker-stat-card"
                >
                  <v-card-text class="pa-3">
                    <div class="d-flex flex-column align-center">
                      <span class="text-h4 font-weight-bold mb-1">{{ stat.count }}</span>
                      <span class="text-caption text-center">{{ stat.state }}</span>
                    </div>
                  </v-card-text>
                </v-card>
              </v-col>
            </v-row>
          </v-card-text>
          <v-card-text v-else class="text-center pa-4">
            暂无数据
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- 日志查看器 -->
    <v-row>
      <v-col cols="12">
        <LogViewer 
          websocketUrl="/ws" 
          :expanded="true"
        />
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.dashboard {
  background-color: rgb(var(--v-theme-background));
  min-height: 100%;
}

.v-card {
  border-radius: 12px;
  transition: transform 0.2s, box-shadow 0.2s;
}

.v-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 25px 0 rgba(var(--v-theme-on-surface), 0.1);
}

.refresh-select {
  width: 120px;
}

.refresh-select :deep(.v-field__input) {
  min-height: 36px;
}

.v-card-item {
  padding: 20px;
}

.v-card-text {
  padding: 20px;
}

.v-table {
  background: transparent !important;
}

.v-table th {
  font-size: 0.875rem !important;
  color: rgba(var(--v-theme-on-surface), 0.7) !important;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.v-table td {
  height: 48px;
  color: rgba(var(--v-theme-on-surface), 0.87);
}

.v-divider {
  opacity: 0.08;
}

.worker-stat-card {
  transition: all 0.2s ease;
}

.worker-stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 8px rgba(var(--v-theme-on-surface), 0.1);
}

.text-caption {
  text-transform: capitalize;
  opacity: 0.9;
}
</style> 