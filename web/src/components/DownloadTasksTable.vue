<template>
  <div class="download-tasks-table">
    <v-card class="mb-4" elevation="0" color="rgba(32, 32, 32, 0.95)" rounded="lg">
      <v-card-title class="d-flex align-center pa-4">
        <span class="text-h6 font-weight-medium">下载任务列表</span>
        <v-spacer />
        <v-select
          v-model="selectedStatus"
          :items="statusOptions"
          density="compact"
          class="status-filter"
          hide-details
          variant="solo"
          bg-color="rgba(0, 0, 0, 0.2)"
          rounded="lg"
        >
          <template #selection="{ item }">
            <v-chip
              size="small"
              :color="item.raw.value ? getStatusColor(item.raw.value) : 'default'"
              variant="flat"
              class="font-weight-medium"
            >
              {{ item.title }}
            </v-chip>
          </template>
          <template #item="{ item, props }">
            <v-list-item
              v-bind="props"
              :prepend-icon="item.raw.value ? 'mdi-circle-small' : ''"
              :class="{ 'status-item': true, [`status-${item.raw.value}`]: item.raw.value }"
            >
              <template #prepend>
                <div
                  class="status-dot"
                  :class="item.raw.value ? `bg-${getStatusColor(item.raw.value)}` : ''"
                />
              </template>
              <template #title>
                {{ item.title }}
              </template>
            </v-list-item>
          </template>
        </v-select>
      </v-card-title>

      <v-data-table
        :headers="headers"
        :items="tasks"
        :loading="loading"
        :items-per-page="-1"
        class="elevation-0 mt-2 download-table rounded-lg"
        :expanded="expandedItems"
        show-expand
        hover
        item-value="info_hash"
        @update:expanded="e => (expandedItems = e)"
      >
        <template #[`item.name`]="{ item }">
          <div class="d-flex align-center">
            <span class="text-body-2 font-weight-medium">{{ item.name }}</span>
            <v-btn
              size="x-small"
              variant="text"
              density="comfortable"
              icon="mdi-open-in-new"
              class="ms-2 detail-btn"
              @click="navigateToBangumiDetail(item.bangumi_id)"
            />
          </div>
        </template>

        <template #[`item.file_size`]="{ item }">
          <span class="text-body-2 font-weight-medium">{{ formatFileSize(item.file_size) }}</span>
        </template>

        <template #[`item.episode_number`]="{ item }">
          <span class="text-body-2 font-weight-medium">{{ item.episode_number }}</span>
        </template>

        <template #[`item.download_status`]="{ item }">
          <v-chip
            :color="getStatusColor(item.download_status)"
            size="small"
            class="text-caption font-weight-medium"
            variant="flat"
          >
            {{ getStatusText(item.download_status) }}
          </v-chip>
        </template>

        <template #[`item.created_at`]="{ item }">
          <span class="text-body-2 font-weight-medium">{{ formatDate(item.created_at) }}</span>
        </template>

        <template #[`item.updated_at`]="{ item }">
          <span class="text-body-2 font-weight-medium">{{ formatDate(item.updated_at) }}</span>
        </template>

        <template #expanded-row="{ item }">
          <td :colspan="headers.length">
            <v-card flat class="ma-2 pa-4 expanded-details" color="rgba(0, 0, 0, 0.2)" rounded="lg">
              <v-row dense class="expanded-row">
                <v-col cols="12" sm="6" md="4" class="mb-3">
                  <div class="text-caption text-medium-emphasis mb-2">文件名</div>
                  <div class="text-body-1 font-weight-medium text-truncate">
                    {{ item.file_name || '-' }}
                  </div>
                </v-col>
                <v-col cols="12" sm="6" md="4" class="mb-3">
                  <div class="text-caption text-medium-emphasis mb-2">下载器</div>
                  <div class="text-body-1 font-weight-medium">{{ item.downloader || '-' }}</div>
                </v-col>
                <v-col cols="12" sm="6" md="4" class="mb-3">
                  <div class="text-caption text-medium-emphasis mb-2">重试次数</div>
                  <div class="text-body-1 font-weight-medium">{{ item.retry_count || 0 }}</div>
                </v-col>
                <v-col cols="12" class="mt-2">
                  <div class="text-caption text-medium-emphasis mb-2">错误信息</div>
                  <div class="error-message pa-3">
                    {{ item.err_msg || '无错误信息' }}
                  </div>
                </v-col>
              </v-row>
            </v-card>
          </td>
        </template>

        <template #no-data>
          <div class="d-flex justify-center align-center pa-6 text-medium-emphasis">
            暂无下载任务
          </div>
        </template>

        <template #bottom>
          <div class="d-flex align-center justify-center pa-4">
            <v-pagination
              v-if="tasks.length > 0"
              v-model="currentPage"
              :length="currentPage + (hasNextPage ? 1 : 0)"
              :total-visible="3"
              :show-first-last-page="true"
              rounded="lg"
              active-color="primary"
            />
          </div>
        </template>
      </v-data-table>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { fetchDownloadTasks } from '@/api/api'
