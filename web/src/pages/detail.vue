<template>
  <div v-if="anime" class="detail-page">
    <!-- 1. 背景层：海报墙 -->
    <div class="backdrop-section">
      <v-img :src="anime.backdrop_image_url" class="backdrop-image" cover>
        <div class="backdrop-overlay"></div>
      </v-img>
    </div>

    <!-- 2. 内容层 -->
    <div class="content-wrapper">
      <v-container>
        <v-row>
          <!-- 左侧：封面和元数据 -->
          <v-col cols="12" sm="4" md="3" class="sidebar-col">
            <div class="sidebar-content">
              <!-- 封面图 -->
              <div class="poster-wrapper">
                <!-- 已订阅彩带 -->
                <div class="ribbon-wrapper" v-if="isSubscribed">
                  <div class="ribbon">
                    <v-icon size="16" class="me-1">mdi-check-circle</v-icon>
                    已追番
                  </div>
                </div>

                <v-img
                  :src="anime.poster_image_url"
                  cover
                  class="poster-image"
                >
                  <template v-slot:placeholder>
                    <v-row class="fill-height ma-0" align="center" justify="center">
                      <v-progress-circular indeterminate color="grey-lighten-5" />
                    </v-row>
                  </template>

                  <!-- 右下角订阅按钮 -->
                  <div class="image-overlay">
                    <div class="action-buttons">
                      <!-- 刷新按钮 -->
                      <v-btn
                        variant="text"
                        size="small"
                        class="action-btn"
                        @click.stop="handleRefresh"
                        :loading="isRefreshing"
                      >
                        <v-icon icon="mdi-refresh" size="20" />
                        <v-tooltip activator="parent" location="top">
                          刷新元数据
                        </v-tooltip>
                      </v-btn>
                      <!-- 订阅按钮 -->
                      <v-btn
                        variant="text"
                        size="small"
                        class="action-btn"
                        @click.stop="toggleSubscribe"
                      >
                        <v-icon
                          :icon="isSubscribed ? 'mdi-heart' : 'mdi-heart-outline'"
                          :color="isSubscribed ? 'error' : undefined"
                          size="20"
                        />
                        <v-tooltip activator="parent" location="top">
                          {{ isSubscribed ? '已追番' : '追番' }}
                        </v-tooltip>
                      </v-btn>
                    </div>
                  </div>
                </v-img>
              </div>

              <!-- 评分部分 -->
              <div class="rating-section mt-6">
                <div class="d-flex align-center">
                  <span class="text-h4 font-weight-bold">{{ anime.rating }}</span>
                  <v-rating
                    v-model="ratingValue"
                    color="amber"
                    density="compact"
                    half-increments
                    readonly
                    class="ms-2"
                  />
                </div>
              </div>

              <!-- 将评分等基础信息移到封面下方 -->
              <div class="info-section">
                <div class="info-list">
                  <div class="info-card">
                    <div class="info-icon">
                      <v-icon>mdi-calendar</v-icon>
                    </div>
                    <div class="info-content">
                      <div class="info-label">首播日期</div>
                      <div class="info-value">{{ formatDate(anime.air_date) }}</div>
                    </div>
                  </div>
                  <div class="info-card">
                    <div class="info-icon">
                      <v-icon>mdi-refresh</v-icon>
                    </div>
                    <div class="info-content">
                      <div class="info-label">更新时间</div>
                      <div class="info-value">2025-02-03</div>
                    </div>
                  </div>
                  <div class="info-card">
                    <div class="info-icon">
                      <v-icon>mdi-format-list-numbered</v-icon>
                    </div>
                    <div class="info-content">
                      <div class="info-label">总集数</div>
                      <div class="info-value">13集</div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 外部链接 -->
              <div class="mt-6">
                <div class="text-h6 mb-3">外部链接</div>
                <div class="external-links">
                  <v-btn
                    v-if="anime.bangumi_tv_id"
                    variant="text"
                    class="link-btn"
                    :href="`https://bgm.tv/subject/${anime.bangumi_tv_id}`"
                    target="_blank"
                  >
                    <v-icon start icon="mdi-link-variant" />
                    Bangumi
                    <div class="link-id">#{{ anime.bangumi_tv_id }}</div>
                  </v-btn>
                  <v-btn
                    v-if="anime.tmdb_id"
                    variant="text"
                    class="link-btn"
                    :href="`https://www.themoviedb.org/tv/${anime.tmdb_id}`"
                    target="_blank"
                  >
                    <v-icon start icon="mdi-link-variant" />
                    TMDB
                    <div class="link-id">#{{ anime.tmdb_id }}</div>
                  </v-btn>
                  <v-btn
                    v-if="anime.mikan_id"
                    variant="text"
                    class="link-btn"
                    :href="`https://mikanani.me/Home/Bangumi/${anime.mikan_id}`"
                    target="_blank"
                  >
                    <v-icon start icon="mdi-link-variant" />
                    Mikan
                    <div class="link-id">#{{ anime.mikan_id }}</div>
                  </v-btn>
                </div>
              </div>
            </div>
          </v-col>

          <!-- 右侧：详情和剧集列表 -->
          <v-col cols="12" sm="8" md="9">
            <!-- 上部：番剧详情 -->
            <div class="anime-details">
              <h1 class="text-h3 font-weight-medium mb-4">{{ anime.name }}</h1>
              <p class="text-body-1 text-medium-emphasis mb-6">{{ anime.description }}</p>
            </div>

            <!-- 下部：剧集列表 -->
            <div class="episodes-section">
              <div v-if="episodes.length">
                
                <div class="episodes-list">
                  <div
                    v-for="episode in episodes"
                    :key="episode.id"
                    class="episode-item mb-3"
                  >
                    <div
                      class="episode-header pa-4"
                      :class="{ 'expanded': currentExpandedId === episode.id }"
                      @click="toggleEpisode(episode.id)"
                    >
                      <div class="d-flex align-center w-100">
                        <div class="episode-number me-4">{{ episode.number }}</div>
                        <div class="episode-info flex-grow-1 d-flex align-center justify-space-between">
                          <div class="d-flex align-center">
                            <div class="episode-name">{{ episode.name }}</div>
                            <v-icon
                              v-if="episode.download_state === State.Downloaded"
                              size="16"
                              color="success"
                              class="ms-2 download-icon"
                            >
                              mdi-cloud-download
                            </v-icon>
                          </div>
                          <div class="episode-meta d-flex align-center">
                            <div class="meta-item me-4" v-if="episode.duration_seconds">
                              <v-icon size="16" class="me-1">mdi-clock-outline</v-icon>
                              {{ formatDuration(episode.duration_seconds) }}
                            </div>
                            <div class="meta-item">
                              <v-icon size="16" class="me-1">mdi-calendar</v-icon>
                              {{ formatDate(episode.air_date) }}
                            </div>
                            <v-icon
                              size="20"
                              :class="['expand-icon ms-4', { 'expanded': currentExpandedId === episode.id }]"
                            >
                              mdi-chevron-down
                            </v-icon>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <div v-if="currentExpandedId === episode.id" class="episode-content">
                      <div class="pa-6">
                        <!-- 剧集描述 -->
                        <div class="episode-description mb-6" v-if="episode.description">
                          {{ formatDescription(episode.description) }}
                        </div>

                        <!-- 种子列表 -->
                        <div class="torrents-section" v-if="episodeTorrents(episode).length">
                          <div class="section-header d-flex align-center mb-3">
                            <v-icon class="me-2" color="primary" size="18">mdi-download-circle</v-icon>
                            <span class="section-title">可用资源</span>
                          </div>

                          <!-- 按字幕组分组显示 -->
                          <div v-for="group in groupedTorrents(episode)" :key="group.name" class="torrent-group mb-4">
                            <!-- 字幕组标题 -->
                            <div class="group-header d-flex align-center mb-2">
                              <span class="group-name">{{ group.name || '未知字幕组' }}</span>
                              <v-chip
                                size="x-small"
                                color="primary"
                                variant="flat"
                                class="ms-2"
                              >
                                {{ group.torrents.length }}
                              </v-chip>
                            </div>

                            <!-- 种子列表表格 -->
                            <v-table class="torrent-table">
                              <tbody>
                                <tr v-for="torrent in group.torrents" :key="torrent.info_hash" class="torrent-row">
                                  <!-- 标题列 -->
                                  <td class="title-cell">
                                    <div class="torrent-title">{{ torrent.title }}</div>
                                  </td>
                                  
                                  <!-- 信息列 -->
                                  <td class="info-cell">
                                    <div class="d-flex align-center">
                                      <v-chip
                                        v-if="torrent.video_resolution"
                                        size="small"
                                        :color="getResolutionColor(torrent.video_resolution)"
                                        variant="flat"
                                        class="me-2"
                                      >
                                        {{ torrent.video_resolution }}
                                      </v-chip>
                                    </div>
                                  </td>
                                  <td class="info-cell">
                                    <div class="d-flex align-center">
                                      <v-chip
                                        v-if="torrent.size"
                                        size="small"
                                        variant="flat"
                                        class="me-2"
                                      >
                                        {{ formatFileSize(torrent.size)  }}
                                      </v-chip>
                                    </div>
                                  </td>
                                  <td class="info-cell">
                                    <div class="d-flex align-center">
                                      <v-chip
                                        v-if="torrent.language"
                                        size="small"
                                        variant="flat"
                                        class="me-2"
                                      >
                                        {{ formatLanguage(torrent.language) }}
                                      </v-chip>
                                    </div>
                                  </td>
                                  
                                  <!-- 操作列 -->
                                  <td class="action-cell">
                                    <v-btn
                                      :color="getActionButtonColor(torrent.download_status)"
                                      :loading="torrent.download_status === DownloadStatus.Downloading"
                                      :disabled="torrent.download_status === DownloadStatus.Completed"
                                      size="small"
                                      variant="tonal"
                                      class="download-btn"
                                      elevation="0"
                                    >
                                      <template v-if="torrent.download_status === DownloadStatus.Completed">
                                        <v-icon size="16" color="success">mdi-check</v-icon>
                                      </template>
                                      <template v-else>
                                        <v-icon size="16" class="me-1">{{ getActionButtonIcon(torrent.download_status) }}</v-icon>
                                        <span class="btn-text">{{ getActionButtonText(torrent.download_status) }}</span>
                                      </template>
                                    </v-btn>
                                  </td>
                                </tr>
                              </tbody>
                            </v-table>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </v-col>
        </v-row>
      </v-container>
    </div>

    <SubscribeDialog
      v-model="showSubscribeDialog"
      :bangumi-id="anime.id"
      :current-status="anime.subscribe_status || SubscribeStatus.None"
      @subscribe="handleSubscribe"
    />
  </div>
