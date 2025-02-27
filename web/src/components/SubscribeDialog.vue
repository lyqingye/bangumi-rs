<script setup lang="ts">
import { ref, watch } from 'vue'
import type { SubscribeParams } from '../api/model'
import { SubscribeStatus } from '../api/model'

const props = defineProps<{
  modelValue: boolean
  bangumiId: number
  currentStatus: SubscribeStatus
  releaseGroups: string[]
  currentSubscribeSettings?: SubscribeParams
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'subscribe', params: SubscribeParams): void
}>()

// 添加新字幕组相关变量
const showAddReleaseGroupDialog = ref(false)
const newReleaseGroup = ref('')
const customReleaseGroups = ref<string[]>([])
const allReleaseGroups = ref<string[]>([...props.releaseGroups])

// 合并原有字幕组和自定义字幕组
watch(() => props.releaseGroups, (newVal) => {
  allReleaseGroups.value = [...newVal, ...customReleaseGroups.value]
}, { immediate: true })

// 添加新字幕组
function addNewReleaseGroup() {
  if (newReleaseGroup.value && newReleaseGroup.value.trim() !== '') {
    const trimmedGroup = newReleaseGroup.value.trim()
    if (!allReleaseGroups.value.includes(trimmedGroup)) {
      customReleaseGroups.value.push(trimmedGroup)
      allReleaseGroups.value.push(trimmedGroup)
      // 自动选中新添加的字幕组
      formData.value.release_group_filter.push(trimmedGroup)
    }
    newReleaseGroup.value = ''
  }
  showAddReleaseGroupDialog.value = false
}

const formData = ref({
  status: SubscribeStatus.Subscribed,
  start_episode_number: props.currentSubscribeSettings?.start_episode_number ?? 1,
  resolution_filter: props.currentSubscribeSettings?.resolution_filter
    ? props.currentSubscribeSettings.resolution_filter.split(',')
    : [],
  language_filter: props.currentSubscribeSettings?.language_filter
    ? props.currentSubscribeSettings.language_filter.split(',')
    : [],
  release_group_filter: props.currentSubscribeSettings?.release_group_filter
    ? props.currentSubscribeSettings.release_group_filter.split(',')
    : [],
  collector_interval: props.currentSubscribeSettings?.collector_interval
    ? Math.floor(props.currentSubscribeSettings.collector_interval / 60)
    : 30,
  metadata_interval: props.currentSubscribeSettings?.metadata_interval
    ? Math.floor(props.currentSubscribeSettings.metadata_interval / 60)
    : 60
})

interface Resolution {
  text: string
  value: string
}

const resolutions: Resolution[] = [
  { text: '2160P', value: '2160P' },
  { text: '1440P', value: '1440P' },
  { text: '1080P', value: '1080P' },
  { text: '720P', value: '720P' }
]

interface Language {
  text: string
  value: string
}

const languages: Language[] = [
  { text: '简体中文', value: 'CHS' },
  { text: '繁体中文', value: 'CHT' },
  { text: '日语', value: 'JPN' },
  { text: '英语', value: 'ENG' }
]

function arrayToString(arr: string[]): string {
  if (!arr || arr.length === 0) return ''
  return arr.join(',')
}

function onSubmit() {
  const params: SubscribeParams = {
    status: formData.value.status,
    start_episode_number: formData.value.start_episode_number,
    resolution_filter: arrayToString(formData.value.resolution_filter),
    language_filter: arrayToString(formData.value.language_filter),
    release_group_filter: arrayToString(formData.value.release_group_filter),
    collector_interval: formData.value.collector_interval
      ? formData.value.collector_interval * 60
      : undefined,
    metadata_interval: formData.value.metadata_interval
      ? formData.value.metadata_interval * 60
      : undefined
  }
  emit('subscribe', params)
  emit('update:modelValue', false)
}

function onCancel() {
  emit('update:modelValue', false)
}

