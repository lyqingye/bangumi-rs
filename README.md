<div align="center">

# Bangumi-rs

</div>

<div align="center">

[![Lint](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml) [![Release](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml) [![Docker](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml) ![Version](https://img.shields.io/github/v/release/lyqingye/bangumi-rs.svg?color=blue&logo=github)

</div>

<div align="center">

<a href="https://lyqingye.github.io/bangumi-rs/">
  <img src="https://img.shields.io/badge/官方文档-4285F4?style=flat-square&logo=google-docs&logoColor=white&labelColor=4285F4&borderRadius=12">
</a>
<a href="https://lyqingye.github.io/bangumi-rs/quickstart/">
  <img src="https://img.shields.io/badge/快速开始-34A853?style=flat-square&logo=clockify&logoColor=white&borderRadius=12">
</a>

</div>

<div align="center", style = "margin-top: 20px;">
<img src="docs/public/screenshot/home.png" width="80%" alt="home" style="box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7); border-radius: 8px; margin: 40px 0;" />

</div>

<p>
基于 Rust + Vue 3 开发的动漫追番工具，支持订阅番剧、智能下载、在线播放等功能。
</p>

## 功能特性

- 🎯 番剧订阅

  - 支持订阅/取消订阅番剧
  - 可配置分辨率、字幕组、字幕语言过滤条件
  - 支持自定义订阅更新间隔

- 🌐 资源站点

  - Mikan (https://mikanani.me/)

- 🔍 资源解析

  - 采用传统方式解析文件名
  - 采用 ChatGPT 进行解析，支持多个 API 服务提供商 (SiliconFlow、OpenAI、Claude、DeepSeek)

- 📥 智能下载

  - 支持剧集偏移
  - 自动选择最佳种子 (根据分辨率以及语言字幕优先选择)
  - 支持用户手动选择要下载的剧集种子
  - 支持 115 网盘 离线下载
  - 下载失败后会自动尝试其它种子

- 🎬 在线播放

  - 支持 IINA,Infuse 播放器 在线播放

- 📚 元数据管理

  - 自动获取番剧信息
  - 支持从多个数据源获取(TMDB、Bangumi.tv、Mikan)
  - 支持手动刷新元数据
  - 支持剧集、海报墙、封面等信息显示

- 🔔 通知提醒
  - 支持 Telegram 通知

## 许可证

MIT License

