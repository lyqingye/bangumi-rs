<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { getConfig, updateConfig } from '@/api/api'
import type { Config, ParserConfig } from '@/api/model'
import { LogLevel } from '@/api/model'
import { useSnackbar } from '@/composables/useSnackbar'

const { showSnackbar } = useSnackbar()
const loading = ref(false)
const config = ref<Config>()
const currentTab = ref(0)
const expandedPanels = ref<number[]>([]) // 控制面板展开状态

// 计算属性：处理no_proxy数组与字符串的转换
const noProxyStr = computed({
  get() {
    if (!config.value?.proxy.no_proxy) return ''
    return config.value.proxy.no_proxy.join(',')
  },
  set(val: string) {
    if (!config.value) return
    config.value.proxy.no_proxy = val ? val.split(',').map(s => s.trim()) : []
  }
})

// 配置分组
const tabs = [
  { title: '基础配置', icon: 'mdi-tune' },
  { title: '下载器配置', icon: 'mdi-download' },
  { title: '通知配置', icon: 'mdi-bell' },
  { title: '解析器配置', icon: 'mdi-file-search' },
  { title: '元数据配置', icon: 'mdi-database-search' }
]

// 日志级别选项
const logLevelOptions = [
  { title: '信息', value: LogLevel.Info },
  { title: '追踪', value: LogLevel.Trace },
  { title: '调试', value: LogLevel.Debug },
  { title: '警告', value: LogLevel.Warn },
  { title: '错误', value: LogLevel.Error }
]

// 解析器类型
type Parser = keyof ParserConfig
const parsers: Parser[] = ['raw', 'siliconflow', 'deepseek', 'deepbricks']

// 判断是否为需要完整配置的解析器
const isLLMParser = (parser: Parser) => {
  return parser !== 'raw'
}

// 加载配置
const loadConfig = async () => {
  try {
    loading.value = true
    config.value = await getConfig()
  } catch (error) {
    console.error('加载配置失败:', error)
  } finally {
    loading.value = false
  }
}

// 保存配置
const saveConfig = async () => {
  if (!config.value) return

  try {
    loading.value = true
    await updateConfig(config.value)
    showSnackbar({
      text: '配置保存成功',
      color: 'success',
      location: 'top right'
    })
  } catch (error) {
    console.error('保存配置失败:', error)
    showSnackbar({
      text: `保存配置失败: ${error}`,
      color: 'error',
      location: 'top right'
    })
  } finally {
    loading.value = false
  }
}

