<template>
  <v-dialog
    v-model="dialog"
    max-width="550px"
    content-class="file-selection-dialog"
    :scrim="true"
    :persistent="true"
    transition="dialog-transition"
    @update:model-value="handleDialogChange"
  >
    <v-card bg-color="background" class="rounded-lg">
      <v-card-item class="dialog-header px-6 py-4">
        <template #prepend>
          <v-icon icon="mdi-play-circle" color="primary" />
        </template>
        <v-card-title>选择播放文件</v-card-title>
        <template #append>
          <v-btn icon="mdi-close" variant="text" @click="handleClose" />
        </template>
      </v-card-item>

      <v-divider></v-divider>

      <v-card-text class="pa-6">
        <div v-if="localFiles.length === 0" class="text-center my-6 text-medium-emphasis">
          <v-icon icon="mdi-alert-circle-outline" class="mb-2" size="36" color="warning" />
          <div>没有可播放的文件</div>
        </div>
        
        <div v-else>
          <!-- IINA播放器提示信息 -->
          <v-alert
            v-if="playerType === 'iina' && subtitleFiles.length > 0"
            color="warning"
            variant="tonal"
            class="mb-5"
            border="start"
            density="comfortable"
            icon="mdi-alert-circle"
            elevation="1"
          >
            <div class="font-weight-medium mb-1">IINA播放器不支持外挂字幕</div>
            <div class="text-caption">请下载字幕文件后，在播放器中手动加载</div>
          </v-alert>
          
          <!-- 视频文件列表 -->
          <div v-if="videoFiles.length > 0" class="mb-6">
            <div class="section-title mb-3 d-flex align-center">
              <v-icon icon="mdi-video" class="mr-2" color="primary" />
              <span class="text-h6">视频文件</span>
              <v-chip class="ml-2" size="small" color="primary" variant="flat">{{ videoFiles.length }}</v-chip>
            </div>
            <v-card class="file-list-card">
              <v-list class="file-list py-0" bg-color="transparent">
                <v-list-item
                  v-for="file in videoFiles"
                  :key="`video-${file.file_id}-${isVideoSelected(file)}`"
                  @click="handleVideoSelect(file)"
                  :class="{ 'selected-item': isVideoSelected(file) }"
                  class="file-item"
                  rounded="lg"
                  lines="two"
                >
                  <template #prepend>
                    <v-avatar color="primary" variant="tonal" size="36" class="file-icon">
                      <v-icon icon="mdi-file-video" size="16" />
                    </v-avatar>
                  </template>
                  <template #title>
                    <div class="file-name d-flex align-center">
                      {{ file.file_name }}
                    </div>
                  </template>
                  <template #subtitle>
                    <div class="file-size mt-1">{{ formatFileSize(file.file_size) }}</div>
                  </template>
                  <template #append>
                    <v-icon
                      v-if="isVideoSelected(file)"
                      icon="mdi-check-circle"
                      color="success"
                      size="24"
                      class="selection-icon"
                    />
                  </template>
                </v-list-item>
              </v-list>
            </v-card>
          </div>

          <!-- 字幕文件列表 -->
          <div v-if="subtitleFiles.length > 0">
            <div class="section-title mb-3 d-flex align-center">
              <v-icon icon="mdi-closed-caption" class="mr-2" color="info" />
              <span class="text-h6">字幕文件</span>
              <v-chip class="ml-2" size="small" color="info" variant="flat">{{ subtitleFiles.length }}</v-chip>
            </div>
            <v-card class="file-list-card">
              <v-list class="file-list py-0" bg-color="transparent">
                <v-list-item
                  v-for="file in subtitleFiles"
                  :key="`subtitle-${file.file_id}-${isSubtitleSelected(file)}`"
                  class="file-item"
                  rounded="lg"
                  :class="{ 'selected-item': (playerType === 'infuse' || playerType === 'mpv') && isSubtitleSelected(file) }"
                  lines="two"
                  @click="(playerType === 'infuse' || playerType === 'mpv') ? handleSubtitleSelect(file) : null"
                >
                  <template #prepend>
                    <v-avatar color="info" variant="tonal" size="36" class="file-icon">
                      <v-icon icon="mdi-file-document-outline" size="16" />
                    </v-avatar>
                  </template>
                  <template #title>
                    <div class="file-name d-flex align-center">
                      {{ file.file_name }}
                      <v-chip v-if="isSubtitleSelected(file)" color="success" size="x-small" class="ml-2">已选择</v-chip>
                    </div>
                  </template>
                  <template #subtitle>
                    <div class="file-size mt-1">{{ formatFileSize(file.file_size) }}</div>
                  </template>
                  <template #append>
                    <template v-if="playerType === 'iina'">
                      <v-btn
                        color="info"
                        size="small"
                        variant="tonal"
                        :href="getDownloadUrl(file)"
                        target="_blank"
                        class="download-btn"
                      >
                        <v-icon start icon="mdi-download" />
                        <span>下载</span>
                      </v-btn>
                    </template>
                    <template v-else>
                      <div class="d-flex align-center">
                        <v-icon
                          v-if="isSubtitleSelected(file)"
                          icon="mdi-check-circle"
                          color="success"
                          size="24"
                          class="mr-2 selection-icon"
                        />
                        <v-checkbox
                          density="compact"
                          color="info"
                          :model-value="isSubtitleSelected(file)"
                          @click.stop="handleSubtitleSelect(file)"
                          hide-details
                          class="subtitle-checkbox mr-0"
                        />
                      </div>
                    </template>
                  </template>
                </v-list-item>
              </v-list>
            </v-card>
          </div>
        </div>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions class="pa-5">
        <div class="file-selection-info d-flex align-center">
          <v-icon 
            :icon="currentVideoSelection ? 'mdi-check-circle' : 'mdi-alert-circle-outline'" 
            :color="currentVideoSelection ? 'success' : 'warning'"
            size="small"
            class="mr-2"
          />
          <span class="text-caption" :class="{'text-success': currentVideoSelection, 'text-warning': !currentVideoSelection}">
            {{ currentVideoSelection ? `已选择: ${formatFileName(currentVideoSelection.file_name)}` : '请选择视频文件' }}
          </span>
          
          <template v-if="(playerType === 'infuse' || playerType === 'mpv') && subtitleFiles.length > 0">
            <v-divider vertical class="mx-3" />
            <v-icon 
              :icon="currentSubtitleSelection ? 'mdi-check-circle' : 'mdi-information-outline'" 
              :color="currentSubtitleSelection ? 'success' : 'info'"
              size="small"
              class="mr-2"
            />
            <span class="text-caption" :class="{'text-success': currentSubtitleSelection}">
              {{ currentSubtitleSelection ? `字幕: ${formatFileName(currentSubtitleSelection.file_name)}` : '可选择字幕' }}
            </span>
          </template>
        </div>
        
        <v-spacer></v-spacer>
        
        <div class="d-flex gap-2">
          <v-btn color="grey-darken-1" variant="text" @click="handleClose">取消</v-btn>
          <v-btn 
            color="primary" 
            variant="elevated" 
            :disabled="!currentVideoSelection" 
            @click="handleConfirm"
            class="play-btn"
          >
            <v-icon start>mdi-play</v-icon>播放
          </v-btn>
        </div>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { FileType } from '../api/model'
