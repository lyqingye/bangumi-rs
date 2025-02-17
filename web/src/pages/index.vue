<template>
  <v-layout class="fill-height">
    <!-- 左侧导航树 -->
    <v-navigation-drawer permanent class="navigation-drawer" elevation="0" width="280">
      <!-- 毛玻璃背景 -->
      <div class="glass-bg"></div>

      <!-- 导航内容 -->
      <div class="nav-content">
        <div class="pa-6">
          <div class="text-h5 font-weight-semibold">Bangumi App</div>
        </div>

        <v-list class="nav-list px-3" nav>
          <v-list-item
            v-for="(item, i) in navItems"
            :key="i"
            :value="item.to"
            :prepend-icon="item.icon"
            :title="item.title"
            :to="item.to"
            :active="currentRoute === item.to"
            class="nav-item mb-2"
            rounded="lg"
          >
            <template #title>
              <span class="font-weight-medium">{{ item.title }}</span>
            </template>
          </v-list-item>
        </v-list>
      </div>
    </v-navigation-drawer>

    <!-- 主要内容区域 -->
    <v-main class="fill-height main-content">
      <router-view />
    </v-main>
  </v-layout>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const currentRoute = computed(() => route.path)

const navItems = [
  { title: '首页', icon: 'mdi-home', to: '/' },
  { title: '下载', icon: 'mdi-download', to: '/downloads' },
]
</script>

<style scoped>
.navigation-drawer {
  position: relative;
  border: none;
  overflow: hidden;
}

/* 毛玻璃背景 */
.glass-bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(24, 24, 24, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255, 255, 255, 0.1);
}

/* 导航内容容器 */
.nav-content {
  position: relative;
  z-index: 1;
  height: 100%;
}

.nav-list {
  background: transparent;
}

.nav-item {
  min-height: 48px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
  margin-bottom: 8px;
  padding: 0 16px;
}

.nav-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.05);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.nav-item:hover::before {
  opacity: 1;
}

.nav-item.v-list-item--active {
  background: rgba(var(--v-theme-primary), 0.15);
  color: rgb(var(--v-theme-primary));
}

.nav-item.v-list-item--active::before {
  display: none;
}

/* 优化导航项样式 */
.nav-item :deep(.v-list-item__content) {
  font-size: 15px !important;
  letter-spacing: 0.025em;
  line-height: 1.5;
}

.nav-item :deep(.v-list-item__content .font-weight-medium) {
  font-size: 15px !important;
}

/* 优化图标样式 */
.nav-item :deep(.v-list-item__prepend) {
  opacity: 0.9;
  font-size: 1.25rem;
  padding-right: 20px;
}

.nav-item.v-list-item--active :deep(.v-list-item__prepend) {
  opacity: 1;
}

.nav-item.v-list-item--active :deep(.v-list-item__content) {
  font-weight: unset;
}

.nav-item.v-list-item--active :deep(.font-weight-medium) {
  font-weight: 600 !important;
}

/* 优化应用标题 */
.text-h5 {
  font-size: 28px !important;
  letter-spacing: 0.025em;
  line-height: 1.4;
}

.main-content {
  background: rgb(16, 16, 16);
}

/* 优化悬停效果 */
.nav-item:hover :deep(.v-list-item__content) {
  transform: translateX(4px);
  transition: transform 0.3s ease;
}

.nav-item :deep(.v-list-item__content) {
  transform: translateX(0);
  transition: transform 0.3s ease;
}
</style>