</template>

<style scoped>
.detail-page {
  min-height: 100vh;
  background: #121212;
  position: relative;
}

/* 1. 背景层样式 */
.backdrop-section {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 500px;
  z-index: 1;
}

.backdrop-image {
  width: 100%;
  height: 100%;
}

.backdrop-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.3) 0%,
    rgba(18, 18, 18, 1) 100%
  );
}

/* 2. 内容层样式 */
.content-wrapper {
  position: relative;
  z-index: 2;
  padding-top: 60px;
}

/* 左侧边栏样式 */
.sidebar-col {
  position: relative;
}

.sidebar-content {
  position: sticky;
  top: 24px;
  padding-bottom: 24px;
}

.poster-wrapper {
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
}

.metadata-section {
  margin-top: 24px;
  background: rgba(32, 32, 32, 0.9);
  border-radius: 12px;
  padding: 20px;
}

/* 右侧内容样式 */
.anime-details {
  padding-top: 40px;
  margin-bottom: 60px;
}

.episodes-section {
  background: rgba(255, 255, 255, 0.03);
  border-radius: 16px;
  padding: 24px;
  margin-bottom: 40px;
}

/* 确保文字在背景上清晰可见 */
.text-h3, .text-body-1 {
  color: rgba(255, 255, 255, 0.95);
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.episodes-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.episode-item {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  overflow: hidden;
}

.episode-header {
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 12px;
}

.episode-info {
  font-size: 0.95rem;
}

.meta-item {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.85rem;
  display: flex;
  align-items: center;
}

.episode-name {
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  display: flex;
  align-items: center;
}

.expand-icon {
  transition: transform 0.3s ease;
  color: rgba(255, 255, 255, 0.7);
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.episode-content {
  background: rgba(255, 255, 255, 0.02);
  margin-top: 1px;
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}

.episode-description {
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.6;
  font-size: 0.95rem;
  white-space: pre-line;
}

.episode-header.expanded {
  background: rgba(var(--v-theme-primary), 0.08);
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}

.episode-header:hover {
  background: rgba(255, 255, 255, 0.08);
}

.episode-number {
  font-size: 1.2rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  min-width: 32px;
  text-align: center;
}

.episode-duration {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.6);
  display: flex;
  align-items: center;
}

:deep(.v-expansion-panel-title) {
  min-height: unset !important;
}

:deep(.v-expansion-panel-title__overlay) {
  background: rgba(var(--v-theme-surface-variant), 0.08) !important;
}

:deep(.v-expansion-panel-title--active) {
  min-height: unset !important;
  background: rgba(var(--v-theme-primary), 0.08) !important;
}

:deep(.v-expansion-panel-text__wrapper) {
  padding: 0 !important;
}

/* 添加淡入动画 */
.fade-transition-enter-active,
.fade-transition-leave-active {
  transition: opacity 0.2s ease;
}

.fade-transition-enter-from,
.fade-transition-leave-to {
  opacity: 0;
}

/* 展开动画 */
:deep(.v-expand-transition-enter-active),
:deep(.v-expand-transition-leave-active) {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

:deep(.v-expand-transition-enter-from),
:deep(.v-expand-transition-leave-to) {
  opacity: 0;
  transform: translateY(-8px);
}

.staff-content {
  font-size: 0.875rem;
  line-height: 1.8;
  color: rgba(255, 255, 255, 0.7);
  white-space: pre-line;
  font-family: monospace;
}

/* 添加分隔线 */
.episode-staff {
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 1rem;
}

.info-sidebar {
  position: static;
  overflow: visible;
}

/* 自定义滚动条样式 */
.info-sidebar::-webkit-scrollbar {
  width: 6px;
}

.info-sidebar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}

.info-sidebar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

/* 右下角订阅按钮 */
.subscribe-wrapper {
  display: none;
}

.subscribe-btn {
  width: 36px !important;
  height: 36px !important;
  min-width: 36px !important;
  padding: 0 !important;
  border-radius: 50% !important;
  transition: all 0.3s ease;
}

.subscribe-btn:not(.v-btn--variant-tonal) {
  background: rgba(255, 255, 255, 0.05);
  color: white;
}

.subscribe-btn:hover {
  transform: scale(1.1);
}

.subscribe-btn.v-btn--variant-tonal {
  color: white;
}

/* 右上角彩带 */
.ribbon-wrapper {
  width: 150px;
  height: 150px;
  overflow: hidden;
  position: absolute;
  top: -10px;
  right: -10px;
  z-index: 2;
}

.ribbon {
  font-size: 0.75rem;
  font-weight: 500;
  color: white;
  text-align: center;
  transform: rotate(45deg);
  position: relative;
  padding: 6px 0;
  left: -10px;
  top: 30px;
  width: 200px;
  background: rgba(var(--v-theme-primary), 0.9);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

/* 彩带两端的装饰 */
.ribbon::before,
.ribbon::after {
  content: '';
  position: absolute;
  border-top: 3px solid rgba(var(--v-theme-primary), 1);
  border-left: 3px solid transparent;
  border-right: 3px solid transparent;
  bottom: -3px;
}

.ribbon::before {
  left: 0;
}

.ribbon::after {
  right: 0;
}

/* 彩带悬停效果 */
.ribbon:hover {
  background: rgba(var(--v-theme-primary), 1);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
}

.container-position {
  position: relative;
  z-index: 2;
  max-width: 1400px;
  margin: 0 auto;
}

/* 文字阴影 */
.text-h3, .text-h5, .text-body-1, .text-subtitle-1 {
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* 亮色文本 */
.text-h3, .text-h5, .text-body-1 {
  color: rgba(255, 255, 255, 0.95);
}

/* 修改扩展面板的样式 */
:deep(.v-expansion-panels) {
  background: transparent !important;
}

:deep(.v-expansion-panel) {
  background: rgba(255, 255, 255, 0.03) !important;
  border: 1px solid rgba(255, 255, 255, 0.08);
  margin-bottom: 8px;
}

:deep(.v-expansion-panel-title) {
  padding: 12px 16px;
  min-height: unset !important;
  color: rgba(255, 255, 255, 0.9) !important;
}

:deep(.v-expansion-panel-title__overlay) {
  background: rgba(255, 255, 255, 0.05) !important;
}

:deep(.v-expansion-panel-title--active) {
  background: rgba(var(--v-theme-primary), 0.08) !important;
}

:deep(.v-expansion-panel-text) {
  color: rgba(255, 255, 255, 0.8) !important;
}

:deep(.v-expansion-panel-text__wrapper) {
  padding: 0 !important;
}

/* 修改芯片样式 */
:deep(.v-chip) {
  color: rgba(255, 255, 255, 0.9) !important;
  background: rgba(var(--v-theme-primary), 0.15) !important;
}

:deep(.v-chip--variant-flat) {
  opacity: 0.9;
}

/* 修改按钮样式 */
:deep(.v-btn) {
  text-transform: none !important;
}

:deep(.v-btn--variant-tonal) {
  background: rgba(var(--v-theme-primary), 0.15) !important;
  color: rgba(255, 255, 255, 0.9) !important;
}

:deep(.v-btn--variant-tonal:hover) {
  background: rgba(var(--v-theme-primary), 0.25) !important;
}

/* 修改图标颜色 */
:deep(.v-icon) {
  color: inherit;
  opacity: 0.9;
}

/* 修改分隔线样式 */
.section-divider {
  height: 1px;
  background: linear-gradient(
    to right,
    rgba(var(--v-theme-primary), 0.05),
    rgba(var(--v-theme-primary), 0.2),
    rgba(var(--v-theme-primary), 0.05)
  );
  margin: 16px 0;
}

/* 修改详情网格样式 */
.details-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 12px;
  padding: 0 4px;
}

.detail-item {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 12px 16px;
  transition: background-color 0.3s ease;
}

.detail-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

/* 修改状态指示器样式 */
.status-indicator {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.05);
  transition: all 0.3s ease;
}

.status-indicator:hover {
  transform: scale(1.1);
  background: rgba(255, 255, 255, 0.08);
}

/* 修改操作按钮区域样式 */
.torrent-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.2);
  margin: 16px -16px -16px -16px;
  border-bottom-left-radius: 8px;
  border-bottom-right-radius: 8px;
}

