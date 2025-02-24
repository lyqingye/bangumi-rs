<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getConfig, updateConfig } from '@/api/api'
import type { Config, ParserConfig } from '@/api/model'
import { LogLevel } from '@/api/model'
import { useSnackbar } from '@/composables/useSnackbar'

const { showSnackbar } = useSnackbar()
const loading = ref(false)
const config = ref<Config>()
const expandedPanels = ref<number[]>([]) // 控制面板展开状态

// 日志级别选项
const logLevelOptions = [
  { title: '错误', value: LogLevel.Error },
  { title: '警告', value: LogLevel.Warn },
  { title: '信息', value: LogLevel.Info },
  { title: '调试', value: LogLevel.Debug },
  { title: '追踪', value: LogLevel.Trace }
]

// 解析器类型
type Parser = keyof ParserConfig
const parsers: Parser[] = ['siliconflow', 'deepseek', 'deepbricks']

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
  <v-container class="settings-container">
    <v-row>
      <v-col cols="12" class="d-flex align-center justify-space-between mb-6">
        <div class="d-flex align-center">
          <v-icon icon="mdi-cog" size="x-large" class="mr-4" />
          <h1 class="text-h4">系统设置</h1>
        </div>
        <div class="d-flex">
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

    <v-row v-if="config">
      <v-col cols="12" md="10" lg="8" class="mx-auto">
        <v-card>
          <v-card-text>
            <v-expansion-panels v-model="expandedPanels" multiple>
              <!-- 基础配置 -->
              <v-expansion-panel elevation="0">
                <v-expansion-panel-title>
                  <template v-slot:default="{ expanded }">
                    <v-row no-gutters>
                      <v-col cols="12" class="d-flex align-center">
                        <v-icon
                          :icon="expanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
                          class="mr-4"
                        />
                        <v-icon icon="mdi-tune" class="mr-4" color="primary" />
                        <span class="text-h6">基础配置</span>
                      </v-col>
                    </v-row>
                  </template>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
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
                </v-expansion-panel-text>
              </v-expansion-panel>

              <!-- 下载器配置 -->
              <v-expansion-panel elevation="0">
                <v-expansion-panel-title>
                  <template v-slot:default="{ expanded }">
                    <v-row no-gutters>
                      <v-col cols="12" class="d-flex align-center">
                        <v-icon
                          :icon="expanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
                          class="mr-4"
                        />
                        <v-icon icon="mdi-download" class="mr-4" color="primary" />
                        <span class="text-h6">下载器配置</span>
                      </v-col>
                    </v-row>
                  </template>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <v-card variant="outlined">
                    <v-card-item>
                      <v-card-title>115网盘</v-card-title>
                      <v-card-subtitle>配置115网盘下载器</v-card-subtitle>
                    </v-card-item>
                    <v-card-text>
                      <v-row>
                        <v-col cols="12">
                          <v-textarea
                            v-model="config.pan115.cookies"
                            label="Cookies"
                            variant="outlined"
                            density="comfortable"
                            class="mb-4"
                            rows="3"
                            prepend-inner-icon="mdi-cookie"
                          />
                        </v-col>
                      </v-row>
                      <v-row>
                        <v-col cols="12" md="8">
                          <v-text-field
                            v-model="config.pan115.download_dir"
                            label="下载目录"
                            variant="outlined"
                            density="comfortable"
                            class="mb-4"
                            prepend-inner-icon="mdi-folder-download"
                          />
                        </v-col>
                        <v-col cols="12" md="4">
                          <v-text-field
                            v-model="config.pan115.max_requests_per_second"
                            label="每秒最大请求数"
                            type="number"
                            variant="outlined"
                            density="comfortable"
                            class="mb-4"
                            prepend-inner-icon="mdi-speedometer"
                          />
                        </v-col>
                      </v-row>
                    </v-card-text>
                  </v-card>
                </v-expansion-panel-text>
              </v-expansion-panel>

              <!-- 通知配置 -->
              <v-expansion-panel elevation="0">
                <v-expansion-panel-title>
                  <template v-slot:default="{ expanded }">
                    <v-row no-gutters>
                      <v-col cols="12" class="d-flex align-center">
                        <v-icon
                          :icon="expanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
                          class="mr-4"
                        />
                        <v-icon icon="mdi-bell" class="mr-4" color="primary" />
                        <span class="text-h6">通知配置</span>
                      </v-col>
                    </v-row>
                  </template>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
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
                </v-expansion-panel-text>
              </v-expansion-panel>

              <!-- 解析器配置 -->
              <v-expansion-panel elevation="0">
                <v-expansion-panel-title>
                  <template v-slot:default="{ expanded }">
                    <v-row no-gutters>
                      <v-col cols="12" class="d-flex align-center">
                        <v-icon
                          :icon="expanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
                          class="mr-4"
                        />
                        <v-icon icon="mdi-file-search" class="mr-4" color="primary" />
                        <span class="text-h6">解析器配置</span>
                      </v-col>
                    </v-row>
                  </template>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <v-card
                    v-for="parser in parsers"
                    :key="parser"
                    class="mb-6"
                    variant="outlined"
                  >
                    <v-card-item>
                      <v-card-title>{{ parser }}</v-card-title>
                      <v-card-subtitle>配置 {{ parser }} 解析器</v-card-subtitle>
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
                    </v-card-text>
                  </v-card>
                </v-expansion-panel-text>
              </v-expansion-panel>

              <!-- 元数据配置 -->
              <v-expansion-panel elevation="0">
                <v-expansion-panel-title>
                  <template v-slot:default="{ expanded }">
                    <v-row no-gutters>
                      <v-col cols="12" class="d-flex align-center">
                        <v-icon
                          :icon="expanded ? 'mdi-chevron-up' : 'mdi-chevron-down'"
                          class="mr-4"
                        />
                        <v-icon icon="mdi-database-search" class="mr-4" color="primary" />
                        <span class="text-h6">元数据配置</span>
                      </v-col>
                    </v-row>
                  </template>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
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
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- 加载中状态 -->
    <v-row v-else>
      <v-col cols="12" class="d-flex justify-center">
        <v-progress-circular
          indeterminate
          color="primary"
          size="64"
        />
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.settings-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

.v-expansion-panels {
  background: transparent;
}

.v-expansion-panel {
  margin-bottom: 16px;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 8px;
}

.v-expansion-panel-title {
  padding: 16px;
}

.v-expansion-panel-text {
  padding: 24px;
}

.v-card {
  border-radius: 8px;
}
</style> 