import type { DownloadedFile } from '../api/model'

const props = defineProps<{
  modelValue: boolean
  files: DownloadedFile[]
  playerType: 'iina' | 'infuse' | 'mpv'
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'select', videoFile: DownloadedFile, subtitleFile?: DownloadedFile): void
}>()

// 内部状态管理
const dialog = ref(false)
const isInitialized = ref(false)
const currentVideoSelection = ref<DownloadedFile | null>(null)
const currentSubtitleSelection = ref<DownloadedFile | null>(null)
const localFiles = ref<DownloadedFile[]>([])
const processingSelection = ref(false)

// 计算属性
const videoFiles = computed(() => {
  return localFiles.value.filter(file => file.file_type === FileType.Video)
})

const subtitleFiles = computed(() => {
  return localFiles.value.filter(file => file.file_type === FileType.Subtitle)
})

// 监听对话框状态变化
watch(() => props.modelValue, (newVal) => {
  // 延迟设置内部dialog状态，避免直接绑定
  setTimeout(() => {
    dialog.value = newVal
    
    if (newVal) {
      // 对话框打开时，初始化本地文件列表和选择状态
      initializeComponent()
    }
  }, 0)
}, { immediate: true })

// 监听文件列表变化
watch(() => props.files, (newFiles) => {
  if (dialog.value && newFiles.length > 0) {
    // 创建深拷贝，避免直接引用外部数据
    localFiles.value = JSON.parse(JSON.stringify(newFiles))
    
    // 延迟初始化选择状态，确保DOM已更新
    setTimeout(() => {
      initializeSelection()
      forceRender()
    }, 50)
  }
}, { deep: true })

