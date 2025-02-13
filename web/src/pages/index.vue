<template>
  <v-layout class="fill-height">
    <!-- 左侧导航树 -->
    <v-navigation-drawer
      permanent
      class="navigation-drawer"
      elevation="0"
      width="280"
    >
      <!-- 毛玻璃背景 -->
      <div class="glass-bg"></div>

      <!-- 导航内容 -->
      <div class="nav-content">
        <div class="pa-6">
          <div class="text-h5 font-weight-bold">Bangumi App</div>
        </div>

        <v-list class="nav-list px-3">
          <v-list-item
            v-for="(item, i) in navItems"
            :key="i"
            :value="item.value"
            :prepend-icon="item.icon"
            :title="item.title"
            class="nav-item mb-2"
            rounded="lg"
          />
        </v-list>
      </div>
    </v-navigation-drawer>

    <!-- 主要内容区域 -->
    <v-main class="fill-height main-content">
      <div class="content-wrapper">
        <!-- 搜索框移到这里 -->
        <div class="search-container mb-6">
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

        <v-row v-if="selectedItem[0] === 'home'">
          <!-- 星期导航条 -->
          <v-col cols="12" class="mb-6">
            <v-card
              class="weekday-tabs-card"
              elevation="0"
            >
              <v-tabs 
                v-model="selectedWeekday" 
                color="primary"
                class="weekday-tabs"
                show-arrows
                fixed-tabs
              >
                <v-tab 
                  v-for="tab in weekTabs" 
                  :key="tab.value" 
                  :value="tab.value"
                  class="text-body-2"
                >
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
            <v-progress-circular
              indeterminate
              color="primary"
              size="64"
            />
          </v-col>
          <v-col
            v-else-if="error"
            cols="12"
            class="text-center text-error"
          >
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
      </div>
    </v-main>
  </v-layout>
</template>

<style scoped>
.navigation-drawer {
  position: relative;
  border: none;
  overflow: hidden;
}

/* 毛玻璃背景 */
.glass-bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(24, 24, 24, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255, 255, 255, 0.1);
}

/* 导航内容容器 */
.nav-content {
  position: relative;
  z-index: 1;
  height: 100%;
}

.nav-list {
  background: transparent;
}

.nav-item {
  min-height: 48px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.nav-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.05);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.nav-item:hover::before {
  opacity: 1;
}

.nav-item.v-list-item--active {
  background: rgba(var(--v-theme-primary), 0.15);
  color: rgb(var(--v-theme-primary));
}

.nav-item.v-list-item--active::before {
  display: none;
}

/* 优化图标样式 */
.nav-item :deep(.v-list-item__prepend) {
  opacity: 0.8;
}

.nav-item.v-list-item--active :deep(.v-list-item__prepend) {
  opacity: 1;
}

.search-container {
  max-width: 400px;
  margin: 0 auto;
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

.main-content {
  background: rgb(16, 16, 16);
}

.content-wrapper {
  max-width: 1920px;
  margin: 0 auto;
  padding: 24px;
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

/* 移除顶部应用栏 */
:deep(.v-app-bar) {
  display: none;
}
</style>

<script lang="ts" setup>
import { ref, onMounted, computed } from 'vue'
import { useTheme } from 'vuetify'
import { fetchCalendar } from "@/api/api"
import { type Bangumi } from "@/api/model"

const selectedItem = ref(['home'])
const selectedWeekday = ref(String(new Date().getDay()))
const calendarItems = ref<Bangumi[]>([])
const loading = ref(false)
const error = ref('')

const filteredCalendarItems = computed(() => {
  if (selectedWeekday.value === '-1') {
    return calendarItems.value
  }
  return calendarItems.value.filter(item =>
    item.air_week === parseInt(selectedWeekday.value)
  )
})

const loadCalendarData = async () => {
  if (selectedItem.value[0] !== 'home') return

  loading.value = true
  error.value = ''
  try {
    calendarItems.value = await fetchCalendar()
  } catch (e) {
    error.value = e instanceof Error ? e.message : '获取数据失败'
  } finally {
    loading.value = false
  }
}

// 导航项配置
const navItems = [
  { title: '首页', icon: 'mdi-home', value: 'home' },
  { title: '电影', icon: 'mdi-movie', value: 'movies' },
  { title: '电视剧', icon: 'mdi-television-classic', value: 'tv' },
  { title: '动画', icon: 'mdi-animation', value: 'anime' },
]

// 星期标签配置
const weekTabs = [
  { label: '全部', value: '-1' },
  { label: '周一', value: '1' },
  { label: '周二', value: '2' },
  { label: '周三', value: '3' },
  { label: '周四', value: '4' },
  { label: '周五', value: '5' },
  { label: '周六', value: '6' },
  { label: '周日', value: '0' },
]

onMounted(() => {
  loadCalendarData()
})
</script>
