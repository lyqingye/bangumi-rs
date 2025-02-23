# Bangumi 下载管理器

<div align="center">
  <img src="docs/screenshot/home.png" width="80%" />
</div>

基于 Rust + Vue 3 开发的动漫追番,支持订阅番剧、自动下载, 在线播放等功能。

## 功能特性

- 🎯 番剧订阅

  - 支持订阅/取消订阅番剧
  - 可配置分辨率、字幕组、字幕语言过滤条件
  - 支持自定义订阅更新间隔

- 🔍 资源解析

  - 采用 ChatGPT 进行解析，支持多个 API 服务提供商 (SiliconFlow、OpenAI、Claude、DeepSeek)

- 📥 智能下载

  - 自动选择最佳种子 (根据分辨率以及语言字幕优先选择)
  - 支持用户手动选择要下载的剧集种子
  - 支持 115 网盘 离线下载

- 🎬 在线播放

  - 支持在线播放, 支持 IINA,Infuse 播放器

- 📚 元数据管理

  - 自动获取番剧信息
  - 支持从多个数据源获取(TMDB、Bangumi.tv、Mikan)
  - 支持手动刷新元数据
  - 支持剧集、海报墙、封面等信息显示

- 🔔 通知提醒
  - 支持 Telegram 通知

## 构建

[构建说明](docs/build.md)

## 配置文件

[配置说明](docs/config.md)

## 快速开始

```shell
mv config.example.toml config.toml
docker-compose up -d
```

前端运行在: http://localhost:80

## 许可证

MIT License