import type { DownloadTask } from '@/api/model'
import { DownloadStatus } from '@/api/model'
import { useRouter } from 'vue-router'

const router = useRouter()
const loading = ref(false)
const tasks = ref<DownloadTask[]>([])
const currentPage = ref(1)
const pageSize = ref(10)
const hasNextPage = ref(false)
const selectedStatus = ref<DownloadStatus | null>(null)
const expandedItems = ref<string[]>([])

const headers = [
  { title: '番剧名称', key: 'name', align: 'start' as const },
  { title: '集数', key: 'episode_number', align: 'center' as const },
  { title: '文件大小', key: 'file_size', align: 'center' as const },
  { title: '下载状态', key: 'download_status', align: 'center' as const },
  { title: '创建时间', key: 'created_at', align: 'center' as const },
  { title: '更新时间', key: 'updated_at', align: 'center' as const }
]

const statusOptions = [
  { title: '全部', value: null },
  { title: '等待中', value: DownloadStatus.Pending },
  { title: '下载中', value: DownloadStatus.Downloading },
  { title: '已完成', value: DownloadStatus.Completed },
  { title: '失败', value: DownloadStatus.Failed }
]

const loadTasks = async () => {
  loading.value = true
  try {
    const offset = (currentPage.value - 1) * pageSize.value
    const params = {
      offset,
      limit: pageSize.value + 1, // 多请求一条数据用于判断是否有下一页
      status: selectedStatus.value || undefined
    }
    const data = await fetchDownloadTasks(params)

    // 判断是否有下一页
    hasNextPage.value = data.length > pageSize.value

    // 如果有下一页，则移除多余的一条数据
    tasks.value = hasNextPage.value ? data.slice(0, -1) : data
  } catch (error) {
    console.error('加载下载任务失败:', error)
  } finally {
    loading.value = false
  }
}

const getStatusColor = (status: DownloadStatus) => {
  switch (status) {
    case DownloadStatus.Pending:
      return 'warning'
    case DownloadStatus.Downloading:
      return 'info'
    case DownloadStatus.Completed:
      return 'success'
    case DownloadStatus.Failed:
      return 'error'
    default:
      return 'grey'
  }
}

const getStatusText = (status: DownloadStatus) => {
  switch (status) {
    case DownloadStatus.Pending:
      return '等待中'
    case DownloadStatus.Downloading:
      return '下载中'
    case DownloadStatus.Completed:
      return '已完成'
    case DownloadStatus.Failed:
      return '失败'
    default:
      return '未知'
  }
}

const formatDate = (date: string) => {
  return new Date(date).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 格式化文件大小
const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
}

// 跳转到番剧详情页
const navigateToBangumiDetail = (bangumiId: number) => {
  router.push(`/detail/${bangumiId}`)
}

watch([currentPage, selectedStatus], () => {
  loadTasks()
})

onMounted(() => {
  loadTasks()
})
</script>

<style scoped>
.download-tasks-table {
  padding: 24px;
}

.status-filter {
  max-width: 160px;
}

.detail-btn {
  opacity: 0;
  transition: opacity 0.2s ease;
}

