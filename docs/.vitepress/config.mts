import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Bangumi-rs",
  description: "基于 Rust + Vue 3 开发的动漫追番工具",
  lang: 'zh-CN',
  lastUpdated: true,
  base: '/bangumi-rs/',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/logo.png',
    
    // 右上角导航栏，只保留三个主要入口
    nav: [
      { text: '项目说明', link: '/introduction/' },
      { text: '快速开始', link: '/quickstart/' },
      { text: '常见问题', link: '/faq/' },
    ],

    // 左侧边栏，包含所有栏目
    sidebar: [
      {
        text: '项目说明',
        collapsed: false,
        items: [
          { text: '项目介绍', link: '/introduction/' },
          { text: '技术架构', link: '/introduction/architecture' },
          { text: '项目特性', link: '/introduction/features' },
        ]
      },
      {
        text: '快速开始',
        collapsed: false,
        items: [
          { text: '环境准备', link: '/quickstart/' },
          { text: '基本使用', link: '/quickstart/basic-usage' },
          { text: '界面预览', link: '/quickstart/ui-preview' },
        ]
      },
      {
        text: '部署',
        collapsed: false,
        items: [
          { text: '部署概述', link: '/deploy/' },
          { text: '本地部署', link: '/deploy/binary' },
          { text: 'Docker 部署', link: '/deploy/docker' },
          { text: '源码部署', link: '/deploy/source' },
        ]
      },
      {
        text: '配置说明',
        collapsed: false,
        items: [
          { text: '配置概览', link: '/configuration/' },
          { text: '服务器配置', link: '/configuration/server' },
          { text: '资源站点配置', link: '/configuration/sites' },
          { text: '解析器配置', link: '/configuration/parser' },
          { text: '下载器配置', link: '/configuration/downloader' },
          { text: '通知配置', link: '/configuration/notification' },
          { text: '代理配置', link: '/configuration/proxy' },
        ]
      },
      {
        text: '功能说明',
        collapsed: false,
        items: [
          { text: '功能概览', link: '/features/' },
          { text: '番剧订阅', link: '/features/subscription' },
          { text: '智能解析', link: '/features/parser' },
          { text: '智能下载', link: '/features/download' },
          { text: '在线播放', link: '/features/play' },
          { text: '元数据管理', link: '/features/metadata' },
          { text: '通知提醒', link: '/features/notification' },
        ]
      },
      {
        text: '常见问题',
        collapsed: false,
        items: [
          { text: '常见问题', link: '/faq/' },
          { text: '安装问题', link: '/faq/installation' },
          { text: '配置问题', link: '/faq/configuration' },
          { text: '使用问题', link: '/faq/usage' },
        ]
      },
      {
        text: '更新日志',
        collapsed: false,
        items: [
          { text: '版本历史', link: '/changelog/' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/lyqingye/bangumi-rs' }
    ],
    
    footer: {
      message: '基于 MIT 许可发布',
      copyright: 'Copyright © 2023-present lyqingye'
    },
    
    search: {
      provider: 'local'
    },
  }
})
