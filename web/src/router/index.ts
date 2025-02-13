/**
 * router/index.ts
 *
 * Automatic routes for `./src/pages/*.vue`
 */

// Composables
import { createRouter, createWebHistory } from 'vue-router'
import { routes } from './routes'

// 打印一下路由信息，帮助调试
console.log('Current routes:', routes)

// 确保 detail 路由存在并正确配置
const detailRoute = routes.find(route => route.name === 'detail')
if (detailRoute) {
  detailRoute.path = '/detail/:id'
} else {
  // 如果没有找到 detail 路由，手动添加
  routes.push({
    name: 'detail',
    path: '/detail/:id',
    component: () => import('@/pages/detail.vue')
  })
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

// Workaround for https://github.com/vitejs/vite/issues/11804
router.onError((err, to) => {
  if (err?.message?.includes?.('Failed to fetch dynamically imported module')) {
    if (!localStorage.getItem('vuetify:dynamic-reload')) {
      console.log('Reloading page to fix dynamic import error')
      localStorage.setItem('vuetify:dynamic-reload', 'true')
      location.assign(to.fullPath)
    } else {
      console.error('Dynamic import error, reloading page did not fix it', err)
    }
  } else {
    console.error(err)
  }
})

router.isReady().then(() => {
  localStorage.removeItem('vuetify:dynamic-reload')
})

export default router
