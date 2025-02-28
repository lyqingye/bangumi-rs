<template>
  <v-card class="media-card" elevation="0" @click="goToDetail">
    <div class="card-image-wrapper">
      <!-- 订阅状态丝带 -->
      <div class="ribbon-wrapper" v-if="item.subscribe_status === SubscribeStatus.Subscribed">
        <div class="ribbon">
          <v-icon size="16" class="me-1">mdi-check-circle</v-icon>
          已追番
        </div>
      </div>
      <div class="ribbon-wrapper" v-if="item.subscribe_status === SubscribeStatus.Downloaded">
        <div class="ribbon">
          <v-icon size="16" class="me-1">mdi-check-circle</v-icon>
          已完成
        </div>
      </div>

      <v-img :src="item.poster_image_url" height="360" cover class="card-image">
        <template v-slot:placeholder>
          <v-row class="fill-height ma-0" align="center" justify="center">
            <v-progress-circular indeterminate color="grey-lighten-5" />
          </v-row>
        </template>

        <!-- 悬浮操作按钮 -->
        <div class="image-overlay">
          <div class="action-buttons">
            <!-- 刷新按钮 -->
            <v-btn
              variant="text"
              size="small"
              class="action-btn"
              @click.stop="handleRefresh"
              :loading="isRefreshing"
            >
              <v-icon icon="mdi-refresh" size="20" />
              <v-tooltip activator="parent" location="top">刷新元数据</v-tooltip>
            </v-btn>
            <!-- 订阅按钮 -->
            <v-btn variant="text" size="small" class="action-btn" @click.stop="toggleSubscribe">
              <v-icon
                :icon="isSubscribed ? 'mdi-heart' : 'mdi-heart-outline'"
                :color="isSubscribed ? 'error' : undefined"
                size="20"
              />
              <v-tooltip activator="parent" location="top">
                {{ isSubscribed ? '已追番' : '追番' }}
              </v-tooltip>
            </v-btn>

            <!-- TMDB 搜索按钮 -->
            <v-btn variant="text" size="small" class="action-btn" @click.stop="showTMDBSearch">
              <v-icon icon="mdi-magnify" size="20" />
              <v-tooltip activator="parent" location="top">
                搜索 TMDB
              </v-tooltip>
            </v-btn>
          </div>
        </div>
      </v-img>
    </div>

    <v-card-item class="pa-4">
      <!-- 标题和评分 -->
      <div class="d-flex justify-space-between align-start mb-2">
        <div class="text-subtitle-1 font-weight-medium text-truncate me-2">
          {{ item.name }}
        </div>
        <div class="rating-badge" v-if="item.rating">
          {{ item.rating.toFixed(1) }}
        </div>
      </div>

      <!-- 信息行 -->
      <div class="info-row">
        <div class="d-flex align-center">
          <v-icon size="16" icon="mdi-calendar" class="me-1" />
          <span class="text-caption text-medium-emphasis">
            {{ item.air_date?.split('T')[0] }}
          </span>
        </div>
        <div class="d-flex align-center ms-3">
          <v-icon size="16" icon="mdi-clock" class="me-1" />
          <span class="text-caption text-medium-emphasis">
            {{ getWeekday(item.air_week) }}
          </span>
        </div>
      </div>
    </v-card-item>

    <!-- 订阅对话框 -->
    <SubscribeDialog
      v-model="showSubscribeDialog"
      :bangumi-id="item.id"
      :current-status="item.subscribe_status || SubscribeStatus.None"
      :release-groups="[]"
      :current-subscribe-settings="currentSubscribeSettings"
      @subscribe="handleSubscribe"
    />

    <!-- TMDB 搜索对话框 -->
    <TMDBSearchDialog
      v-model="showTMDBSearchDialog"
      :bangumi-id="item.id"
      :initial-query="item.name"
      @selected="handleTMDBSelected"
    />

    <!-- 刷新对话框 -->
    <RefreshDialog
      v-model="showRefreshDialog"
      @confirm="handleRefreshConfirm"
    />
  </v-card>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { SubscribeStatus, type Bangumi, type SubscribeParams } from '@/api/model'
import { subscribeBangumi, refreshBangumi } from '@/api/api'
import { useSnackbar } from '../composables/useSnackbar'
import SubscribeDialog from '../components/SubscribeDialog.vue'
import TMDBSearchDialog from '../components/TMDBSearchDialog.vue'
import RefreshDialog from '../components/RefreshDialog.vue'

const props = defineProps<{
  item: Bangumi
}>()

const router = useRouter()
const ratingValue = computed(() => props.item.rating / 2)
const { showSnackbar } = useSnackbar()

// 订阅状态
const isSubscribed = ref(props.item.subscribe_status === SubscribeStatus.Subscribed)
const showSubscribeDialog = ref(false)

// 添加刷新状态
const isRefreshing = ref(false)

// TMDB 搜索相关
const showTMDBSearchDialog = ref(false)

// 添加刷新对话框状态
const showRefreshDialog = ref(false)

