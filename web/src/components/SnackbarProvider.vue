<script setup lang="ts">
import { ref } from 'vue'

interface SnackbarOptions {
  text: string
  color?: string
  timeout?: number
  location?: string
}

const show = ref(false)
const text = ref('')
const color = ref('success')
const timeout = ref(3000)
const location = ref('top right')

const showSnackbar = (options: SnackbarOptions) => {
  text.value = options.text
  color.value = options.color || 'success'
  timeout.value = options.timeout || 3000
  location.value = options.location || 'top right'
  show.value = true
}

// 导出为全局方法
defineExpose({
  showSnackbar
})
</script>

<template>
  <v-snackbar v-model="show" :color="color" :timeout="timeout" :location="location">
    {{ text }}
  </v-snackbar>
</template>
