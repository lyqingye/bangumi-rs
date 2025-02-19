<template>
  <v-dialog v-model="dialog" max-width="900" transition="dialog-bottom-transition">
    <v-card class="tmdb-search-dialog">
      <!-- 标题栏 -->
      <v-card-title class="dialog-title">
        <div class="d-flex align-center">
          <v-icon icon="mdi-movie-search" class="me-2" />
          <span class="text-h6">搜索 TMDB 信息</span>
        </div>
        <v-btn
          icon="mdi-close"
          variant="text"
          class="close-btn"
          @click="close"
        />
      </v-card-title>

      <v-divider />

      <!-- 搜索区域 -->
      <div class="search-section pa-6">
        <v-text-field
          v-model="searchQuery"
          prepend-inner-icon="mdi-magnify"
          label="搜索番剧"
          placeholder="输入番剧名称搜索..."
          variant="outlined"
          density="comfortable"
          hide-details
          class="search-input"
          @keyup.enter="search"
        >
          <template v-slot:append>
            <v-btn
              :loading="loading"
              :disabled="!searchQuery"
              color="primary"
              class="search-btn"
              prepend-icon="mdi-magnify"
              @click="search"
            >
              搜索
            </v-btn>
          </template>
        </v-text-field>
      </div>

      <!-- 结果区域 -->
      <v-card-text class="results-container pa-6 pt-0">
        <v-fade-transition group>
          <!-- 搜索结果 -->
          <div v-if="results.length > 0" key="results" class="results-grid">
            <v-hover v-for="item in results" :key="item.id" v-slot="{ isHovering, props }">
              <v-card
                v-bind="props"
                :elevation="isHovering ? 8 : 2"
                class="result-card"
                :class="{ 'on-hover': isHovering }"
              >
                <div class="result-card-content">
                  <!-- 海报区域 -->
                  <div class="poster-wrapper">
                    <v-img
                      :src="item.poster_image_url || '/placeholder.jpg'"
                      cover
                      class="poster-image"
                    >
                      <template v-slot:placeholder>
                        <v-row class="fill-height ma-0" align="center" justify="center">
                          <v-progress-circular indeterminate color="grey-lighten-5" />
                        </v-row>
                      </template>
                    </v-img>
                  </div>

                  <!-- 标题和日期 -->
                  <div class="info-header">
                    <h3 class="title-text">{{ item.name }}</h3>
                    <div class="air-date">
                      <v-icon icon="mdi-calendar" size="small" />
                      {{ item.air_date || '未知日期' }}
                    </div>
                  </div>

                  <!-- 描述 -->
                  <div v-if="item.description" class="description">
                    {{ item.description }}
                  </div>

                  <!-- 季度列表 -->
                  <div class="seasons-list">
                    <div
                      v-for="season in item.seasons"
                      :key="season.number"
                      class="season-item"
                      :class="{ 'season-selected': selectedSeasons[item.id] === season.number }"
                      @click="selectSeason(item.id, season.number)"
                    >
                      <div class="season-radio">
                        <v-icon
                          :icon="selectedSeasons[item.id] === season.number ? 'mdi-radiobox-marked' : 'mdi-radiobox-blank'"
                          :color="selectedSeasons[item.id] === season.number ? 'primary' : undefined"
                          size="small"
                        />
                      </div>
                      <div class="season-info">
                        <div class="season-header">
                          <div class="season-number">第 {{ season.number }} 季</div>
                          <div class="episode-count">{{ season.ep_count }}集</div>
                        </div>
                        <div class="season-name">{{ season.name }}</div>
                      </div>
                    </div>
                  </div>

                  <!-- 操作按钮 -->
                  <div class="action-area">
                    <v-btn
                      color="primary"
                      variant="flat"
                      class="select-btn"
                      @click="selectTMDB(item)"
                    >
                      <v-icon icon="mdi-check" size="small" class="me-1" />
                      选择
                    </v-btn>
                  </div>
                </div>
              </v-card>
            </v-hover>
          </div>

          <!-- 无结果提示 -->
          <div v-else-if="searched" key="no-results" class="no-results">
            <v-icon icon="mdi-alert-circle-outline" size="48" color="grey-lighten-1" class="mb-4" />
            <div class="text-h6 text-grey-lighten-1">未找到相关结果</div>
            <div class="text-body-2 text-medium-emphasis mt-2">
              请尝试使用不同的关键词搜索
            </div>
          </div>
        </v-fade-transition>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'