// 初始化组件
function initializeComponent() {
  // 重置状态
  resetSelections()
  
  // 初始化本地文件列表
  if (props.files.length > 0) {
    localFiles.value = JSON.parse(JSON.stringify(props.files))
  } else {
    localFiles.value = []
  }
  
  // 初始化选择状态
  nextTick(() => {
    initializeSelection()
    isInitialized.value = true
    forceRender()
  })
}

// 初始化选择状态
function initializeSelection() {
  // 默认选择第一个视频文件
  if (videoFiles.value.length > 0 && !currentVideoSelection.value) {
    const firstVideo = JSON.parse(JSON.stringify(videoFiles.value[0]))
    handleVideoSelection(firstVideo)
  }
  
  // 只有在Infuse模式下才自动选择字幕
  if ((props.playerType === 'infuse' || props.playerType === 'mpv') && subtitleFiles.value.length === 1 && !currentSubtitleSelection.value) {
    const firstSubtitle = JSON.parse(JSON.stringify(subtitleFiles.value[0]))
    handleSubtitleSelection(firstSubtitle)
  }
}

// 检查文件是否被选中
function isVideoSelected(file: DownloadedFile) {
  return currentVideoSelection.value?.file_id === file.file_id
}

function isSubtitleSelected(file: DownloadedFile) {
  return currentSubtitleSelection.value?.file_id === file.file_id
}

// 处理视频文件选择 (与UI交互)
function handleVideoSelect(file: DownloadedFile) {
  if (processingSelection.value) return
  
  processingSelection.value = true
  const fileClone = JSON.parse(JSON.stringify(file))
  
  // 使用setTimeout确保DOM操作和状态更新分离
  setTimeout(() => {
    handleVideoSelection(fileClone)
    processingSelection.value = false
  }, 0)
}

// 内部视频选择逻辑
function handleVideoSelection(file: DownloadedFile) {
  // 先清空当前选择，再设置新选择
  currentVideoSelection.value = null
  
  // 延迟设置新选择，确保Vue能检测到变化
  setTimeout(() => {
    currentVideoSelection.value = file
    forceRender()
  }, 10)
}

// 处理字幕文件选择 (与UI交互)
function handleSubtitleSelect(file: DownloadedFile) {
  if (processingSelection.value || (props.playerType !== 'infuse' && props.playerType !== 'mpv')) return
  
  processingSelection.value = true
  const fileClone = JSON.parse(JSON.stringify(file))
  
  // 使用setTimeout确保DOM操作和状态更新分离
  setTimeout(() => {
    handleSubtitleSelection(fileClone)
    processingSelection.value = false
  }, 0)
}

// 内部字幕选择逻辑
function handleSubtitleSelection(file: DownloadedFile) {
  // 切换选择状态
  if (currentSubtitleSelection.value?.file_id === file.file_id) {
    currentSubtitleSelection.value = null
  } else {
    // 先清空再设置，确保Vue能检测到变化
    currentSubtitleSelection.value = null
    
    setTimeout(() => {
      currentSubtitleSelection.value = file
      forceRender()
    }, 10)
  }
}

// 获取字幕下载链接
function getDownloadUrl(file: DownloadedFile) {
  return `/api/bangumi/${file.file_id}/download/${encodeURIComponent(file.file_name)}`
}

