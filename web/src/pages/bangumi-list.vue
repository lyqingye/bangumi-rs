<template>
  <div class="bangumi-list-content">
    <div class="position-relative mb-6">
      <div class="d-flex align-center justify-space-between">
        <!-- 左侧空白区域，用于平衡布局 -->
        <div class="flex-grow-1 flex-shrink-1" style="max-width: 200px;">
          <h1 class="text-h5 font-weight-bold">番剧列表</h1>
        </div>
        
        <!-- 搜索框居中 -->
        <div class="search-container">
          <v-text-field
            v-model="searchQuery"
            density="compact"
            hide-details
            prepend-inner-icon="mdi-magnify"
            placeholder="搜索..."
            variant="solo-filled"
            class="search-field"
            bg-color="rgba(32, 32, 32, 0.95)"
            @keyup.enter="handleSearch"
            @click:prepend-inner="handleSearch"
            clearable
            @click:clear="clearSearch"
          />
        </div>
        
        <!-- 右侧筛选器 -->
        <div class="d-flex align-center filter-container">
          <!-- 年份选择器 -->
          <v-select
            v-model="selectedYear"
            :items="yearOptions"
            label="年份"
            variant="outlined"
            density="compact"
            hide-details
            class="me-2 year-select"
            bg-color="rgba(48, 48, 48, 0.95)"
            item-color="white"
          ></v-select>
          
          <!-- 季节选择器 -->
          <v-select
            v-model="selectedSeason"
            :items="seasonOptions"
            label="季节"
            variant="outlined"
            density="compact"
            hide-details
            class="me-2 season-select"
            bg-color="rgba(48, 48, 48, 0.95)"
            item-color="white"
          ></v-select>
          
          <!-- 订阅状态过滤器 -->
          <v-select
            v-model="selectedStatus"
            :items="statusOptions"
            label="订阅状态"
            variant="outlined"
            density="compact"
            class="season-select"
            hide-details
            bg-color="rgba(48, 48, 48, 0.95)"
            item-color="white"
          ></v-select>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="d-flex justify-center align-center" style="min-height: 400px">
      <v-progress-circular indeterminate color="primary" size="64" />
    </div>

    <!-- 错误提示 -->
    <v-alert v-else-if="error" type="error" class="mb-4">
      {{ error }}
    </v-alert>

    <!-- 番剧列表 -->
    <template v-else>
      <!-- 无数据提示 -->
      <v-alert v-if="bangumis.length === 0" type="info" class="mb-4">
        没有找到符合条件的番剧
      </v-alert>

      <!-- 番剧卡片网格 -->
      <v-row>
        <v-col
          v-for="item in bangumis"
          :key="item.id"
          cols="12"
          sm="6"
          md="4"
          lg="3"
          xl="2"
          class="media-card-col"
        >
          <MediaCard :item="item" />
        </v-col>
      </v-row>

      <!-- 分页控件 -->
      <div class="d-flex justify-center mt-6">
        <v-pagination
          v-model="currentPage"
          :length="totalPages"
          :total-visible="7"
          rounded
          @update:model-value="handlePageChange"
        ></v-pagination>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { fetchBangumiList } from '@/api/api'
import { SubscribeStatus, type Bangumi, type QueryBangumiParams } from '@/api/model'
import MediaCard from '@/components/MediaCard.vue'
import { useSeason } from '@/stores/season'

// 分页参数
const pageSize = 12
const currentPage = ref(1)
const total = ref(0)
const totalPages = computed(() => Math.ceil(total.value / pageSize))

// 搜索参数
const searchQuery = ref('')

// 使用season store
const { 
  state: seasonState, 
  yearOptions, 
  seasonOptions, 
  setYear, 
  setSeason, 
  getCalendarSeason, 
  initializeSeasonInfo 
} = useSeason()

// 将本地状态映射到store状态
const selectedYear = computed({
  get: () => seasonState.selectedYear,
  set: (value) => setYear(value)
})

const selectedSeason = computed({
  get: () => seasonState.selectedSeason,
  set: (value) => setSeason(value)
})

// 订阅状态
const selectedStatus = ref<SubscribeStatus | null>(null)

// 数据状态
const bangumis = ref<Bangumi[]>([])
const loading = ref(false)
const error = ref('')

// 订阅状态选项
const statusOptions = [
  { title: '全部状态', value: null },
  { title: '已订阅', value: SubscribeStatus.Subscribed },
  { title: '已完成', value: SubscribeStatus.Downloaded },
  { title: '未订阅', value: SubscribeStatus.None },
]

