<template>
  <v-layout class="fill-height">
    <!-- 左侧导航树 -->
    <v-navigation-drawer
      v-model="drawer"
      :rail="rail"
      permanent
      class="navigation-drawer"
      elevation="0"
      :width="280"
      :rail-width="56"
    >
      <!-- 毛玻璃背景 -->
      <div class="glass-bg"></div>

      <!-- 导航内容 -->
      <div class="nav-content">
        <!-- 顶部标题和切换按钮 -->
        <div class="nav-header">
          <div class="app-title" v-show="!rail">Bangumi App</div>
          <v-btn
            variant="text"
            :icon="rail ? 'mdi-menu' : 'mdi-menu-open'"
            size="small"
            @click.stop="rail = !rail"
          ></v-btn>
        </div>

        <!-- 导航菜单 -->
        <div class="nav-menu">
          <div
            v-for="(item, i) in navItems"
            :key="i"
            class="nav-item"
            :class="{ active: currentRoute === item.to }"
            @click="$router.push(item.to)"
          >
            <v-icon :icon="item.icon" class="nav-icon" />
            <span class="nav-text" v-show="!rail">{{ item.title }}</span>
          </div>
        </div>
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
  { title: '仪表盘', icon: 'mdi-chart-box', to: '/dashboard' },
  { title: '设置', icon: 'mdi-cog', to: '/settings' }
]

const drawer = ref(true)
const rail = ref(true)
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
  display: flex;
  flex-direction: column;
}

/* 导航头部 */
.nav-header {
  padding: 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.app-title {
  font-size: 24px;
  font-weight: 600;
  letter-spacing: 0.025em;
}

/* 导航菜单 */
.nav-menu {
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  height: 44px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  user-select: none;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.nav-item.active {
  background: rgba(var(--v-theme-primary), 0.15);
  color: rgb(var(--v-theme-primary));
}

.nav-icon {
  font-size: 24px;
}

.nav-text {
  margin-left: 12px;
  font-weight: 500;
}

/* 锁进按钮动画 */
.rotate-180 {
  transform: rotate(180deg);
}

/* 锁进模式样式 */
:deep(.v-navigation-drawer--rail) {
  .nav-menu {
    padding: 0;
  }

  .nav-item {
    padding: 0;
    justify-content: center;
  }

  .nav-icon {
    margin: 0;
  }
}

/* 主内容区域 */
.main-content {
  background: rgb(16, 16, 16);
}
</style>

