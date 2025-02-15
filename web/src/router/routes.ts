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
        path: 'movies',
        name: 'movies',
        component: () => import('@/pages/movies.vue')
      },
      {
        path: 'tv',
        name: 'tv',
        component: () => import('@/pages/tv.vue')
      },
      {
        path: 'anime',
        name: 'anime',
        component: () => import('@/pages/anime.vue')
      }
    ]
  },
  {
    path: '/detail/:id',
    name: 'detail',
    component: () => import('@/pages/detail.vue')
  }
]
