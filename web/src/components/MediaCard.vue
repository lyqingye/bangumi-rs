<template>
  <v-card 
    class="media-card" 
    elevation="0"
    @click="goToDetail"
  >
    <div class="card-image-wrapper">
      <!-- 订阅状态丝带 -->
      <div class="ribbon-wrapper" v-if="item.subscribe_status === SubscribeStatus.Subscribed">
        <div class="ribbon">
          <v-icon size="16" class="me-1">mdi-check-circle</v-icon>
          已追番
        </div>
      </div>

      <v-img
        :src="item.poster_image_url"
        height="360"
        cover
        class="card-image"
      >
        <template v-slot:placeholder>
          <v-row
            class="fill-height ma-0"
            align="center"
            justify="center"
          >
            <v-progress-circular
              indeterminate
              color="grey-lighten-5"
            />
          </v-row>
        </template>

        <!-- 悬浮操作按钮 -->
        <div class="image-overlay">
          <div class="action-buttons">
            <!-- 订阅按钮 -->
            <v-btn
              variant="text"
              size="small"
              class="action-btn"
              @click.stop="toggleSubscribe"
            >
              <v-icon
                :icon="isSubscribed ? 'mdi-heart' : 'mdi-heart-outline'"
                :color="isSubscribed ? 'error' : undefined"
                size="20"
              />
              <v-tooltip activator="parent" location="top">
                {{ isSubscribed ? '已追番' : '追番' }}
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
  </v-card>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { SubscribeStatus, type Bangumi } from '@/api/model'
import { subscribeBangumi } from '@/api/api'

const props = defineProps<{
  item: Bangumi
}>()

const router = useRouter()
const ratingValue = computed(() => props.item.rating / 2)

// 订阅状态
const isSubscribed = ref(props.item.subscribe_status === SubscribeStatus.Subscribed)

// 切换订阅状态
const toggleSubscribe = async () => {
  try {
    const newStatus = isSubscribed.value ? SubscribeStatus.None : SubscribeStatus.Subscribed
    await subscribeBangumi(props.item.id, newStatus)
    
    // 更新本地状态
    isSubscribed.value = !isSubscribed.value
    props.item.subscribe_status = newStatus
  } catch (error) {
    console.error('订阅操作失败:', error)
    // TODO: 添加错误提示
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
  background: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.2) 0%,
    rgba(0, 0, 0, 0.6) 100%
  );
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
