<template>
  <v-dialog v-model="dialog" max-width="500" persistent>
    <v-card class="refresh-dialog">
      <v-card-title class="text-h5 pa-4">
        <v-icon icon="mdi-refresh" size="24" class="me-2" color="primary" />
        选择刷新方式
      </v-card-title>

      <v-card-text class="pa-4">
        <div class="options-container">
          <!-- 强制刷新选项 -->
          <v-card
            class="option-card mb-3"
            elevation="0"
            :class="{ selected: selectedOption === true }"
            @click="selectedOption = true"
            variant="outlined"
          >
            <div class="d-flex align-center">
              <v-icon
                :icon="selectedOption === true ? 'mdi-radiobox-marked' : 'mdi-radiobox-blank'"
                color="primary"
                class="me-3"
                size="20"
              />
              <div class="option-content">
                <div class="text-subtitle-1 font-weight-medium">强制刷新所有元数据</div>
                <div class="text-body-2 text-medium-emphasis mt-1">
                  将替换所有现有数据，包括标题、简介、封面等信息
                </div>
              </div>
            </div>
          </v-card>

          <!-- 补齐缺失选项 -->
          <v-card
            class="option-card"
            elevation="0"
            :class="{ selected: selectedOption === false }"
            @click="selectedOption = false"
            variant="outlined"
          >
            <div class="d-flex align-center">
              <v-icon
                :icon="selectedOption === false ? 'mdi-radiobox-marked' : 'mdi-radiobox-blank'"
                color="primary"
                class="me-3"
                size="20"
              />
              <div class="option-content">
                <div class="text-subtitle-1 font-weight-medium">补齐缺失元数据</div>
                <div class="text-body-2 text-medium-emphasis mt-1">
                  仅添加缺失的数据，不会覆盖已有信息
                </div>
              </div>
            </div>
          </v-card>
        </div>
      </v-card-text>

      <v-card-actions class="pa-4 pt-0">
        <v-spacer />
        <v-btn variant="text" @click="handleCancel" :disabled="loading">取消</v-btn>
        <v-btn
          color="primary"
          @click="handleConfirm"
          :loading="loading"
          :disabled="selectedOption === null"
        >
          确认
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'confirm', force: boolean): void
}>()

const dialog = ref(props.modelValue)
const selectedOption = ref<boolean | null>(false)
const loading = ref(false)

watch(
  () => props.modelValue,
  (newVal) => {
    dialog.value = newVal
    if (newVal) {
      selectedOption.value = false
      loading.value = false
    }
  }
)

watch(dialog, (newVal) => {
  emit('update:modelValue', newVal)
})

const handleCancel = () => {
  dialog.value = false
}

const handleConfirm = async () => {
  if (selectedOption.value === null) return
  loading.value = true
  emit('confirm', selectedOption.value)
}
</script>

<style scoped>
.refresh-dialog {
  background: rgba(30, 30, 30, 0.95);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.option-card {
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  background: rgba(255, 255, 255, 0.05) !important;
}

.option-card:hover {
  background: rgba(var(--v-theme-primary), 0.1) !important;
  transform: translateY(-1px);
}

.option-card.selected {
  background: rgba(var(--v-theme-primary), 0.15) !important;
  border-color: rgb(var(--v-theme-primary)) !important;
}

.option-content {
  flex: 1;
  min-width: 0;
}
</style> 