// 格式化文件大小
function formatFileSize(bytes: number) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`
}

// 格式化文件名（如果太长则截断）
function formatFileName(fileName: string) {
  if (fileName.length > 25) {
    return fileName.substring(0, 22) + '...'
  }
  return fileName
}

// 处理对话框变化
function handleDialogChange(val: boolean) {
  if (!val && isInitialized.value) {
    // 如果是关闭对话框，发送更新事件
    emit('update:modelValue', false)
    
    // 延迟重置选择状态，确保DOM已更新
    setTimeout(() => {
      resetSelections()
    }, 200)
  }
}

// 关闭对话框
function handleClose() {
  dialog.value = false
  emit('update:modelValue', false)
}

// 确认选择
function handleConfirm() {
  if (!currentVideoSelection.value) return
  
  // 先保存临时变量，避免引用问题
  const videoFile = JSON.parse(JSON.stringify(currentVideoSelection.value))
  let subtitleFile = undefined
  
  if ((props.playerType === 'infuse' || props.playerType === 'mpv') && currentSubtitleSelection.value) {
    subtitleFile = JSON.parse(JSON.stringify(currentSubtitleSelection.value))
  }
  
  // 先关闭对话框
  dialog.value = false
  emit('update:modelValue', false)
  
  // 重要：使用较长的延迟确保DOM完全更新
  setTimeout(() => {
    // 在发送选择事件前最后一次检查
    if (!videoFile) {
      console.error('无效的视频文件选择')
      return
    }
    
    // 发送选择事件
    emit('select', videoFile, subtitleFile)
    
    // 延迟清理状态
    setTimeout(() => {
      resetSelections()
    }, 100)
  }, 300)
}

// 重置选择状态
function resetSelections() {
  currentVideoSelection.value = null
  currentSubtitleSelection.value = null
}

// 强制重新渲染
function forceRender() {
  nextTick(() => {
    // 触发DOM更新
    const items = document.querySelectorAll('.file-item')
    items.forEach(item => {
      item.classList.remove('render-update')
      setTimeout(() => {
        item.classList.add('render-update')
      }, 0)
    })
    
    // 确保选择图标可见
    const icons = document.querySelectorAll('.selection-icon')
    icons.forEach(icon => {
      if (icon instanceof HTMLElement) {
        icon.style.opacity = '0'
        setTimeout(() => {
          icon.style.opacity = '1'
        }, 10)
      }
    })
  })
}

// 组件生命周期钩子
onMounted(() => {
  // 组件挂载后初始化
  if (props.modelValue) {
    initializeComponent()
  }
})

// 组件卸载前清理状态
onBeforeUnmount(() => {
  resetSelections()
  isInitialized.value = false
  localFiles.value = []
})
</script>

<style scoped>
.file-selection-dialog {
  overflow: hidden;
}

.dialog-header {
  background: rgba(var(--v-theme-surface-variant), 0.06);
}

.file-list-card {
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(30, 30, 30, 0.8) !important;
  border-radius: 12px;
  overflow: hidden;
}

.file-list {
  max-height: 300px;
  overflow-y: auto;
}

.file-item {
  transition: all 0.2s ease;
  margin: 4px;
  border-left: 3px solid transparent;
  background: rgba(255, 255, 255, 0.02);
  padding: 12px 16px;
  min-height: 64px !important;
  cursor: pointer;
}

.file-item:hover {
  background: rgba(var(--v-theme-primary), 0.07);
  transform: translateY(-1px);
}

.selected-item {
  background: rgba(var(--v-theme-primary), 0.1) !important;
  border-left-color: rgb(var(--v-theme-primary));
}

.file-name {
  font-size: 0.95rem;
  font-weight: 500;
  white-space: normal;
  word-break: break-word;
  line-height: 1.4;
  margin-right: 8px;
}

.file-size {
  font-size: 0.8rem;
  opacity: 0.7;
}

.section-title {
  font-weight: 500;
}

.download-btn {
  transition: all 0.2s ease;
}

.download-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.file-icon {
  margin: 4px 12px 4px 0;
  align-self: flex-start;
}

.subtitle-checkbox {
  margin: 0;
}

.selection-icon {
  transition: opacity 0.2s ease;
}

.render-update {
  /* 辅助类，用于强制DOM更新 */
  opacity: 1;
}

.play-btn {
  min-width: 100px;
}

/* 自定义滚动条样式 */
.file-list::-webkit-scrollbar {
  width: 5px;
}

.file-list::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.02);
}

.file-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.file-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style> 