import type { TMDBMetadata } from '@/api/model'
import { searchBangumiAtTMDB, updateBangumiMDB } from '@/api/api'
import { useSnackbar } from '../composables/useSnackbar'

const props = defineProps<{
  modelValue: boolean
  bangumiId: number
  initialQuery?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'selected'): void
}>()

const { showSnackbar } = useSnackbar()
const dialog = ref(false)
const searchQuery = ref(props.initialQuery || '')
const results = ref<TMDBMetadata[]>([])
const loading = ref(false)
const searched = ref(false)
const selectedSeasons = ref<Record<number, number>>({}) // tmdbId -> seasonNumber

// 监听 dialog 的变化
watch(
  () => props.modelValue,
  (newVal) => {
    dialog.value = newVal
    if (newVal) {
      // 打开对话框时，重置搜索框为初始查询词
      searchQuery.value = props.initialQuery || ''
    }
  }
)

// 监听内部 dialog 的变化
watch(dialog, (newVal) => {
  emit('update:modelValue', newVal)
  if (!newVal) {
    // 关闭对话框时重置状态
    results.value = []
    searched.value = false
  }
})

// 搜索功能
const search = async () => {
  if (!searchQuery.value) return

  loading.value = true
  try {
    results.value = await searchBangumiAtTMDB(searchQuery.value)
    searched.value = true
  } catch (error) {
    console.error('TMDB 搜索失败:', error)
  } finally {
    loading.value = false
  }
}

// 选择 TMDB 条目
const selectTMDB = async (item: TMDBMetadata) => {
  const selectedSeason = selectedSeasons.value[item.id]
  if (!selectedSeason && item.seasons.length > 0) {
    showSnackbar({
      text: '请选择一个季度',
      color: 'warning',
      location: 'top right',
      timeout: 3000
    })
    return
  }

  try {
    await updateBangumiMDB({
      bangumi_id: props.bangumiId,
      tmdb_id: item.id,
      season_number: selectedSeason
    })
    
    showSnackbar({
      text: 'TMDB 信息更新成功',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
    
    emit('selected')
    close()
  } catch (error) {
    console.error('更新 TMDB 信息失败:', error)
  }
}

// 添加选择季度的处理函数
const selectSeason = (tmdbId: number, seasonNumber: number) => {
  selectedSeasons.value[tmdbId] = seasonNumber
}

const close = () => {
  dialog.value = false
}
</script>

<style scoped>
.tmdb-search-dialog {
  border-radius: 16px;
  overflow: hidden;
  background: rgb(var(--v-theme-surface));
}

.dialog-title {
  background: transparent;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  position: relative;
  padding: 20px 24px;
}

.dialog-title .close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 36px;
  height: 36px;
  border-radius: 50%;
}

.dialog-title .close-btn:hover {
  background: rgba(var(--v-theme-on-surface), 0.04);
}

.search-section {
  position: sticky;
  top: 0;
  z-index: 1;
  background: rgb(var(--v-theme-surface));
  padding: 24px 32px;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}

.search-input {
  max-width: 100%;
}

.search-input :deep(.v-field) {
  border-radius: 12px;
  background: rgba(var(--v-theme-surface-variant), 0.06);
}

.search-input :deep(.v-field.v-field--focused) {
  background: rgba(var(--v-theme-surface-variant), 0.1);
}

.search-btn {
  margin-left: 12px;
  height: 48px;
  min-width: 120px;
  border-radius: 12px;
  font-weight: 500;
  letter-spacing: 0.5px;
}

.results-container {
  max-height: 75vh;
  overflow-y: auto;
  padding: 24px 32px;
  background: rgba(var(--v-theme-surface-variant), 0.02);
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(700px, 1fr));
  gap: 24px;
}

.result-card {
  height: auto;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  border-radius: 16px;
  overflow: hidden;
}

.result-card.on-hover {
  transform: translateY(-2px);
  border-color: rgba(var(--v-theme-primary), 0.2);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
}

