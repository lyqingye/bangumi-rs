# 环境准备

在开始使用 Bangumi-rs 之前，你需要确保你的系统满足以下要求，并准备好相应的环境。

## 系统要求

Bangumi-rs 支持以下操作系统：

- **Linux**
- **macOS**

### Docker 环境（推荐）

如果你计划使用 Docker 部署，请确保安装了以下软件：

1. **Docker**

   - 安装指南: [Docker 官方文档](https://docs.docker.com/get-docker/)

2. **Docker Compose**
   - 安装指南: [Docker Compose 官方文档](https://docs.docker.com/compose/install/)

## 网络要求

Bangumi-rs 需要访问以下网站，请确保你的网络环境能够正常连接：

- **Mikan**: https://mikanani.me
- **Bangumi**: https://bgm.tv
- **TMDB**: https://themoviedb.org
  > TMDB 需要申请对应的 APIKey, 请你注册并登陆后跳转到 https://www.themoviedb.org/settings/api 生成对应的 APIkey

- **115网盘**
> 你需要参考该文档生成对应的115网盘Cookies: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F

- **Telegram 通知（可选）**
> 如果你需要推送消息到Telegram，那么你就需要创建自己的Telegram Bot，获得Token以及ChatId


> [!WARNING] 注意 : 如果你所在的网络环境无法直接访问这些网站，你可能需要配置代理。意味着你可能需要 VPN 等之类的工具
> 或者使用 Cloudflare 对站点进行反向代理

## 播放器准备

如果你计划使用在线播放功能，请安装以下播放器之一：

- **IINA** (macOS): [下载地址](https://iina.io/)
- **Infuse** (iOS/macOS): [App Store](https://apps.apple.com/app/infuse-video-player/id1136220934)

## 下一步

完成环境准备后，你可以选择：

- [Docker 部署](/deploy/docker): 使用 Docker 快速部署
- [基本使用](/quickstart/basic-usage): 了解基本使用方法
