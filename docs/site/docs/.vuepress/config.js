 import { defaultTheme } from '@vuepress/theme-default'
import { defineUserConfig } from 'vuepress/cli'
import { viteBundler } from '@vuepress/bundler-vite'

export default defineUserConfig({
  lang: 'zh-CN',
  title: 'Bangumi-rs',
  description: '基于 Rust + Vue 3 开发的动漫追番工具',

  theme: defaultTheme({
    logo: '/images/home.png',
    navbar: [
      {
        text: '指南',
        link: '/guide/',
      },
      {
        text: '配置',
        link: '/config/',
      },
      {
        text: '开发',
        link: '/development/',
      },
      {
        text: 'GitHub',
        link: 'https://github.com/lyqingye/bangumi-rs'
      }
    ],
    sidebar: {
      '/guide/': [
        {
          text: '指南',
          children: [
            '/guide/README.md',
            '/guide/getting-started.md',
          ],
        },
      ],
      '/config/': [
        {
          text: '配置',
          children: [
            '/config/README.md',
          ],
        },
      ],
      '/development/': [
        {
          text: '开发',
          children: [
            '/development/README.md',
          ],
        },
      ],
    },
    editLink: false,
    lastUpdated: true,
    contributors: false,
  }),

  bundler: viteBundler(),
})