# 环境准备

在开始使用 Bangumi-rs 之前，你需要确保你的系统满足以下要求，并准备好相应的环境。

## 系统要求

Bangumi-rs 支持以下操作系统：

- **Linux**: 推荐 Ubuntu 20.04+ 或 Debian 11+
- **macOS**: 10.15 (Catalina) 及以上版本
- **Windows**: Windows 10 及以上版本

## 硬件要求

最低配置：

- CPU: 双核处理器
- 内存: 2GB RAM
- 存储: 取决于你计划下载的番剧数量，建议至少 10GB 可用空间

推荐配置：

- CPU: 四核处理器
- 内存: 4GB RAM 或更多
- 存储: 50GB+ 可用空间

## 依赖安装

### Docker 环境（推荐）

如果你计划使用 Docker 部署，请确保安装了以下软件：

1. **Docker**

   - 安装指南: [Docker 官方文档](https://docs.docker.com/get-docker/)

2. **Docker Compose**
   - 安装指南: [Docker Compose 官方文档](https://docs.docker.com/compose/install/)

### 本地环境

如果你计划在本地直接运行，需要安装以下依赖：

1. **Rust 环境** (如果从源码构建)

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js 环境** (如果需要修改前端)

   - 安装 Node.js 16+ 和 npm

3. **数据库**
   - SQLite 3.x (默认已包含)
   - 或 MySQL 5.7+ (可选)

## 网络要求

Bangumi-rs 需要访问以下网站，请确保你的网络环境能够正常连接：

- **Mikan**: https://mikanani.me
- **Bangumi.tv**: https://api.bgm.tv
- **TMDB**: https://api.themoviedb.org

如果你所在的网络环境无法直接访问这些网站，你可能需要配置代理。

## 播放器准备

如果你计划使用在线播放功能，请安装以下播放器之一：

- **IINA** (macOS): [下载地址](https://iina.io/)
- **Infuse** (iOS/macOS): [App Store](https://apps.apple.com/app/infuse-video-player/id1136220934)

## 下一步

完成环境准备后，你可以选择：

- [Docker 部署](/deployment/docker): 使用 Docker 快速部署
- [本地部署](/deployment/local): 在本地系统直接安装运行
- [基本使用](/quickstart/basic-usage): 了解基本使用方法
