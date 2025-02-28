<template>
  <div class="bangumi-list-content">
    <div class="d-flex align-center justify-space-between mb-6">
      <h1 class="text-h5 font-weight-bold">番剧列表</h1>
      <div class="d-flex align-center">
        <!-- 年份选择器 -->
        <v-select
          v-model="selectedYear"
          :items="yearOptions"
          label="年份"
          variant="outlined"
          density="compact"
          class="me-2 year-select"
          bg-color="rgba(48, 48, 48, 0.95)"
          item-color="white"
          hide-details
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
          class="me-2 season-select"
          hide-details
          bg-color="rgba(48, 48, 48, 0.95)"
          item-color="white"
        ></v-select>
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
import { fetchBangumiList, fetchCalendarSeason } from '@/api/api'
import { SubscribeStatus, type Bangumi, type QueryBangumiParams } from '@/api/model'
import MediaCard from '@/components/MediaCard.vue'

// 分页参数
const pageSize = 12
const currentPage = ref(1)
const total = ref(0)
const totalPages = computed(() => Math.ceil(total.value / pageSize))

// 定义选项类型
interface SelectOption<T> {
  title: string;
  value: T;
}

// 查询参数
const selectedYear = ref<number | null>(null)
const selectedSeason = ref<string | null>(null)
const selectedStatus = ref<SubscribeStatus | null>(null)
const availableSeasons = ref<string[]>([])

// 数据状态
const bangumis = ref<Bangumi[]>([])
const loading = ref(false)
const error = ref('')

// 年份选项
const yearOptions = computed<SelectOption<number | null>[]>(() => {
  const options: SelectOption<number | null>[] = [{ title: '全部年份', value: null }];
  
  // 获取当前年份
  const currentYear = new Date().getFullYear();
  
  // 从2015年到当前年份
  for (let year = currentYear; year >= 2015; year--) {
    options.push({
      title: `${year}年`,
      value: year
    });
  }
  
  return options;
});

// 季节选项 - 改为计算属性，根据selectedYear动态生成
const seasonOptions = computed<SelectOption<string | null>[]>(() => {
  // 基础季节选项
  const baseOptions: SelectOption<string | null>[] = [
    { title: '冬季番组', value: '冬季番组' },
    { title: '春季番组', value: '春季番组' },
    { title: '夏季番组', value: '夏季番组' },
    { title: '秋季番组', value: '秋季番组' }
  ];
  
  // 如果没有选择具体年份，则只能选择"全部季节"
  if (selectedYear.value === null) {
    return [{ title: '全部季节', value: null }];
  }
  
  // 如果选择了具体年份，则只能选择具体季节
  return baseOptions;
});

// 订阅状态选项
const statusOptions: SelectOption<SubscribeStatus | null>[] = [
  { title: '全部状态', value: null },
  { title: '已订阅', value: SubscribeStatus.Subscribed },
  { title: '未订阅', value: SubscribeStatus.None },
];

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
      calendar_season: getCalendarSeason()
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

// 获取当前选择的季度值
function getCalendarSeason(): string | undefined {
  if (!selectedYear.value && !selectedSeason.value) return undefined;
  if (!selectedYear.value) return selectedSeason.value || undefined;
  if (!selectedSeason.value) return String(selectedYear.value);
  return `${selectedYear.value} ${selectedSeason.value}`;
}

// 处理页码变化
const handlePageChange = () => {
  loadBangumiList()
}

// 监听年份变化
watch(selectedYear, (newYear) => {
  if (newYear === null) {
    // 如果选择了"全部年份"，则季节必须是"全部季节"
    selectedSeason.value = null;
  } else {
    // 如果选择了具体年份，但季节是"全部季节"，则自动选择第一个具体季节
    if (selectedSeason.value === null) {
      selectedSeason.value = '冬季番组';
    }
  }
  
  currentPage.value = 1; // 重置到第一页
  loadBangumiList();
});

// 监听季节变化
watch(selectedSeason, () => {
  currentPage.value = 1; // 重置到第一页
  loadBangumiList();
});

// 监听订阅状态变化
watch(selectedStatus, () => {
  currentPage.value = 1; // 重置到第一页
  loadBangumiList();
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
      }
    }
  } catch (e) {
    console.error('获取最新季度信息失败:', e);
  }
}

// 组件挂载时加载数据
onMounted(async () => {
  // 先获取最新季度信息
  await fetchLatestSeason();
  // 然后加载番剧列表
  loadBangumiList();
});
</script>

<style scoped>
.bangumi-list-content {
  max-width: 1920px;
  margin: 0 auto;
  padding: 24px;
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
</style> 