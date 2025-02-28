<template>
  <div class="home-content">
    <!-- 搜索框和筛选器 -->
    <div class="position-relative mb-6">
      <div class="d-flex align-center justify-space-between">
        <!-- 左侧空白区域，用于平衡布局 -->
        <div class="flex-grow-1 flex-shrink-1" style="max-width: 200px;"></div>
        
        <!-- 搜索框居中 -->
        <div class="search-container">
          <v-text-field
            density="compact"
            hide-details
            prepend-inner-icon="mdi-magnify"
            placeholder="搜索..."
            variant="solo-filled"
            class="search-field"
            bg-color="rgba(32, 32, 32, 0.95)"
          />
        </div>
        
        <!-- 右侧筛选器和刷新按钮 -->
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
            style="height: 40px;"
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
            style="height: 40px;"
          ></v-select>
          
          <v-btn
            :loading="refreshing"
            :disabled="loading || refreshing"
            variant="tonal"
            class="refresh-btn"
            size="40"
            icon
            @click="handleRefresh"
          >
            <v-icon>mdi-refresh</v-icon>
            <v-tooltip
              activator="parent"
              location="bottom"
              text="刷新放送列表"
            />
          </v-btn>
        </div>
      </div>
    </div>

    <v-row>
      <!-- 星期导航条 -->
      <v-col cols="12" class="mb-6">
        <v-card class="weekday-tabs-card" elevation="0">
          <v-tabs
            v-model="selectedWeekday"
            color="primary"
            class="weekday-tabs"
            show-arrows
            fixed-tabs
          >
            <v-tab v-for="tab in weekTabs" :key="tab.value" :value="tab.value" class="text-body-2">
              {{ tab.label }}
            </v-tab>
          </v-tabs>
        </v-card>
      </v-col>

      <v-col
        v-if="loading"
        cols="12"
        class="d-flex justify-center align-center"
        style="min-height: 400px"
      >
        <v-progress-circular indeterminate color="primary" size="64" />
      </v-col>
      <v-col v-else-if="error" cols="12" class="text-center text-error">
        {{ error }}
      </v-col>
      <template v-else>
        <v-col
          v-for="item in filteredCalendarItems"
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
      </template>
    </v-row>
    
    <!-- 添加刷新对话框 -->
    <RefreshDialog
      v-model="showRefreshDialog"
      @confirm="handleRefreshConfirm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { fetchCalendar, refreshCalendar, fetchCalendarSeason } from '@/api/api'
import { type Bangumi } from '@/api/model'
import MediaCard from '@/components/MediaCard.vue'
import RefreshDialog from '@/components/RefreshDialog.vue'
import { useSnackbar } from '@/composables/useSnackbar'

const { showSnackbar } = useSnackbar()
// 定义选项类型
interface SelectOption<T> {
  title: string;
  value: T;
}

const selectedWeekday = ref(String(new Date().getDay()))
const calendarItems = ref<Bangumi[]>([])
const loading = ref(false)
const refreshing = ref(false)
const error = ref('')

// 添加刷新对话框状态
const showRefreshDialog = ref(false)

// 季节筛选相关
const selectedYear = ref<number>(new Date().getFullYear())
const selectedSeason = ref<string>('冬季番组')
const availableSeasons = ref<string[]>([])

// 年份选项
const yearOptions = computed<SelectOption<number>[]>(() => {
  const options: SelectOption<number>[] = [];
  
  // 获取当前年份
  const currentYear = new Date().getFullYear();
  
  // 从当前年份到2015年
  for (let year = currentYear; year >= 2015; year--) {
    options.push({
      title: `${year}年`,
      value: year
    });
  }
  
  return options;
});

// 季节选项
const seasonOptions = computed<SelectOption<string>[]>(() => {
  // 基础季节选项
  return [
    { title: '冬季番组', value: '冬季番组' },
    { title: '春季番组', value: '春季番组' },
    { title: '夏季番组', value: '夏季番组' },
    { title: '秋季番组', value: '秋季番组' }
  ];
});

const filteredCalendarItems = computed(() => {
  if (selectedWeekday.value === '-1') {
    return calendarItems.value
  }
  return calendarItems.value.filter(item => item.air_week === parseInt(selectedWeekday.value))
})

// 获取当前选择的季度值
function getCalendarSeason(): string {
  return `${selectedYear.value} ${selectedSeason.value}`;
}

const loadCalendarData = async () => {
  loading.value = true
  error.value = ''
  try {
    const season = getCalendarSeason();
    calendarItems.value = await fetchCalendar(season)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '获取数据失败'
  } finally {
    loading.value = false
  }
}

// 修改刷新按钮点击处理函数，显示刷新对话框
const handleRefresh = async () => {
  showRefreshDialog.value = true
}