// 重置配置
const resetConfig = async () => {
  if (confirm('确定要重置所有配置吗？这将恢复所有设置到默认值。')) {
    await loadConfig()
    showSnackbar({
      text: '配置已重置',
      color: 'info',
      location: 'top right'
    })
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<template>
  <v-container fluid class="settings-page pa-6">
    <!-- 顶部标题栏 -->
    <v-row>
      <v-col cols="12" class="d-flex justify-space-between align-center">
        <div class="d-flex align-center">
          <v-icon icon="mdi-cog" size="32" class="mr-4" color="primary" />
          <h1 class="text-h4 font-weight-medium mb-0">系统配置</h1>
        </div>
        <div class="d-flex align-center">
          <v-btn
            prepend-icon="mdi-refresh"
            variant="outlined"
            class="mr-4"
            :loading="loading"
            @click="resetConfig"
          >
            重置
          </v-btn>
          <v-btn
            color="primary"
            prepend-icon="mdi-content-save"
            :loading="loading"
            @click="saveConfig"
          >
            保存配置
          </v-btn>
        </div>
      </v-col>
    </v-row>

    <!-- 导航条 -->
    <v-tabs
      v-model="currentTab"
      color="primary"
      align-tabs="start"
      class="settings-tabs"
      show-arrows
    >
      <v-tab
        v-for="(tab, index) in tabs"
        :key="index"
        :value="index"
      >
        <v-icon :icon="tab.icon" class="mr-2" />
        {{ tab.title }}
      </v-tab>
    </v-tabs>

    <!-- 配置内容 -->
    <v-window v-model="currentTab" class="settings-content">
      <v-window-item
        v-for="(tab, index) in tabs"
        :key="index"
        :value="index"
      >
        <v-container v-if="config" class="pa-6">
          <!-- 基础配置 -->
          <template v-if="index === 0">
            <v-row>
              <v-col cols="12" md="6">
                <v-select
                  v-model="config.log.level"
                  :items="logLevelOptions"
                  item-title="title"
                  item-value="value"
                  label="日志级别"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                />
              </v-col>
            </v-row>
            <v-row>
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="config.server.assets_path"
                  label="资源路径"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  prepend-inner-icon="mdi-folder"
                />
              </v-col>
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="config.server.listen_addr"
                  label="监听地址"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  prepend-inner-icon="mdi-web"
                />
              </v-col>
            </v-row>
            <v-row>
              <v-col cols="12">
                <v-text-field
                  v-model="config.server.database_url"
                  label="数据库地址"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  prepend-inner-icon="mdi-database"
                />
              </v-col>
            </v-row>
            <v-divider class="my-4" />
            <v-row>
              <v-col cols="12">
                <v-switch
                  v-model="config.proxy.enabled"
                  label="启用代理"
                  color="primary"
                  class="mb-4"
                  inset
                />
              </v-col>
            </v-row>
            <v-row>
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="config.proxy.http"
                  label="HTTP 代理"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  :disabled="!config.proxy.enabled"
                  prepend-inner-icon="mdi-web"
                />
              </v-col>
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="config.proxy.https"
                  label="HTTPS 代理"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  :disabled="!config.proxy.enabled"
                  prepend-inner-icon="mdi-shield"
                />
              </v-col>
            </v-row>
            <v-row>
              <v-col cols="12">
                <v-text-field
                  v-model="noProxyStr"
                  label="不使用代理的地址（多个地址以逗号分隔）"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  :disabled="!config.proxy.enabled"
                  prepend-inner-icon="mdi-block-helper"
                  hint="例如: localhost,127.0.0.1"
                  persistent-hint
                />
              </v-col>
            </v-row>
            
            <v-divider class="my-4" />
            
            <!-- Sentry配置 -->
            <v-row>
              <v-col cols="12">
                <v-switch
                  v-model="config.sentry.enabled"
                  label="启用 Sentry 错误监控"
                  color="primary"
                  class="mb-4"
                  inset
                />
              </v-col>
            </v-row>
            <v-row>
              <v-col cols="12">
                <v-text-field
                  v-model="config.sentry.dsn"
                  label="Sentry DSN"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  :disabled="!config.sentry.enabled"
                  prepend-inner-icon="mdi-bug"
                />
              </v-col>
            </v-row>
          </template>

          <!-- 下载器配置 -->
          <template v-if="index === 1">
            <!-- 115网盘配置 -->
            <v-card variant="outlined" class="mb-6">
              <v-card-item>
                <v-card-title>115网盘</v-card-title>
                <v-card-subtitle>配置115网盘下载器</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.downloader.pan115.enabled"
                      label="启用"
                      color="primary"
                      class="mb-4"
                      inset
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.pan115.download_dir"
                      label="下载目录"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-folder"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-textarea
                      v-model="config.downloader.pan115.cookies"
                      label="Cookies"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      rows="3"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-cookie"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.pan115.max_requests_per_second"
                      label="每秒最大请求数"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-speedometer"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.pan115.max_retry_count"
                      label="最大重试次数"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-refresh"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.pan115.download_timeout"
                      label="下载超时"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-timer-sand"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.pan115.retry_min_interval"
                      label="最小重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-timer-outline"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.pan115.retry_max_interval"
                      label="最大重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-timer"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.pan115.priority"
                      label="优先级"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      prepend-inner-icon="mdi-priority-high"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.downloader.pan115.delete_task_on_completion"
                      label="完成后删除任务（不会删除文件）"
                      color="primary"
                      class="mb-4"
                      :disabled="!config.downloader.pan115.enabled"
                      inset
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>

            <!-- qBittorrent配置 -->
            <v-card variant="outlined" class="mb-6">
              <v-card-item>
                <v-card-title>qBittorrent</v-card-title>
                <v-card-subtitle>配置qBittorrent下载器</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.downloader.qbittorrent.enabled"
                      label="启用"
                      color="primary"
                      class="mb-4"
                      inset
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.download_dir"
                      label="下载目录"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-folder"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.mount_path"
                      label="挂载路径（可选）"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-harddisk"
                      hint="容器内的挂载路径，该路径指向下载目录，用于在线播放"
                      persistent-hint
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.url"
                      label="服务器地址"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-web"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.username"
                      label="用户名"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-account"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.password"
                      label="密码"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      type="password"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-key"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.qbittorrent.max_retry_count"
                      label="最大重试次数"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-refresh"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.download_timeout"
                      label="下载超时"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-timer-sand"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.retry_min_interval"
                      label="最小重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-timer-outline"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.qbittorrent.retry_max_interval"
                      label="最大重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-timer"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.qbittorrent.priority"
                      label="优先级"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      prepend-inner-icon="mdi-priority-high"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-switch
                      v-model="config.downloader.qbittorrent.delete_task_on_completion"
                      label="完成后删除任务（不会删除文件）"
                      color="primary"
                      class="mb-4"
                      :disabled="!config.downloader.qbittorrent.enabled"
                      inset
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>

            <!-- Transmission配置 -->
            <v-card variant="outlined" class="mb-6">
              <v-card-item>
                <v-card-title>Transmission</v-card-title>
                <v-card-subtitle>配置Transmission下载器</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.downloader.transmission.enabled"
                      label="启用"
                      color="primary"
                      class="mb-4"
                      inset
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.transmission.download_dir"
                      label="下载目录"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-folder"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.transmission.mount_path"
                      label="挂载路径（可选）"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-harddisk"
                      hint="容器内的挂载路径，该路径指向下载目录，用于在线播放"
                      persistent-hint
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12">
                    <v-text-field
                      v-model="config.downloader.transmission.url"
                      label="服务器地址"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-web"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.transmission.username"
                      label="用户名"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-account"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.transmission.password"
                      label="密码"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      type="password"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-key"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.transmission.max_retry_count"
                      label="最大重试次数"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-refresh"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.transmission.download_timeout"
                      label="下载超时"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-timer-sand"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.transmission.retry_min_interval"
                      label="最小重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-timer-outline"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.downloader.transmission.retry_max_interval"
                      label="最大重试间隔"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-timer"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model.number="config.downloader.transmission.priority"
                      label="优先级"
                      type="number"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      prepend-inner-icon="mdi-priority-high"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-switch
                      v-model="config.downloader.transmission.delete_task_on_completion"
                      label="完成后删除任务（不会删除文件）"
                      color="primary"
                      class="mb-4"
                      :disabled="!config.downloader.transmission.enabled"
                      inset
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </template>

          <!-- 通知配置 -->
          <template v-if="index === 2">
            <v-card variant="outlined">
              <v-card-item>
                <v-card-title>Telegram</v-card-title>
                <v-card-subtitle>配置 Telegram 通知</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.notify.telegram.enabled"
                      label="启用通知"
                      color="primary"
                      class="mb-4"
                      inset
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.notify.telegram.token"
                      label="Bot Token"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.notify.telegram.enabled"
                      prepend-inner-icon="mdi-key"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.notify.telegram.chat_id"
                      label="Chat ID"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      :disabled="!config.notify.telegram.enabled"
                      prepend-inner-icon="mdi-chat"
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </template>

          <!-- 解析器配置 -->
          <template v-if="index === 3">
            <v-card
              v-for="parser in parsers"
              :key="parser"
              class="mb-6"
              variant="outlined"
            >
              <v-card-item>
                <v-card-title>{{ parser }}</v-card-title>
                <v-card-subtitle>配置 {{ parser === 'raw' ? '原生' : parser }} 解析器</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12">
                    <v-switch
                      v-model="config.parser[parser].enabled"
                      label="启用"
                      color="primary"
                      class="mb-4"
                      inset
                    />
                  </v-col>
                </v-row>
                <template v-if="isLLMParser(parser)">
                  <v-row>
                    <v-col cols="12" md="6">
                      <v-text-field
                        v-model="config.parser[parser].api_key"
                        label="API Key"
                        variant="outlined"
                        density="comfortable"
                        class="mb-4"
                        :disabled="!config.parser[parser].enabled"
                        prepend-inner-icon="mdi-key"
                      />
                    </v-col>
                    <v-col cols="12" md="6">
                      <v-text-field
                        v-model="config.parser[parser].model"
                        label="模型"
                        variant="outlined"
                        density="comfortable"
                        class="mb-4"
                        :disabled="!config.parser[parser].enabled"
                        prepend-inner-icon="mdi-brain"
                      />
                    </v-col>
                  </v-row>
                  <v-row>
                    <v-col cols="12">
                      <v-text-field
                        v-model="config.parser[parser].base_url"
                        label="Base URL"
                        variant="outlined"
                        density="comfortable"
                        class="mb-4"
                        :disabled="!config.parser[parser].enabled"
                        prepend-inner-icon="mdi-web"
                      />
                    </v-col>
                  </v-row>
                </template>
              </v-card-text>
            </v-card>
          </template>

          <!-- 元数据配置 -->
          <template v-if="index === 4">
            <!-- Mikan -->
            <v-card class="mb-6" variant="outlined">
              <v-card-item>
                <v-card-title>Mikan</v-card-title>
                <v-card-subtitle>配置 Mikan 元数据源</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-text-field
                  v-model="config.mikan.endpoint"
                  label="API 地址"
                  variant="outlined"
                  density="comfortable"
                  class="mb-4"
                  prepend-inner-icon="mdi-web"
                />
              </v-card-text>
            </v-card>

            <!-- Bangumi.tv -->
            <v-card class="mb-6" variant="outlined">
              <v-card-item>
                <v-card-title>Bangumi.tv</v-card-title>
                <v-card-subtitle>配置 Bangumi.tv 元数据源</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.bangumi_tv.endpoint"
                      label="API 地址"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-web"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.bangumi_tv.image_base_url"
                      label="图片 Base URL"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-image"
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>

            <!-- TMDB -->
            <v-card variant="outlined">
              <v-card-item>
                <v-card-title>TMDB</v-card-title>
                <v-card-subtitle>配置 TMDB 元数据源</v-card-subtitle>
              </v-card-item>
              <v-card-text>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.tmdb.api_key"
                      label="API Key"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-key"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.tmdb.language"
                      label="语言"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-translate"
                    />
                  </v-col>
                </v-row>
                <v-row>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.tmdb.base_url"
                      label="API 地址"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-web"
                    />
                  </v-col>
                  <v-col cols="12" md="6">
                    <v-text-field
                      v-model="config.tmdb.image_base_url"
                      label="图片 Base URL"
                      variant="outlined"
                      density="comfortable"
                      class="mb-4"
                      prepend-inner-icon="mdi-image"
                    />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </template>
        </v-container>

        <!-- 加载中状态 -->
        <v-container v-else class="d-flex justify-center align-center" style="height: 400px">
          <v-progress-circular
            indeterminate
            color="primary"
            size="64"
          />
        </v-container>
      </v-window-item>
    </v-window>
  </v-container>
</template>

<style scoped>
.settings-page {
  background-color: rgb(var(--v-theme-background));
  min-height: 100%;
}

.v-card {
  border-radius: 12px;
  transition: transform 0.2s, box-shadow 0.2s;
  background: transparent;
}

.v-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 25px 0 rgba(var(--v-theme-on-surface), 0.1);
}

.settings-tabs {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.settings-content {
  flex: 1;
  overflow-y: auto;
}

.v-card-item {
  padding: 20px;
}

.v-card-text {
  padding: 20px;
}

.v-divider {
  opacity: 0.08;
}

.v-row {
  margin: 0;
}

.v-col {
  padding: 12px;
}
</style> 