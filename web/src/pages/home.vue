<template>
  <div class="home-content">
    <!-- 搜索框 -->
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

    <v-row>
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
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { fetchCalendar } from "@/api/api"
import { type Bangumi } from "@/api/model"
import MediaCard from '@/components/MediaCard.vue'

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

<style scoped>
.home-content {
  max-width: 1920px;
  margin: 0 auto;
  padding: 24px;
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
</style> 