.result-card-content {
  display: grid;
  grid-template-areas: 
    "poster header"
    "poster description"
    "poster seasons"
    "poster action";
  grid-template-columns: auto 1fr;
  grid-template-rows: auto 80px auto auto;
  gap: 16px;
  padding: 24px;
}

.poster-wrapper {
  grid-area: poster;
  width: 200px;
  height: 300px;
  position: relative;
  margin-right: 24px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  align-self: start;
  margin-bottom: auto;
}

.poster-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.result-card.on-hover .poster-image {
  transform: scale(1.05);
}

.info-header {
  grid-area: header;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  padding: 4px 0;
}

.title-text {
  font-size: 1.6rem;
  line-height: 1.3;
  font-weight: 600;
  margin-bottom: 12px;
  color: rgba(var(--v-theme-on-surface), 0.87);
}

.air-date {
  font-size: 0.95rem;
  color: rgba(var(--v-theme-on-surface), 0.6);
  display: flex;
  align-items: center;
  gap: 8px;
}

.description {
  grid-area: description;
  font-size: 1rem;
  color: rgba(var(--v-theme-on-surface), 0.7);
  line-height: 1.6;
  margin-top: -8px;
  height: 80px;
  overflow-y: auto;
  padding-right: 8px;
}

.description::-webkit-scrollbar {
  width: 4px;
}

.description::-webkit-scrollbar-track {
  background: transparent;
}

.description::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 2px;
}

.description::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-on-surface), 0.2);
}

.seasons-list {
  grid-area: seasons;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}

.season-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  background: transparent;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  min-width: 180px;
  max-width: 220px;
}

.season-item:hover {
  background: rgba(var(--v-theme-primary), 0.04);
  border-color: rgba(var(--v-theme-primary), 0.1);
}

.season-item.season-selected {
  background: rgba(var(--v-theme-primary), 0.06);
  border-color: rgba(var(--v-theme-primary), 0.15);
}

.season-radio {
  display: flex;
  align-items: center;
}

.season-radio :deep(.v-icon) {
  font-size: 18px;
}

.season-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.season-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.season-number {
  color: rgb(var(--v-theme-primary));
  font-weight: 600;
  font-size: 0.9rem;
}

.episode-count {
  color: rgba(var(--v-theme-on-surface), 0.5);
  font-size: 0.75rem;
  background: rgba(var(--v-theme-surface-variant), 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.season-name {
  font-size: 0.85rem;
  color: rgba(var(--v-theme-on-surface), 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-area {
  grid-area: action;
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

.select-btn {
  min-width: 100px;
  height: 32px;
  font-size: 0.85rem;
  font-weight: 500;
  letter-spacing: 0.3px;
  border-radius: 6px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  background: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
  padding: 0 16px;
}

.select-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(var(--v-theme-primary), 0.2);
}

.select-btn :deep(.v-btn__content) {
  gap: 4px;
}

.select-btn :deep(.v-icon) {
  font-size: 16px;
}

/* 自定义 seasons-list 的滚动条样式 */
.seasons-list::-webkit-scrollbar {
  width: 4px;
}

.seasons-list::-webkit-scrollbar-track {
  background: transparent;
}

.seasons-list::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-primary), 0.1);
  border-radius: 2px;
}

.seasons-list::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-primary), 0.2);
}

.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  text-align: center;
  background: rgba(var(--v-theme-surface-variant), 0.02);
  border-radius: 16px;
  border: 1px dashed rgba(var(--v-theme-on-surface), 0.1);
}

.no-results .v-icon {
  color: rgba(var(--v-theme-on-surface), 0.2);
  margin-bottom: 24px;
}

.no-results .text-h6 {
  color: rgba(var(--v-theme-on-surface), 0.6);
  font-weight: 500;
  margin-bottom: 8px;
}

.no-results .text-body-2 {
  color: rgba(var(--v-theme-on-surface), 0.5);
}

/* 自定义滚动条样式 */
.results-container::-webkit-scrollbar {
  width: 8px;
}

.results-container::-webkit-scrollbar-track {
  background: transparent;
}

.results-container::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 4px;
}

.results-container::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-on-surface), 0.15);
}
</style> 