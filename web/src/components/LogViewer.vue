<template>
  <!-- 添加 v-expansion-panels 外层容器 -->
  <v-expansion-panels v-model="panelState">
    <v-expansion-panel>
      <v-expansion-panel-title>
        <div class="d-flex align-center">
          <v-icon icon="mdi-console" class="me-2" />
          系统日志
          <v-chip v-if="connected" color="success" size="small" class="ms-2"> 已连接 </v-chip>
          <v-chip v-else color="error" size="small" class="ms-2"> 未连接 </v-chip>
        </div>
      </v-expansion-panel-title>
      <v-expansion-panel-text>
        <div ref="logContainer" class="log-container" :class="{ 'dark-theme': isDarkTheme }">
          <pre v-for="(log, index) in logs" :key="index" class="log-entry"><code>{{ log }}</code></pre>
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
  expanded?: boolean
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

// 定义 WebSocket 相关函数
const connectWebSocket = () => {
  if (ws) return // 防止重复连接

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

// 初始化面板状态
onMounted(() => {
  if (props.expanded) {
    panelState.value = [0]
    connectWebSocket()
  }
})

// 监听 expanded 属性变化
watch(() => props.expanded, (newVal) => {
  if (newVal) {
    panelState.value = [0]
    connectWebSocket()
  } else {
    panelState.value = []
    disconnectWebSocket()
  }
}, { immediate: true })

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
/* 面板样式 */
:deep(.v-expansion-panel) {
  border-radius: 12px !important;
  box-shadow: 0 2px 8px rgba(var(--v-theme-on-surface), 0.05) !important;
  transition: all 0.3s ease;
  background-color: rgb(var(--v-theme-surface)) !important;
}

:deep(.v-expansion-panel:hover) {
  box-shadow: 0 4px 12px rgba(var(--v-theme-on-surface), 0.1) !important;
}

:deep(.v-expansion-panel-title) {
  padding: 16px 20px !important;
  min-height: 64px !important;
}

:deep(.v-expansion-panel-text__wrapper) {
  padding: 0 !important;
}

/* 日志容器样式 */
.log-container {
  height: 400px;
  overflow-y: auto;
  padding: 12px;
  font-family: 'Roboto', sans-serif;
  font-size: 14px;
  line-height: 1.1;
  letter-spacing: 0;
  border-radius: 0 0 12px 12px;
  background-color: rgb(var(--v-theme-surface));
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}

.log-container.dark-theme {
  background-color: rgb(var(--v-theme-surface));
  color: rgba(var(--v-theme-on-surface), 0.87);
}

/* 滚动条样式 */
.log-container::-webkit-scrollbar {
  width: 8px;
}

.log-container::-webkit-scrollbar-track {
  background: transparent;
}

.log-container::-webkit-scrollbar-thumb {
  background-color: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 4px;
}

.log-container::-webkit-scrollbar-thumb:hover {
  background-color: rgba(var(--v-theme-on-surface), 0.3);
}

/* 日志条目样式 */
.log-entry {
  margin: 0;
  padding: 0;
  white-space: pre;
  font-family: inherit;
  line-height: 1;
}

.log-entry code {
  font-family: inherit;
  display: inline;
  padding: 0;
  margin: 0;
  line-height: 1.1;
  background-color: transparent;
  color: inherit;
  opacity: 1;
}

/* 连接状态标签样式 */
:deep(.v-chip) {
  font-weight: 500;
  letter-spacing: 0.5px;
}

/* 图标样式 */
:deep(.v-icon) {
  opacity: 0.9;
}
</style>