// 加载番剧列表数据
const loadBangumiList = async () => {
  loading.value = true
  error.value = ''
  
  try {
    // 构建查询参数
    const params: QueryBangumiParams = {
      offset: (currentPage.value - 1) * pageSize,
      limit: pageSize,
      status: selectedStatus.value || undefined,
      calendar_season: getCalendarSeason(),
      name: searchQuery.value || undefined
    }
    
    // 调用API
    const result = await fetchBangumiList(params)
    
    // 更新数据
    bangumis.value = result.list
    total.value = result.total
  } catch (e) {
    error.value = e instanceof Error ? e.message : '获取数据失败'
  } finally {
    loading.value = false
  }
}

// 处理页码变化
const handlePageChange = () => {
  loadBangumiList()
}

// 处理搜索
const handleSearch = () => {
  currentPage.value = 1 // 重置到第一页
  loadBangumiList()
}

// 清除搜索
const clearSearch = () => {
  searchQuery.value = ''
  currentPage.value = 1
  loadBangumiList()
}

// 监听年份变化
watch(selectedYear, () => {
  currentPage.value = 1 // 重置到第一页
  loadBangumiList()
})

// 监听季节变化
watch(selectedSeason, () => {
  currentPage.value = 1 // 重置到第一页
  loadBangumiList()
})

// 监听订阅状态变化
watch(selectedStatus, () => {
  currentPage.value = 1 // 重置到第一页
  loadBangumiList()
})

// 监听搜索关键词变化
watch(searchQuery, () => {
  if (!searchQuery.value) {
    loadBangumiList()
  }
})

// 组件挂载时加载数据
onMounted(async () => {
  // 初始化季节信息
  await initializeSeasonInfo()
  // 加载番剧列表
  loadBangumiList()
})
</script>

<style scoped>
.bangumi-list-content {
  max-width: 1920px;
  margin: 0 auto;
  padding: 24px;
}

.position-relative {
  position: relative;
}

.search-container {
  max-width: 300px;
  width: 100%;
}

.filter-container {
  flex-shrink: 0;
}

/* 搜索框样式 */
.search-field {
  width: 100%;
}

.search-field :deep(.v-field__input) {
  min-height: 40px;
  font-size: 0.875rem;
}

.search-field :deep(.v-field) {
  border-radius: 8px;
  background: rgba(32, 32, 32, 0.95);
}

.search-field :deep(.v-field__input) {
  padding: 8px 12px;
}

.search-field :deep(.v-field__prepend-inner) {
  padding-left: 12px;
}

.media-card-col {
  transition: transform 0.2s ease;
}

.year-select,
.season-select {
  font-weight: 500;
}

.year-select :deep(.v-field__input),
.season-select :deep(.v-field__input) {
  color: white;
  min-height: 40px;
  font-size: 0.875rem;
}

.year-select :deep(.v-field),
.season-select :deep(.v-field) {
  border-radius: 8px;
  background: rgba(48, 48, 48, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.year-select :deep(.v-field__append-inner),
.season-select :deep(.v-field__append-inner) {
  color: rgba(255, 255, 255, 0.7);
}

.year-select :deep(.v-list),
.season-select :deep(.v-list) {
  background: rgba(48, 48, 48, 0.98) !important;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.year-select :deep(.v-list-item),
.season-select :deep(.v-list-item) {
  color: rgba(255, 255, 255, 0.9) !important;
}

.year-select :deep(.v-list-item--active),
.season-select :deep(.v-list-item--active) {
  color: rgb(var(--v-theme-primary)) !important;
  background: rgba(var(--v-theme-primary), 0.15) !important;
}

/* 响应式调整 */
@media (max-width: 1200px) {
  .search-container {
    max-width: 240px;
  }
}

@media (max-width: 960px) {
  .d-flex.align-center.justify-space-between {
    flex-direction: column;
    align-items: stretch !important;
  }

  .search-container {
    max-width: 100%;
    margin: 16px 0;
    order: 1;
  }

  .filter-container {
    width: 100%;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
    order: 2;
  }

  .year-select,
  .season-select {
    flex: 1;
    min-width: 120px;
  }

  h1.text-h5 {
    margin-bottom: 0;
  }
}

@media (max-width: 600px) {
  .filter-container {
    justify-content: stretch;
  }
  
  .year-select,
  .season-select {
    min-width: calc(33.33% - 6px);
  }
}
</style> 