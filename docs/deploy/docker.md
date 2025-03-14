# Docker 部署指南

Docker 是部署 Bangumi-rs 最简单、最推荐的方式。本指南将详细介绍如何使用 Docker 和 Docker Compose 部署 Bangumi-rs。

## 前提条件

在开始之前，请确保你的系统已安装以下软件：

- Docker 20.10.0 或更高版本
- Docker Compose v2.0.0 或更高版本（可选，但推荐）

### 安装 Docker

如果你尚未安装 Docker，可以参考以下指南：

- [Docker 官方安装指南](https://docs.docker.com/get-docker/)

#### Linux 快速安装

```bash
curl -fsSL https://get.docker.com | sh
sudo systemctl enable --now docker
```

#### 验证 Docker 安装

```bash
docker --version
```

### 安装 Docker Compose

Docker Compose 可以简化多容器应用的部署和管理。

#### Linux 安装 Docker Compose

```bash
sudo curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

#### 验证 Docker Compose 安装

```bash
docker-compose --version
```

## **创建配置文件以及缓存目录**

```bash
# 配置文件
touch config.toml
touch nginx.conf
# 缓存目录
mkdir assets
# 数据库目录
mkdir data
```

**添加以下配置 (config.toml):**

```toml
# 服务器配置
[server]
listen_addr = "0.0.0.0:3001"
database_url = "mysql://user:pass@mysql:3306/bangumi"
# 该目录用来存放下载的番剧封面
assets_path = "/app/assets"

# 日志配置
[log]
level = "info" # debug, info, warn, error

# 代理配置 (如果你有梯子的话，可以填写该选项)
[proxy]
enabled = false
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"

# TMDB API 配置
[tmdb]
# 这里需要填写你的TMDB APIkey
api_key = "your_tmdb_api_key"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p"
language = "zh-CN"

# Bangumi.tv API 配置
[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"

# Mikan 配置
[mikan]
endpoint = "https://mikanani.me"

# 下载器配置
[downloader]
# 这里的路径相当于你115网盘根目录下的animes文件夹
download_dir = "/animes"
# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"

# 115网盘下载器配置
[downloader.pan115]
# 获取文档可以参考: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F
cookies = "Your 115 cookies"
# 限流，写1也足够了，请求速率过快的话，会被封禁1小时
max_requests_per_second = 1

# Telegram 通知配置 (可选)
[notify.telegram]
enabled = false
token = "your_bot_token"
chat_id = "your_chat_id"

# 文件名解析器配置
# 原生解析器
[parser.raw]
enabled = true

```
**前端服务Nginx配置文件 (nginx.conf):**
```conf
server {
    listen 80;
    server_name localhost;

    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://backend:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /ws {
        proxy_pass http://backend:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

## 创建 docker-compose.yml 配置文件

```yaml
version: "3.8"

services:
  mysql:
    restart: unless-stopped
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: 123456
      MYSQL_DATABASE: bangumi
    ports:
      - "3306:3306"
    volumes:
      - ./data:/var/lib/mysql
      - ./develop/schema.sql:/docker-entrypoint-initdb.d/schema.sql
    healthcheck:
      test:
        [
          "CMD",
          "mysqladmin",
          "ping",
          "-h",
          "localhost",
          "-u",
          "root",
          "-p123456",
        ]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - bangumi-network

  backend:
    restart: unless-stopped
    image: ghcr.io/lyqingye/bangumi-rs/backend:latest
    ports:
      - "3001:3001"
    volumes:
      - ./assets:/app/assets
      - ./config.toml:/app/config.toml
      - ./animes:/animes
    command: ["/app/bangumi", "start"]
    depends_on:
      mysql:
        condition: service_healthy
    networks:
      - bangumi-network
    labels:
      - "com.centurylinklabs.watchtower.enable=true"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s

  frontend:
    restart: unless-stopped
    image: ghcr.io/lyqingye/bangumi-rs/frontend:latest
    ports:
      - "80:80"
    depends_on:
      - backend
    networks:
      - bangumi-network
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
    labels:
      - "com.centurylinklabs.watchtower.enable=true"

networks:
  bangumi-network:
    driver: bridge
```

## 启动服务

```bash
docker-compose up -d
```
## 查看服务日志
```bash
docker-compose logs -f --tail 100 backend
```
当你看到类似的启动日志时，说明程序配置完全正确:
```bash
2025-03-08T15:31:41.921860Z  INFO sea_orm_migration::migrator: Applying all pending migrations
2025-03-08T15:31:41.944244Z  INFO model::migrator: loading migration file: "V1.0.0__init.sql"
2025-03-08T15:31:41.944790Z  INFO model::migrator: loading migration file: "V1.0.1__bgm_kind.sql"
2025-03-08T15:31:41.944846Z  INFO model::migrator: loading migration file: "V1.0.2__task_interval.sql"
2025-03-08T15:31:41.944927Z  INFO model::migrator: loading migration file: "V1.0.3__sub_option.sql"
2025-03-08T15:31:41.944970Z  INFO model::migrator: loading migration file: "V1.0.4__remove_tmdb_uk.sql"
2025-03-08T15:31:41.954735Z  INFO sea_orm_migration::migrator: No pending migrations
2025-03-08T15:31:42.315786Z  INFO downloader::syncer: 启动远程任务同步器
2025-03-08T15:31:42.315889Z  INFO downloader::worker: 开始恢复未处理的下载任务
2025-03-08T15:31:42.320265Z  INFO downloader::worker: 找到 0 个未处理的任务
2025-03-08T15:31:42.320299Z  INFO downloader::worker: 完成恢复未处理的下载任务
2025-03-08T15:31:42.320330Z  INFO downloader::worker: Downloader 已启动，配置: Config { sync_interval: 10s, event_queue_size: 128, max_retry_count: 5, retry_processor_interval: 5s, retry_min_interval: TimeDelta { secs: 30, nanos: 0 }, retry_max_interval: TimeDelta { secs: 600, nanos: 0 }, download_dir: "/animes", download_timeout: TimeDelta { secs: 1800, nanos: 0 } }
2025-03-08T15:31:42.324020Z  INFO scheduler::scheduler: 启动下载调度器
2025-03-08T15:31:42.339482Z  INFO server::server: server listen at: http://127.0.0.1:3001
2025-03-08T15:31:42.340099Z  INFO actix_server::builder: starting 14 workers
2025-03-08T15:31:42.340283Z  INFO actix_server::server: Tokio runtime found; starting in existing Tokio runtime
2025-03-08T15:31:42.340520Z  INFO actix_server::server: starting service: "actix-web-service-127.0.0.1:3001", workers: 14, listening on: 127.0.0.1:3001
```

## 自动更新

推荐直接使用 [watchtower](https://github.com/containrrr/watchtower) 来实现镜像自动更新, 在你的 `docker-compose.yaml` 配置文件中增加一个服务:
```yaml
  watchtower:
    restart: unless-stopped
    image: containrrr/watchtower:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: --interval 60 --cleanup --label-enable
    networks:
      - bangumi-network
```
完整的`docker-compose.yml`可以参考:
https://github.com/lyqingye/bangumi-rs/blob/master/docker-compose.yml