function unsubscribe() {
  emit('subscribe', { status: SubscribeStatus.None })
  emit('update:modelValue', false)
}
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="400"
    class="subscribe-dialog"
  >
    <v-card class="subscribe-card">
      <v-card-title class="dialog-title py-3 px-4">
        <v-icon icon="mdi-rss" class="me-2" color="primary" size="20" />
        订阅设置
      </v-card-title>

      <v-divider />

      <v-card-text class="pa-4">
        <v-form @submit.prevent="onSubmit" class="form-content">
          <div class="section-title">基本设置</div>
          <div class="input-group">
            <div class="input-label">
              <v-icon icon="mdi-numeric" color="primary" size="16" class="me-2" />
              <span>起始集数</span>
            </div>
            <v-text-field
              v-model.number="formData.start_episode_number"
              type="number"
              min="1"
              density="compact"
              variant="outlined"
              hide-details
              class="input-field"
            />
          </div>

          <div class="section-title">过滤器设置</div>
          <div class="input-group">
            <div class="input-label">
              <v-icon icon="mdi-video" color="primary" size="16" class="me-2" />
              <span>分辨率</span>
            </div>
            <v-select
              v-model="formData.resolution_filter"
              :items="resolutions"
              item-title="text"
              item-value="value"
              density="compact"
              variant="outlined"
              hide-details
              multiple
              chips
              closable-chips
              class="input-field"
            >
              <template v-slot:chip="{ props, item }">
                <v-chip v-bind="props" :text="String(item.raw)" size="x-small" label />
              </template>
            </v-select>
          </div>

          <div class="input-group">
            <div class="input-label">
              <v-icon icon="mdi-translate" color="primary" size="16" class="me-2" />
              <span>语言</span>
            </div>
            <v-select
              v-model="formData.language_filter"
              :items="languages"
              item-title="text"
              item-value="value"
              density="compact"
              variant="outlined"
              hide-details
              multiple
              chips
              closable-chips
              class="input-field"
            >
              <template v-slot:chip="{ props, item }">
                <v-chip v-bind="props" :text="String(item.raw)" size="x-small" label />
              </template>
            </v-select>
          </div>

          <div class="input-group">
            <div class="input-label">
              <v-icon icon="mdi-account-group" color="primary" size="16" class="me-2" />
              <span>字幕组</span>
            </div>
            <v-select
              v-model="formData.release_group_filter"
              :items="allReleaseGroups"
              density="compact"
              variant="outlined"
              hide-details
              multiple
              chips
              closable-chips
              class="input-field"
            >
              <template v-slot:chip="{ props, item }">
                <v-chip v-bind="props" :text="String(item.raw)" size="x-small" label />
              </template>
              <template v-slot:append-item>
                <v-divider class="mb-2"></v-divider>
                <v-list-item
                  density="compact"
                  @click="showAddReleaseGroupDialog = true"
                  class="add-item-btn"
                >
                  <v-list-item-title>
                    <v-icon icon="mdi-plus-circle" size="small" class="me-2" color="primary" />
                    添加新字幕组
                  </v-list-item-title>
                </v-list-item>
              </template>
            </v-select>
          </div>

          <div class="section-title">高级设置</div>
          <div class="input-group">
            <div class="input-label">
              <v-icon icon="mdi-refresh" color="primary" size="16" class="me-2" />
              <span>收集器间隔 (分钟)</span>
            </div>
            <v-text-field
              v-model="formData.collector_interval"
              type="number"
              min="1"
              density="compact"
              variant="outlined"
              hide-details
              class="input-field"
              placeholder="默认 30 分钟"
            >
            </v-text-field>
          </div>

          <div class="input-group mb-0">
            <div class="input-label">
              <v-icon icon="mdi-update" color="primary" size="16" class="me-2" />
              <span>元数据更新间隔 (分钟)</span>
            </div>
            <v-text-field
              v-model="formData.metadata_interval"
              type="number"
              min="1"
              density="compact"
              variant="outlined"
              hide-details
              class="input-field"
              placeholder="默认 60 分钟"
            >
            </v-text-field>
          </div>
        </v-form>
      </v-card-text>

      <v-divider />

      <v-card-actions class="pa-3">
        <v-btn
          v-if="currentStatus === SubscribeStatus.Subscribed"
          color="error"
          variant="tonal"
          density="comfortable"
          prepend-icon="mdi-close-circle"
          class="unsubscribe-btn"
          size="small"
          @click="unsubscribe"
        >
          取消订阅
        </v-btn>
        <v-spacer />
        <v-btn color="grey" variant="text" size="small" class="me-2" @click="onCancel">
          取消
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          size="small"
          :prepend-icon="
            currentStatus === SubscribeStatus.Subscribed ? 'mdi-check-circle' : 'mdi-plus-circle'
          "
          @click="onSubmit"
        >
          {{ currentStatus === SubscribeStatus.Subscribed ? '更新订阅' : '订阅' }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- 添加新字幕组对话框 -->
  <v-dialog v-model="showAddReleaseGroupDialog" max-width="300" class="add-group-dialog">
    <v-card>
      <v-card-title class="text-subtitle-1 py-3 px-4">
        <v-icon icon="mdi-plus-circle" class="me-2" color="primary" size="20" />
        添加新字幕组
      </v-card-title>
      <v-divider></v-divider>
      <v-card-text class="pt-4">
        <v-text-field
          v-model="newReleaseGroup"
          label="字幕组名称"
          variant="outlined"
          density="compact"
          hide-details
          autofocus
          @keyup.enter="addNewReleaseGroup"
        ></v-text-field>
      </v-card-text>
      <v-card-actions class="pa-3">
        <v-spacer></v-spacer>
        <v-btn color="grey" variant="text" size="small" @click="showAddReleaseGroupDialog = false">
          取消
        </v-btn>
        <v-btn color="primary" variant="tonal" size="small" @click="addNewReleaseGroup">
          添加
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.subscribe-dialog {
  backdrop-filter: blur(10px);
}

.subscribe-card {
  border-radius: 8px;
  overflow: hidden;
  background: rgba(30, 30, 30, 0.95);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.dialog-title {
  font-size: 1.1rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  color: rgba(255, 255, 255, 0.9);
}

.form-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 0.85rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.6);
  margin-top: 8px;
  margin-bottom: 4px;
  padding-bottom: 4px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.input-group {
  margin-bottom: 0;
}

.input-label {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.7);
}

