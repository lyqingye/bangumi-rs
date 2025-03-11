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
            :loading="searching"
            clearable
            @click:clear="clearSearch"
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

    <!-- 搜索结果展示 -->
    <div v-if="showSearchResults" class="search-results-container mb-6">
      <v-card class="search-results-card">
        <div class="d-flex justify-space-between align-center pa-4">
          <h3 class="text-h6">搜索结果: {{ searchQuery }}</h3>
          <v-btn icon variant="text" @click="clearSearch">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </div>
        
        <v-divider></v-divider>
        
        <div v-if="searching" class="d-flex justify-center align-center pa-8">
          <v-progress-circular indeterminate color="primary" size="48" />
        </div>
        
        <div v-else-if="searchError" class="text-center pa-8 text-error">
          {{ searchError }}
        </div>
        
        <div v-else-if="searchResults.length === 0" class="text-center pa-8 text-medium-emphasis">
          未找到相关番剧，请尝试其他关键词
        </div>
        
        <v-row v-else class="pa-4">
          <v-col
            v-for="result in searchResults"
            :key="result.id"
            cols="12"
            sm="6"
            md="4"
            lg="3"
            xl="2"
            class="search-result-col"
          >
            <v-card
              class="search-result-card"
              elevation="2"
            >
              <v-img
                :src="result.image_url || '/placeholder-image.jpg'"
                height="400"
                cover
                class="search-result-image"
              >
                <template v-slot:placeholder>
                  <div class="d-flex align-center justify-center fill-height">
                    <v-progress-circular indeterminate color="primary"></v-progress-circular>
                  </div>
                </template>
              </v-img>
              <v-card-title class="search-result-title text-truncate">
                {{ result.title }}
              </v-card-title>
              <v-card-subtitle class="search-result-links">
                <div class="d-flex flex-column">
                  <v-btn
                    variant="text"
                    class="link-btn"
                    :href="`https://mikanani.me/Home/Bangumi/${result.id}`"
                    target="_blank"
                    @click.stop
                  >
                    <v-icon size="small" class="me-1">mdi-link</v-icon>
                    Mikan #{{ result.id }}
                  </v-btn>
                  <v-btn
                    v-if="result.bangumi_tv_id"
                    variant="text"
                    class="link-btn"
                    :href="`https://bgm.tv/subject/${result.bangumi_tv_id}`"
                    target="_blank"
                    @click.stop
                  >
                    <v-icon size="small" class="me-1">mdi-link</v-icon>
                    BgmTV #{{ result.bangumi_tv_id }}
                  </v-btn>
                </div>
              </v-card-subtitle>
              <v-btn
                icon
                variant="elevated"
                size="small"
                color="primary"
                class="add-btn"
                @click.stop="handleSearchResultClick(result)"
                :loading="addingBangumi[result.id]"
                :disabled="addingBangumi[result.id]"
              >
                <v-icon>mdi-plus</v-icon>
                <v-tooltip
                  activator="parent"
                  location="top"
                  text="添加到订阅列表"
                />
              </v-btn>
            </v-card>
          </v-col>
        </v-row>
      </v-card>
    </div>

    <v-row v-if="!showSearchResults">
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
import { fetchCalendar, refreshCalendar, searchBangumiAtMikan, addBangumi } from '@/api/api'
import { type Bangumi, type MikanSearchResultItem, type AddBangumiParams } from '@/api/model'
import MediaCard from '@/components/MediaCard.vue'
import RefreshDialog from '@/components/RefreshDialog.vue'
import { useSnackbar } from '@/composables/useSnackbar'
import { useSeason } from '@/stores/season'
import { useRouter } from 'vue-router'

const { showSnackbar } = useSnackbar()
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

const selectedWeekday = ref(String(new Date().getDay()))
const calendarItems = ref<Bangumi[]>([])
const loading = ref(false)
const refreshing = ref(false)
const error = ref('')

// 添加刷新对话框状态
const showRefreshDialog = ref(false)

