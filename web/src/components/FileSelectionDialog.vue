<template>
  <v-dialog v-model="dialog" max-width="500px">
    <v-card>
      <v-card-title class="text-h5 bg-primary">
        选择播放文件
        <v-spacer></v-spacer>
        <v-btn icon @click="dialog = false">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>

      <v-card-text class="pt-4">
        <p v-if="files.length === 0" class="text-center my-4">没有可播放的文件</p>
        
        <div v-else>
          <p class="text-body-1 mb-4">选择要播放的文件：</p>
          
          <!-- 视频文件列表 -->
          <div v-if="videoFiles.length > 0" class="mb-4">
            <div class="text-subtitle-1 mb-2 d-flex align-center">
              <v-icon class="mr-2">mdi-video</v-icon>
              视频文件
            </div>
            <v-list class="file-list">
              <v-list-item
                v-for="file in videoFiles"
                :key="file.file_id"
                @click="selectFile(file)"
                :class="{ 'selected-item': selectedFile?.file_id === file.file_id }"
                class="file-item"
              >
                <v-list-item-title class="d-flex align-center">
                  <v-icon class="mr-2" color="primary" size="small">mdi-file-video</v-icon>
                  {{ file.file_name }}
                </v-list-item-title>
                <v-list-item-subtitle class="text-caption">
                  {{ formatFileSize(file.file_size) }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </div>

          <!-- 字幕文件列表 -->
          <div v-if="subtitleFiles.length > 0">
            <div class="text-subtitle-1 mb-2 d-flex align-center">
              <v-icon class="mr-2">mdi-closed-caption</v-icon>
              字幕文件
            </div>
            <v-list class="file-list">
              <v-list-item
                v-for="file in subtitleFiles"
                :key="file.file_id"
                @click="selectSubtitleFile(file)"
                :class="{ 'selected-item': selectedSubtitleFile?.file_id === file.file_id }"
                class="file-item"
              >
                <v-list-item-title class="d-flex align-center">
                  <v-icon class="mr-2" color="info" size="small">mdi-file-document</v-icon>
                  {{ file.file_name }}
                </v-list-item-title>
                <v-list-item-subtitle class="text-caption">
                  {{ formatFileSize(file.file_size) }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </div>
        </div>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="primary"
          variant="tonal"
          @click="cancel"
        >
          取消
        </v-btn>
        <v-btn
          color="primary"
          :disabled="!selectedFile"
          @click="confirm"
        >
          播放
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watchEffect } from 'vue'
import { FileType } from '../api/model'
import type { DownloadedFile } from '../api/model'

const props = defineProps<{
  modelValue: boolean
  files: DownloadedFile[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'select', videoFile: DownloadedFile, subtitleFile?: DownloadedFile): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const selectedFile = ref<DownloadedFile | null>(null)
const selectedSubtitleFile = ref<DownloadedFile | null>(null)

// 自动选择第一个视频文件
watchEffect(() => {
  if (props.modelValue && videoFiles.value.length > 0 && !selectedFile.value) {
    selectedFile.value = videoFiles.value[0]
  }
  
  // 如果只有一个字幕文件，自动选择
  if (props.modelValue && subtitleFiles.value.length === 1 && !selectedSubtitleFile.value) {
    selectedSubtitleFile.value = subtitleFiles.value[0]
  }
})

// 分类文件
const videoFiles = computed(() => {
  return props.files.filter(file => file.file_type === FileType.Video)
})

const subtitleFiles = computed(() => {
  return props.files.filter(file => file.file_type === FileType.Subtitle)
})

// 选择文件
const selectFile = (file: DownloadedFile) => {
  selectedFile.value = file
}

const selectSubtitleFile = (file: DownloadedFile) => {
  // 切换选择状态
  selectedSubtitleFile.value = selectedSubtitleFile.value?.file_id === file.file_id ? null : file
}

// 格式化文件大小
const formatFileSize = (bytes: number) => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`
}

// 取消选择
const cancel = () => {
  dialog.value = false
}

// 确认选择
const confirm = () => {
  if (selectedFile.value) {
    emit('select', selectedFile.value, selectedSubtitleFile.value || undefined)
    dialog.value = false
  }
}
</script>

<style scoped>
.file-list {
  border-radius: 8px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.file-item {
  transition: all 0.2s ease;
  border-left: 3px solid transparent;
}

.file-item:hover {
  background: rgba(var(--v-theme-primary), 0.1);
}

.selected-item {
  background: rgba(var(--v-theme-primary), 0.15) !important;
  border-left-color: rgb(var(--v-theme-primary));
}
</style> 