.input-field {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 6px;
}

:deep(.v-field) {
  border-radius: 6px !important;
  transition: all 0.3s ease;
  font-size: 0.9rem;
}

:deep(.v-field__input) {
  min-height: 32px !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

:deep(.v-field:hover) {
  background: rgba(255, 255, 255, 0.08);
}

:deep(.v-field.v-field--focused) {
  background: rgba(255, 255, 255, 0.1);
}

:deep(.v-btn) {
  text-transform: none;
  font-weight: 500;
  letter-spacing: 0.3px;
  min-width: 64px;
}

.unsubscribe-btn {
  transition: all 0.3s ease;
}

.unsubscribe-btn:hover {
  background: rgba(var(--v-theme-error), 0.2) !important;
}

:deep(.v-card-actions) {
  background: rgba(0, 0, 0, 0.2);
}

:deep(.v-divider) {
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.v-select__selection) {
  margin-top: 2px;
  margin-bottom: 2px;
}

:deep(.v-chip.v-chip--size-x-small) {
  --v-chip-height: 20px;
  font-size: 0.75rem;
}

:deep(.v-chip.v-chip--label) {
  border-radius: 4px;
}

:deep(.v-chip__close) {
  opacity: 0.7;
  font-size: 14px;
}

:deep(.v-chip__close:hover) {
  opacity: 1;
}

.add-item-btn {
  transition: background-color 0.2s ease;
}

.add-item-btn:hover {
  background-color: rgba(var(--v-theme-primary), 0.05);
}

.add-group-dialog :deep(.v-card) {
  border-radius: 8px;
  overflow: hidden;
  background: rgba(30, 30, 30, 0.95);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.add-group-dialog :deep(.v-card-title) {
  font-size: 1rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  color: rgba(255, 255, 255, 0.9);
}

.add-group-dialog :deep(.v-text-field) {
  margin-top: 8px;
}
</style>