// 切换订阅状态
const toggleSubscribe = (event: Event) => {
  event.stopPropagation()
  if (!props.item) return
  showSubscribeDialog.value = true
}

const currentSubscribeSettings = computed(() => {
  if (!props.item || props.item.subscribe_status !== SubscribeStatus.Subscribed) {
    return undefined
  }

  return {
    start_episode_number: props.item.start_episode_number ?? undefined,
    resolution_filter: props.item.resolution_filter ?? undefined,
    language_filter: props.item.language_filter ?? undefined,
    release_group_filter: props.item.release_group_filter ?? undefined,
    status: props.item.subscribe_status ?? undefined,
    enforce_torrent_release_after_broadcast: props.item.enforce_torrent_release_after_broadcast ?? undefined
  }
})

// 处理订阅
const handleSubscribe = async (params: SubscribeParams) => {
  try {
    // 调用订阅 API
    await subscribeBangumi(props.item.id, params)
    // 更新本地状态
    isSubscribed.value = params.status === SubscribeStatus.Subscribed
    props.item.subscribe_status = params.status
    // 显示成功提示
    showSnackbar({
      text: params.status === SubscribeStatus.Subscribed ? '订阅成功' : '取消订阅成功',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
  } catch (error) {
    console.error('订阅操作失败:', error)
  }
}

// 处理刷新操作
const handleRefresh = async (event: Event) => {
  event.stopPropagation()
  if (!props.item || isRefreshing.value) return
  showRefreshDialog.value = true
}

// 处理刷新确认
const handleRefreshConfirm = async (force: boolean) => {
  try {
    isRefreshing.value = true
    // 调用刷新 API，传入 force 参数
    await refreshBangumi(props.item.id, force)
    // 显示成功提示
    showSnackbar({
      text: '已经加入刷新队列',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
  } catch (error) {
    console.error('刷新失败:', error)
    // 显示错误提示
    showSnackbar({
      text: '刷新失败',
      color: 'error',
      location: 'top right',
      timeout: 3000
    })
  } finally {
    isRefreshing.value = false
    showRefreshDialog.value = false
  }
}

const getWeekday = (day: number) => {
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  return weekdays[day]
}

const goToDetail = () => {
  router.push({
    path: `/detail/${props.item.id}`
  })
}

const showTMDBSearch = (event: Event) => {
  event.stopPropagation()
  showTMDBSearchDialog.value = true
}

const handleTMDBSelected = () => {
  // 可以在这里添加刷新卡片信息的逻辑
}
</script>

<style scoped>
.media-card {
  background: rgba(32, 32, 32, 0.9);
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.2s ease;
  cursor: pointer;
}

.media-card:hover {
  transform: translateY(-4px);
}

.card-image-wrapper {
  position: relative;
  overflow: hidden;
}

.card-image {
  transition: transform 0.3s ease;
}

.media-card:hover .card-image {
  transform: scale(1.05);
}

.image-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.2) 0%, rgba(0, 0, 0, 0.6) 100%);
  opacity: 0;
  transition: opacity 0.3s ease;
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  padding: 16px;
}

.media-card:hover .image-overlay {
  opacity: 1;
}

.action-buttons {
  display: flex;
  gap: 8px;
}

.action-btn {
  color: white;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  width: 36px !important;
  height: 36px !important;
  min-width: 36px !important;
  border-radius: 50% !important;
  padding: 0 !important;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.7);
  transform: scale(1.1);
}

.action-btn :deep(.v-icon.text-error) {
  color: rgb(var(--v-theme-error));
}

.action-btn :deep(.v-btn__content) {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.rating-badge {
  background: rgba(255, 180, 0, 0.9);
  color: black;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 600;
}

.info-row {
  display: flex;
  align-items: center;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.75rem;
}

:deep(.v-card-item) {
  padding: 12px 16px;
}

/* 丝带容器 */
.ribbon-wrapper {
  width: 150px;
  height: 150px;
  overflow: hidden;
  position: absolute;
  top: -10px;
  right: -10px;
  z-index: 2;
}

/* 丝带样式 */
.ribbon {
  font-size: 0.75rem;
  font-weight: 500;
  color: white;
  text-align: center;
  transform: rotate(45deg);
  position: relative;
  padding: 6px 0;
  left: -10px;
  top: 30px;
  width: 200px;
  background: rgba(var(--v-theme-primary), 0.9);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

/* 丝带两端的装饰 */
.ribbon::before,
.ribbon::after {
  content: '';
  position: absolute;
  border-top: 3px solid rgba(var(--v-theme-primary), 1);
  border-left: 3px solid transparent;
  border-right: 3px solid transparent;
  bottom: -3px;
}

.ribbon::before {
  left: 0;
}

.ribbon::after {
  right: 0;
}

/* 丝带悬停效果 */
.ribbon:hover {
  background: rgba(var(--v-theme-primary), 1);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
}

/* 调整图标垂直对齐 */
.ribbon .v-icon {
  margin-top: -2px;
}
</style>
