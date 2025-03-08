<div align="center">

# 🌟 Bangumi-rs

</div>

<div align="center">

[![Lint](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/checks.yml) [![Release](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/release.yml) [![Docker](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml/badge.svg)](https://github.com/lyqingye/bangumi-rs/actions/workflows/docker.yml) ![Version](https://img.shields.io/github/v/release/lyqingye/bangumi-rs.svg?color=blue&logo=github) ![License](https://img.shields.io/badge/license-MIT-green.svg)

</div>

<div align="center">

<a href="https://lyqingye.github.io/bangumi-rs/">
  <img src="https://img.shields.io/badge/官方文档-4285F4?style=for-the-badge&logo=google-docs&logoColor=white&labelColor=4285F4">
</a>
<a href="https://lyqingye.github.io/bangumi-rs/quickstart/">
  <img src="https://img.shields.io/badge/快速开始-34A853?style=for-the-badge&logo=clockify&logoColor=white">
</a>
<a href="https://github.com/lyqingye/bangumi-rs/issues">
  <img src="https://img.shields.io/badge/问题反馈-EA4335?style=for-the-badge&logo=github&logoColor=white">
</a>

</div>

<br>

<div align="center">
<img src="docs/public/screenshot/home.png" width="90%" alt="home" style="box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7); border-radius: 12px; margin: 20px 0;" />
</div>

<p align="center">
<b>Bangumi-rs</b> 是一款功能强大的动漫追番工具，支持订阅番剧、智能下载、在线播放等功能。<br>
采用 <b>Rust</b> 后端 + <b>Vue 3</b> 前端开发，高效稳定，界面美观。
</p>
<br>

## ✨ 功能特性

<table>
<tr>
<td width="50%">

### 🎯 番剧订阅

- ✅ 支持订阅/取消订阅番剧
- ✅ 可配置分辨率、字幕组、字幕语言过滤条件
- ✅ 支持自定义订阅更新间隔

### 🌐 资源站点

- ✅ Mikan (https://mikanani.me/)
- 🔜 更多站点支持中...

### 🔍 资源解析

- ✅ 采用传统方式解析文件名
- ✅ 采用 ChatGPT 进行解析
- ✅ 支持多个 API 服务提供商:
  - SiliconFlow
  - OpenAI
  - DeepSeek

</td>
<td width="50%">

### 📥 智能下载

- ✅ 支持剧集偏移
- ✅ 自动选择最佳种子
- ✅ 支持用户手动选择要下载的剧集种子
- ✅ 支持 115 网盘离线下载
- ✅ 下载失败后会自动尝试其它种子

### 🎬 在线播放

- ✅ 支持 IINA, Infuse 播放器在线播放

### 📚 元数据管理

- ✅ 自动获取番剧信息
- ✅ 支持从多个数据源获取(TMDB、Bangumi.tv、Mikan)
- ✅ 支持手动刷新元数据
- ✅ 支持剧集、海报墙、封面等信息显示

### 🔔 通知提醒

- ✅ 支持 Telegram 通知

</td>
</tr>
</table>

## 🖼️ 更多截图

<div align="center">
<table>
<tr>
<td><img src="docs/public/screenshot/detail.png" alt="详情页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
<td><img src="docs/public/screenshot/subscribe.png" alt="订阅页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
</tr>
<tr>
<td><img src="docs/public/screenshot/settings.png" alt="设置页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
<td><img src="docs/public/screenshot/download.png" alt="下载页" style="border-radius: 12px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);"/></td>
</tr>
</table>
</div>

## 🤝 贡献

欢迎提交 Pull Request 或创建 Issue！

## 📜 许可证

[MIT License](LICENSE)