// 添加刷新确认处理函数
const handleRefreshConfirm = async (force: boolean) => {
  refreshing.value = true
  try {
    const season = getCalendarSeason();
    await refreshCalendar(season, force)
    await loadCalendarData()
    showSnackbar({
      text: '已经加入刷新队列',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
  } catch (e) {
    // 错误已经在 API 层处理
  } finally {
    refreshing.value = false
    // 关闭对话框
    showRefreshDialog.value = false
  }
}

// 监听年份变化
watch(selectedYear, () => {
  loadCalendarData();
});

// 监听季节变化
watch(selectedSeason, () => {
  loadCalendarData();
});

// 获取最新的季度信息并设置默认选择
const fetchLatestSeason = async () => {
  try {
    const latestSeason = await fetchCalendarSeason();
    if (latestSeason) {
      // 保存可用的季度信息
      availableSeasons.value = [latestSeason];
      
      // 解析季度信息，格式如：2025 冬季番组
      const parts = latestSeason.split(' ');
      if (parts.length === 2) {
        const year = parseInt(parts[0]);
        const season = parts[1];
        // 设置年份和季节选择
        selectedYear.value = year;
        selectedSeason.value = season;
      } else {
        // 如果没有获取到完整的季度信息，则使用当前年份和默认季节
        selectedYear.value = new Date().getFullYear();
        selectedSeason.value = '冬季番组';
      }
    } else {
      // 如果没有获取到季度信息，则使用当前年份和默认季节
      selectedYear.value = new Date().getFullYear();
      selectedSeason.value = '冬季番组';
    }
  } catch (e) {
    console.error('获取最新季度信息失败:', e);
    // 出错时使用当前年份和默认季节
    selectedYear.value = new Date().getFullYear();
    selectedSeason.value = '冬季番组';
  }
}

// 星期标签配置
const weekTabs = [
  { label: '全部', value: '-1' },
  { label: '周一', value: '1' },
  { label: '周二', value: '2' },
  { label: '周三', value: '3' },
  { label: '周四', value: '4' },
  { label: '周五', value: '5' },
  { label: '周六', value: '6' },
  { label: '周日', value: '0' }
]

onMounted(async () => {
  // 先获取最新季度信息
  await fetchLatestSeason();
  // 然后加载番剧列表
  loadCalendarData()
})
</script>

<style scoped>
.home-content {
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

.weekday-tabs-card {
  background: rgba(32, 32, 32, 0.95);
  border-radius: 16px;
  padding: 8px;
}

.weekday-tabs {
  min-height: 48px;
  background: transparent;
}

/* 完全重写 tab 样式 */
.weekday-tabs :deep(.v-slide-group__content) {
  gap: 4px;
}

.weekday-tabs :deep(.v-tab) {
  min-width: 86px;
  height: 36px;
  border-radius: 8px !important;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 500;
  transition: all 0.3s ease;
  opacity: 0.7;
  padding: 0 16px;
  color: rgba(255, 255, 255, 0.9);
}

/* 移除所有默认的背景和边框 */
.weekday-tabs :deep(.v-tab::before),
.weekday-tabs :deep(.v-tab::after) {
  display: none;
}

/* 移除滑块 */
.weekday-tabs :deep(.v-tabs__slider) {
  display: none;
}

/* 选中状态 */
.weekday-tabs :deep(.v-tab--selected) {
  background: rgba(var(--v-theme-primary), 0.15) !important;
  color: rgb(var(--v-theme-primary)) !important;
  opacity: 1;
}

/* 悬停状态 */
.weekday-tabs :deep(.v-tab:hover:not(.v-tab--selected)) {
  background: rgba(255, 255, 255, 0.05);
  opacity: 0.9;
}

/* 箭头按钮样式 */
.weekday-tabs :deep(.v-slide-group__prev),
.weekday-tabs :deep(.v-slide-group__next) {
  border-radius: 8px;
}

.weekday-tabs :deep(.v-slide-group__prev:hover),
.weekday-tabs :deep(.v-slide-group__next:hover) {
  background: rgba(255, 255, 255, 0.05);
}

.media-card-col {
  transition: transform 0.2s ease;
}

.refresh-btn {
  border-radius: 50% !important;
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
@media (max-width: 768px) {
  .search-container {
    max-width: 200px;
  }
  
  .year-select {
    width: 80px;
  }
  
  .season-select {
    width: 90px;
  }
}

@media (max-width: 600px) {
  .position-relative > div {
    flex-direction: column;
    align-items: stretch;
  }
  
  .search-container {
    max-width: 100%;
    margin-bottom: 12px;
    order: 1;
  }
  
  .filter-container {
    width: 100%;
    justify-content: flex-end;
    order: 2;
  }
}
</style>
