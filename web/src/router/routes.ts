export const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/pages/index.vue')
  },
  {
    path: '/detail/:id',
    name: 'detail',
    component: () => import('@/pages/detail.vue')
  }
] 