:deep(.v-data-table__tr:hover) .detail-btn {
  opacity: 0.7;
}

:deep(.detail-btn:hover) {
  opacity: 1 !important;
}

:deep(.v-select .v-field) {
  border-radius: 8px !important;
  overflow: hidden;
}

:deep(.v-select .v-field__input) {
  min-height: 36px;
  padding-top: 0;
  padding-bottom: 0;
  min-width: 100px;
}

:deep(.v-select .v-field__append-inner) {
  padding-inline-start: 8px;
}

:deep(.v-list) {
  background: rgba(32, 32, 32, 0.98) !important;
  border-radius: 8px !important;
  padding: 4px !important;
  overflow: hidden;
}

:deep(.v-list-item) {
  min-height: 36px !important;
  padding: 0 8px !important;
  border-radius: 4px !important;
  margin: 2px 0 !important;
}

:deep(.v-list-item:hover) {
  background: rgba(255, 255, 255, 0.05) !important;
}

:deep(.status-dot) {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 8px;
  background: rgba(255, 255, 255, 0.2);
}

:deep(.bg-warning) {
  background: rgb(var(--v-theme-warning));
}

:deep(.bg-info) {
  background: rgb(var(--v-theme-info));
}

:deep(.bg-success) {
  background: rgb(var(--v-theme-success));
}

:deep(.bg-error) {
  background: rgb(var(--v-theme-error));
}

:deep(.v-data-table) {
  background: transparent !important;
}

:deep(.v-data-table-header) {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px 8px 0 0;
}

:deep(.v-data-table-header th) {
  font-size: 0.875rem !important;
  font-weight: 600 !important;
  color: rgba(255, 255, 255, 0.9) !important;
  border-bottom: none !important;
  letter-spacing: 0.0125em;
}

:deep(.v-data-table__tr) {
  transition: all 0.2s ease;
  border: none !important;
}

:deep(.v-data-table__tr td) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.05) !important;
  padding: 12px 16px !important;
  font-size: 0.875rem !important;
  font-weight: 500 !important;
  letter-spacing: 0.0125em;
  color: rgba(255, 255, 255, 0.85) !important;
}

:deep(.v-data-table__tr:hover) {
  background: rgba(255, 255, 255, 0.05) !important;
}

:deep(.v-data-table__expanded) {
  background: transparent !important;
}

:deep(.v-data-table__expanded td) {
  padding: 0 !important;
}

:deep(.v-data-table-footer) {
  background: transparent !important;
  border-radius: 0 0 8px 8px;
}

:deep(.v-data-table__expand-icon) {
  color: rgba(255, 255, 255, 0.7) !important;
  margin-right: 8px !important;
}

:deep(.v-chip.v-chip--size-small) {
  font-size: 0.75rem !important;
  height: 24px !important;
  min-width: 70px;
  justify-content: center;
  font-weight: 600 !important;
  letter-spacing: 0.0125em;
}

.text-truncate {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

:deep(.v-table) {
  border-radius: 8px;
  overflow: hidden;
}

:deep(.v-table__wrapper) {
  border-radius: 8px;
  overflow: hidden;
}

:deep(.v-overlay__content) {
  border-radius: 8px;
  overflow: hidden;
}

:deep(.v-card) {
  border-radius: 8px !important;
  overflow: hidden;
}

.expanded-details {
  background: rgba(0, 0, 0, 0.3) !important;
  backdrop-filter: blur(10px);
}

.expanded-row {
  position: relative;
}

.error-message {
  background: rgba(var(--v-theme-error), 0.1);
  border-radius: 8px;
  color: rgb(var(--v-theme-error));
  font-family: 'Roboto Mono', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

:deep(.text-body-2) {
  font-size: 0.875rem !important;
  font-weight: 500 !important;
  letter-spacing: 0.0125em;
  color: rgba(255, 255, 255, 0.85) !important;
}

:deep(.text-caption) {
  font-size: 0.75rem !important;
  font-weight: 500 !important;
  letter-spacing: 0.0125em;
  opacity: 0.9;
}

:deep(.text-medium-emphasis) {
  color: rgba(255, 255, 255, 0.85) !important;
}
</style>