// 搜索相关状态
const searchQuery = ref('')
const searchResults = ref<MikanSearchResultItem[]>([])
const searching = ref(false)
const searchError = ref('')
const hasSearched = ref(false)
const showSearchResults = computed(() => searchQuery.value.trim() !== '' && hasSearched.value)

const filteredCalendarItems = computed(() => {
  if (selectedWeekday.value === '-1') {
    return calendarItems.value
  }
  return calendarItems.value.filter(item => item.air_week === parseInt(selectedWeekday.value))
})

const loadCalendarData = async () => {
  loading.value = true
  error.value = ''
  try {
    const season = getCalendarSeason();
    calendarItems.value = await fetchCalendar(season || '')
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
    await refreshCalendar(season || '', force)
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

// 添加 router
const router = useRouter()

// 添加加载状态
const addingBangumi = ref<Record<number, boolean>>({})

// 搜索处理函数
const handleSearch = async () => {
  if (!searchQuery.value.trim()) return
  
  searching.value = true
  searchError.value = ''
  hasSearched.value = true
  
  try {
    searchResults.value = await searchBangumiAtMikan(searchQuery.value.trim())
  } catch (e) {
    searchError.value = e instanceof Error ? e.message : '搜索失败'
    searchResults.value = []
  } finally {
    searching.value = false
  }
}

// 清除搜索结果
const clearSearch = () => {
  searchQuery.value = ''
  searchResults.value = []
  searchError.value = ''
  hasSearched.value = false
}

// 处理搜索结果点击
const handleSearchResultClick = async (result: MikanSearchResultItem) => {
  // 设置加载状态
  addingBangumi.value[result.id] = true
  
  try {
    // 准备添加番剧的参数
    const params: AddBangumiParams = {
      title: result.title,
      mikan_id: result.id,
      bgm_tv_id: result.bangumi_tv_id || null
    }
    
    // 调用添加番剧接口
    const bangumiId = await addBangumi(params)
    
    // 显示成功提示
    showSnackbar({
      text: '添加成功，正在跳转...',
      color: 'success',
      location: 'top right',
      timeout: 2000
    })
    
    // 跳转到详情页
    router.push({
      name: 'detail',
      params: { id: bangumiId }
    })
  } catch (e) {
    // 错误已经在 API 层处理，这里不需要额外处理
  } finally {
    // 清除加载状态
    delete addingBangumi.value[result.id]
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
  // 初始化季节信息
  await initializeSeasonInfo();
  // 加载番剧列表
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

/* 搜索结果样式 */
.search-results-container {
  width: 100%;
}

.search-results-card {
  background: rgba(32, 32, 32, 0.95);
  border-radius: 16px;
  overflow: hidden;
}

.search-result-col {
  transition: transform 0.2s ease;
}

.search-result-col:hover {
  transform: translateY(-4px);
}

.search-result-card {
  height: 100%;
  background: rgba(48, 48, 48, 0.95);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
  cursor: pointer;
  position: relative;
}

.search-result-card:hover {
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
}

.search-result-image {
  position: relative;
}

.add-btn {
  position: absolute !important;
  right: 12px;
  bottom: 12px;
  opacity: 0;
  transform: scale(0.8);
  transition: all 0.3s ease !important;
  z-index: 1;
}

.search-result-card:hover .add-btn {
  opacity: 1;
  transform: scale(1);
}

.search-result-title {
  font-size: 0.95rem;
  line-height: 1.3;
  padding: 12px 12px 4px;
  padding-right: 48px;
  font-weight: 500;
}

.search-result-links {
  padding: 4px 12px 12px;
  padding-right: 48px;
}

.search-result-links .d-flex {
  gap: 4px;
}

.link-btn {
  height: 24px !important;
  min-width: unset !important;
  padding: 0 !important;
  background: transparent !important;
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: none;
  letter-spacing: normal;
  justify-content: flex-start;
}

.link-btn:hover {
  color: rgba(255, 255, 255, 0.9);
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
