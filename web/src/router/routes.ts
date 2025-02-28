export const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/pages/index.vue'),
    children: [
      {
        path: '',
        name: 'home-content',
        component: () => import('@/pages/home.vue')
      },
      {
        path: 'downloads',
        name: 'downloads',
        component: () => import('@/pages/downloads.vue')
      },
      {
        path: 'dashboard',
        name: 'dashboard',
        component: () => import('@/pages/dashboard.vue')
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@/pages/settings.vue')
      },
      {
        path: 'detail/:id',
        name: 'detail',
        component: () => import('@/pages/detail.vue')
      },
      {
        path: 'bangumi-list',
        name: 'bangumi-list',
        component: () => import('@/pages/bangumi-list.vue')
      }
    ]
  },
]
