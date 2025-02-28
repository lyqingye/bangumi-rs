<template>
  <div class="bangumi-list-content">
    <div class="d-flex align-center justify-space-between mb-6">
      <h1 class="text-h5 font-weight-bold">番剧列表</h1>
      <div class="d-flex">
        <!-- 季度选择器 -->
        <v-select
          v-model="selectedSeason"
          :items="seasonOptions"
          label="季度"
          variant="outlined"
          density="compact"
          hide-details
          class="me-4"
          style="width: 150px"
        ></v-select>
        
        <!-- 订阅状态过滤器 -->
        <v-select
          v-model="selectedStatus"
          :items="statusOptions"
          label="订阅状态"
          variant="outlined"
          density="compact"
          hide-details
          style="width: 150px"
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
import { fetchBangumiList } from '@/api/api'
import { SubscribeStatus, type Bangumi, type QueryBangumiParams } from '@/api/model'
import MediaCard from '@/components/MediaCard.vue'

// 分页参数
const pageSize = 12
const currentPage = ref(1)
const total = ref(0)
const totalPages = computed(() => Math.ceil(total.value / pageSize))

// 查询参数
const selectedSeason = ref<string | null>(null)
const selectedStatus = ref<SubscribeStatus | null>(null)

// 数据状态
const bangumis = ref<Bangumi[]>([])
const loading = ref(false)
const error = ref('')

// 季度选项
const seasonOptions = [
  { title: '全部季度', value: null },
  { title: '2024年1月', value: '2024年1月' },
  { title: '2023年10月', value: '2023年10月' },
  { title: '2023年7月', value: '2023年7月' },
  { title: '2023年4月', value: '2023年4月' },
  { title: '2023年1月', value: '2023年1月' }
]

// 订阅状态选项
const statusOptions = [
  { title: '全部状态', value: null },
  { title: '已订阅', value: SubscribeStatus.Subscribed },
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
      calendar_season: selectedSeason.value || undefined
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

// 监听过滤条件变化
watch([selectedSeason, selectedStatus], () => {
  currentPage.value = 1 // 重置到第一页
  loadBangumiList()
})

// 组件挂载时加载数据
onMounted(() => {
  loadBangumiList()
})
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
</style> 