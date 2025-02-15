<template>
  <!-- 添加 v-expansion-panels 外层容器 -->
  <v-expansion-panels v-model="panelState">
    <v-expansion-panel>
      <v-expansion-panel-title>
        <div class="d-flex align-center">
          <v-icon icon="mdi-console" class="me-2" />
          日志查看器
          <v-chip v-if="connected" color="success" size="small" class="ms-2"> 已连接 </v-chip>
          <v-chip v-else color="error" size="small" class="ms-2"> 未连接 </v-chip>
        </div>
      </v-expansion-panel-title>
      <v-expansion-panel-text>
        <div ref="logContainer" class="log-container" :class="{ 'dark-theme': isDarkTheme }">
          <pre v-for="(log, index) in logs" :key="index" class="log-entry">
            <code>{{ log }}</code>
          </pre>
        </div>
      </v-expansion-panel-text>
    </v-expansion-panel>
  </v-expansion-panels>
</template>

<script lang="ts" setup>
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import { useTheme } from 'vuetify'

const props = defineProps<{
  websocketUrl: string
}>()

const theme = useTheme()
const isDarkTheme = computed(() => theme.global.current.value.dark)
const connected = ref(false)
const logs = ref<string[]>([])
const logContainer = ref<HTMLElement | null>(null)
const panelState = ref<number[]>([]) // 使用数组跟踪展开状态
let ws: WebSocket | null = null
let reconnectAttempts = 0
const MAX_RECONNECT_ATTEMPTS = 5

const connectWebSocket = () => {
  if (ws) return // 防止重复连接

  console.log('Connecting to:', props.websocketUrl)
  ws = new WebSocket(props.websocketUrl)

  ws.onopen = () => {
    connected.value = true
    reconnectAttempts = 0
    logs.value.push(`[系统] WebSocket 连接已建立 (${new Date().toLocaleTimeString()})`)
  }

  ws.onmessage = event => {
    logs.value.push(event.data)
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }

  ws.onclose = event => {
    connected.value = false
    logs.value.push(`[系统] WebSocket 连接关闭 (${event.code})`)
    ws = null

    // 自动重连逻辑
    if (panelState.value != undefined && reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
      reconnectAttempts++
      setTimeout(() => {
        logs.value.push(`[系统] 尝试重新连接 (${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`)
        connectWebSocket()
      }, 3000)
    }
  }

  ws.onerror = error => {
    logs.value.push('[系统] WebSocket 连接错误: ' + (error as Event).type)
  }
}

const disconnectWebSocket = () => {
  if (ws) {
    ws.close(1000, '正常关闭')
    ws = null
  }
}

// 监听面板展开状态变化
watch(panelState, newVal => {
  console.log(newVal)
  if (newVal != undefined) {
    connectWebSocket()
  } else {
    disconnectWebSocket()
  }
})

onUnmounted(() => {
  disconnectWebSocket()
})
</script>

<style scoped>
/* 保持原有样式不变 */
.log-container {
  height: 300px;
  overflow-y: auto;
  background-color: #f5f5f5;
  border-radius: 4px;
  padding: 8px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 12px;
}

.log-container.dark-theme {
  background-color: #1e1e1e;
  color: #ffffff;
}

.log-entry {
  margin: 0;
  padding: 2px 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  text-align: left;
}

.log-entry code {
  font-family: inherit;
  display: block;
  text-align: left;
  padding: 0;
  margin: 0;
  background-color: transparent;
  color: inherit;
}
</style>