/* 修改种子标题样式 */
.torrent-title {
  font-size: 0.95rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  margin-right: 8px;
}

/* 修改元信息样式 */
.meta-item {
  display: flex;
  align-items: center;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.6);
  margin-right: 16px;
}

.meta-item:last-child {
  margin-right: 0;
}

.meta-item .v-icon {
  margin-right: 4px;
  opacity: 0.7;
}

.info-section {
  margin: 20px 0;
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-card {
  background: rgba(32, 32, 32, 0.9);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.info-icon {
  width: 36px;
  height: 36px;
  min-width: 36px;
  background: rgba(48, 48, 48, 0.9);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.info-icon :deep(.v-icon) {
  color: rgba(255, 255, 255, 0.9);
  font-size: 20px;
}

.info-content {
  flex: 1;
  min-width: 0;
}

.info-label {
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.75rem;
  margin-bottom: 2px;
}

.info-value {
  color: rgba(255, 255, 255, 0.9);
  font-size: 0.875rem;
  font-weight: 500;
  line-height: 1.2;
}

/* 确保外部链接按钮在深色背景上可见 */
:deep(.v-btn.v-btn--variant-outlined) {
  border-color: none;
}

.external-links {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.link-btn {
  width: 100%;
  justify-content: flex-start;
  padding: 12px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 500;
  text-transform: none;
  letter-spacing: normal;
}

.link-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

.link-id {
  margin-left: auto;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.6);
}

.rating-section {
  background: rgba(32, 32, 32, 0.9);
  border-radius: 12px;
  padding: 16px;
}

/* 更新下载图标样式 */
:deep(.v-icon.download-icon) {
  opacity: 0.9;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
}

/* 添加悬停提示的样式 */
.download-icon {
  transition: transform 0.2s ease;
}

.download-icon:hover {
  transform: scale(1.1);
}

.torrents-section {
  background: rgba(18, 18, 18, 0.4);
  border-radius: 8px;
  padding: 16px;
}

.section-header {
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.section-title {
  font-size: 0.9rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
}

.torrent-list {
  margin-top: 12px;
}

.torrent-item {
  display: grid;
  grid-template-columns: 200px auto 120px;
  gap: 16px;
  align-items: center;
  padding: 8px 12px;
  border-radius: 6px;
  transition: background-color 0.2s ease;
}

.torrent-item:hover {
  background: rgba(255, 255, 255, 0.03);
}

.release-group {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.torrent-info {
  min-width: 0;
}

.file-size {
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.6);
}

.download-btn {
  min-width: 85px;
  height: 32px !important;
  font-weight: 500;
  letter-spacing: 0.3px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.download-btn:not(:disabled) {
  background: rgba(var(--v-theme-primary), 0.15) !important;
}

.download-btn:hover:not(:disabled) {
  background: rgba(var(--v-theme-primary), 0.25) !important;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.download-btn:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: none;
}

.download-btn.v-btn--disabled {
  background: rgba(255, 255, 255, 0.05) !important;
  opacity: 0.7;
  border: none;
}

.btn-text {
  font-size: 0.8rem;
  font-weight: 500;
}

/* 加载状态样式 */
.download-btn.v-btn--loading {
  background: rgba(var(--v-theme-primary), 0.1) !important;
}

/* 成功状态样式 */
.download-btn.v-btn--disabled .v-icon {
  opacity: 0.9;
}

/* 响应式调整 */
@media (max-width: 600px) {
  .download-btn {
    width: 100%;
    height: 36px !important;
  }
}

.torrent-group {
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  overflow: hidden;
}

.group-header {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.group-name {
  font-size: 0.9rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
}

.torrent-table {
  width: 100%;
  background: transparent !important;
}

.torrent-row {
  transition: background-color 0.2s ease;
  height: 52px; /* 增加行高 */
}

.torrent-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.title-cell {
  padding: 12px 16px;
  width: 75%;
  vertical-align: middle;
}

.info-cell {
  padding: 12px;
  width: 15%;
  vertical-align: middle;
  white-space: nowrap;
}

.action-cell {
  padding: 12px;
  width: 10%;
  text-align: right;
  vertical-align: middle;
  white-space: nowrap;
}

.torrent-title {
  font-size: 0.875rem;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.9);
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.file-size {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.7);
}

.download-btn {
  min-width: 90px;
  height: 32px !important;
}

.btn-text {
  font-size: 0.8rem;
}

:deep(.v-chip) {
  font-size: 0.75rem !important;
  height: 24px !important;
  font-weight: 500;
}

/* 响应式布局 */
@media (max-width: 800px) {
  .title-cell {
    width: 70%;
  }
  
  .info-cell {
    width: 20%;
  }
  
  .action-cell {
    width: 10%;
  }
}

@media (max-width: 600px) {
  .torrent-table {
    display: block;
  }
  
  .torrent-row {
    display: flex;
    flex-direction: column;
    height: auto;
    padding: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  
  .title-cell,
  .info-cell,
  .action-cell {
    width: 100%;
    padding: 6px 0;
    text-align: left;
  }
  
  .torrent-title {
    margin-bottom: 8px;
  }
  
  .action-cell {
    margin-top: 8px;
  }
  
  .download-btn {
    width: 100%;
    height: 36px !important;
  }
}

/* 封面图片悬浮层样式 */
.image-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0.2) 0%,
    rgba(0, 0, 0, 0.6) 100%
  );
  opacity: 0;
  transition: opacity 0.3s ease;
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  padding: 16px;
}

.poster-wrapper:hover .image-overlay {
  opacity: 1;
}

.action-buttons {
  display: flex;
  gap: 8px;
}

.action-btn {
  color: white;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  width: 36px !important;
  height: 36px !important;
  min-width: 36px !important;
  border-radius: 50% !important;
  padding: 0 !important;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.7);
  transform: scale(1.1);
}

.action-btn :deep(.v-icon.text-error) {
  color: rgb(var(--v-theme-error));
}

.action-btn :deep(.v-btn__content) {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>

<script lang="ts" setup>
import { ref, computed, onMounted, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { 
  getBangumiById, 
  subscribeBangumi, 
  getBangumiEpisodes,
  getBangumiTorrents,
  refreshBangumi
} from '@/api/api'
import { 
  DownloadStatus, 
  State,
  type Episode, 
  type Bangumi, 
  SubscribeStatus, 
  type Torrent,
  type SubscribeParams 
} from '@/api/model'
import { useSnackbar } from '../composables/useSnackbar'
import SubscribeDialog from '../components/SubscribeDialog.vue'

const route = useRoute()
const anime = ref<Bangumi>()
const showSubscribeDialog = ref(false)
const { showSnackbar } = useSnackbar()

// 评分转换为5分制
const ratingValue = computed(() => anime.value?.rating ? anime.value.rating / 2 : 0)

// 添加订阅状态
const isSubscribed = ref(false)

// 切换订阅状态的方法
const toggleSubscribe = () => {
  if (!anime.value) return
  showSubscribeDialog.value = true
}

// 获取详情数据
const fetchAnimeDetail = async () => {
  try {
    const id = Number(route.params.id)
    if (!id) return
    
    anime.value = await getBangumiById(id)
    isSubscribed.value = anime.value.subscribe_status === 'Subscribed'
  } catch (error) {
    console.error('获取番剧详情失败:', error)
    // TODO: 添加错误提示
  }
}

const episodes = ref<Episode[]>([])

// 获取剧集列表
const fetchEpisodes = async () => {
  try {
    const id = Number(route.params.id)
    if (!id) return
    
    episodes.value = await getBangumiEpisodes(id)
  } catch (error) {
    console.error('获取剧集列表失败:', error)
    // TODO: 添加错误提示
  }
}

// 判断是否是新剧集（7天内）
const isNewEpisode = (airDate: string) => {
  const date = new Date(airDate)
  const now = new Date()
  const diffDays = Math.ceil((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))
  return diffDays <= 7
}

// 添加刷新状态
const isRefreshing = ref(false)

// 处理刷新操作
const handleRefresh = async () => {
  if (!anime.value || isRefreshing.value) return
  
  try {
    isRefreshing.value = true
    // 调用刷新 API
    await refreshBangumi(anime.value.id)
    // 重新加载数据
    await Promise.all([
      fetchAnimeDetail(),
      fetchEpisodes(),
      fetchTorrents()
    ])
    // 显示成功提示
    showSnackbar({
      text: '已经加入刷新队列',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
  } catch (error) {
    console.error('刷新失败:', error)
    // 显示错误提示
    showSnackbar({
      text: '刷新失败',
      color: 'error',
      location: 'top right',
      timeout: 3000
    })
  } finally {
    isRefreshing.value = false
  }
}

// 修复 formatDate 函数的类型错误
const formatDate = (date: string | null) => {
  if (!date) return '暂无'
  return new Date(date).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long'
  })
}

const downloadResource = (resource: any) => {
  // 实现下载逻辑
  console.log('Downloading:', resource.name)
}

// 格式化时长
const formatDuration = (seconds: number) => {
  const minutes = Math.floor(seconds / 60)
  return `${minutes}分钟`
}

// 使用普通的 ref 来存储当前展开的剧集 ID
const currentExpandedId = ref<number | null>(null)

// 切换展开状态
const toggleEpisode = (id: number) => {
  currentExpandedId.value = currentExpandedId.value === id ? null : id
}

// 格式化剧集描述
const formatDescription = (description: string) => {
  // 分割描述和制作信息
  const parts = description.split('\r\n\r\n')
  // 返回第一部分作为剧集描述
  return parts[0]
}

// 格式化制作信息
const formatStaffInfo = (description: string) => {
  const parts = description.split('\r\n\r\n')
  // 如果有多个部分，最后一部分通常是制作信息
  if (parts.length > 1) {
    return parts[parts.length - 1]
  }
  return ''
}

// 存储所有种子信息
const torrents = ref<Torrent[]>([])

// 获取种子列表
const fetchTorrents = async () => {
  try {
    const id = Number(route.params.id)
    if (!id) return
    torrents.value = await getBangumiTorrents(id)
  } catch (error) {
    console.error('获取种子列表失败:', error)
  }
}

// 根据剧集过滤种子
const episodeTorrents = (episode: Episode) => {
  return torrents.value.filter(t => t.episode_number === episode.number)
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

const formatLanguage = (language: string) => {
  return language.split(',').map(lang => 
    lang.trim() === 'CHS' ? '简中' : 
    lang.trim() === 'CHT' ? '繁中' : 
    lang.trim() === 'ENG' ? '英语' : 
    lang.trim() === 'JPN' ? '日语' : 
    lang.trim() === 'KOR' ? '韩语' : 
    lang.trim() === 'RUS' ? '俄语' :
    lang.trim()
  ).join(', ')
}

// 获取分辨率颜色
const getResolutionColor = (resolution: string) => {
  switch (resolution.toLowerCase()) {
    case '1080p':
      return 'primary'
    case '720p':
      return 'info'
    case '2160p':
    case '4k':
      return 'success'
    default:
      return 'grey'
  }
}

// 获取状态图标
const getStatusIcon = (status: string | null) => {
  switch (status) {
    case 'Pending':
      return 'mdi-progress-download'
    case 'Downloading':
      return 'mdi-check-circle'
    case 'Failed':
      return 'mdi-alert-circle'
    default:
      return 'mdi-download-circle-outline'
  }
}

// 获取状态颜色
const getStatusColor = (status: string | null) => {
  switch (status) {
    case DownloadStatus.Pending:
      return 'info'
    case DownloadStatus.Downloading:
      return 'success'
    case DownloadStatus.Failed:
      return 'error'
    default:
      return 'grey'
  }
}

// 格式化下载状态
const formatDownloadStatus = (status: string | null) => {
  switch (status) {
    case DownloadStatus.Downloading:
      return '下载中'
    case DownloadStatus.Completed:
      return '已完成'
    case DownloadStatus.Failed:
      return '下载失败'
    case DownloadStatus.Pending:
      return '等待下载'
    default:
      return '未下载'
  }
}

// 格式化日期时间
const formatDateTime = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
}

// 复制磁力链接
const copyMagnet = (magnet: string) => {
  navigator.clipboard.writeText(magnet)
  // TODO: 添加复制成功提示
}

// 下载种子
const downloadTorrent = (torrent: Torrent) => {
  // TODO: 实现下载逻辑
  console.log('下载种子:', torrent)
}

// 获取操作按钮颜色
const getActionButtonColor = (status: string | null) => {
  switch (status) {
    case 'downloading':
      return 'primary'
    case 'completed':
      return 'success'
    case 'failed':
      return 'error'
    default:
      return 'primary'
  }
}

// 获取操作按钮图标
const getActionButtonIcon = (status: string | null) => {
  switch (status) {
    case 'downloading':
      return 'mdi-progress-download'
    case 'completed':
      return 'mdi-check'
    case 'failed':
      return 'mdi-refresh'
    default:
      return 'mdi-download'
  }
}

// 获取操作按钮文本
const getActionButtonText = (status: string | null) => {
  switch (status) {
    case 'downloading':
      return '下载中'
    case 'completed':
      return '已完成'
    case 'failed':
      return '重试'
    default:
      return '下载'
  }
}

// 按字幕组分组的方法
const groupedTorrents = (episode: Episode) => {
  const torrents = episodeTorrents(episode)
  const groups = new Map<string, Torrent[]>()
  
  torrents.forEach(torrent => {
    const groupName = torrent.release_group || '未知字幕组'
    if (!groups.has(groupName)) {
      groups.set(groupName, [])
    }
    groups.get(groupName)?.push(torrent)
  })
  
  return Array.from(groups.entries()).map(([name, torrents]) => ({
    name,
    torrents: torrents.sort((a, b) => {
      // 按分辨率和文件大小排序
      const resA = a.video_resolution || ''
      const resB = b.video_resolution || ''
      if (resA !== resB) return resB.localeCompare(resA)
      return b.size - a.size
    })
  }))
}

async function handleSubscribe(params: SubscribeParams) {
  if (!anime.value) return
  console.log('订阅参数:', params)
  try {
    // 调用订阅 API
    await subscribeBangumi(anime.value.id, params)
    // 重新加载番剧信息以更新订阅状态
    anime.value = await getBangumiById(anime.value.id)
    // 更新本地订阅状态
    isSubscribed.value = params.status === SubscribeStatus.Subscribed
    // 显示成功提示
    showSnackbar({
      text: params.status === SubscribeStatus.Subscribed ? '订阅成功' : '取消订阅成功',
      color: 'success',
      location: 'top right',
      timeout: 3000
    })
  } catch (error) {
    console.error('订阅操作失败:', error)
  }
}

onMounted(() => {
  fetchAnimeDetail()
  fetchEpisodes()
  fetchTorrents()
})
</script>


