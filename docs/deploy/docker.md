# Docker 部署指南

Docker 是部署 Bangumi-rs 最简单、最推荐的方式。本指南将详细介绍如何使用 Docker 和 Docker Compose 部署 Bangumi-rs。

## **创建配置文件以及缓存目录**

```bash
# 配置文件
touch config.toml
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
database_url = "mysql://root:123456@mysql:3306/bangumi"
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
no_proxy = [
    "localhost",
    "127.0.0.1",
    "qbittorrent",
    "transmission",
]

# TMDB API 配置
[tmdb]
# 这里需要填写你的TMDB APIkey
api_key = "your_tmdb_api_key"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
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


# 115网盘下载器配置 (至少启用一个下载器)
[downloader.pan115]
enabled = false
# 获取文档可以参考: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F
cookies = "Your 115 cookies"
# 限流，写1也足够了，请求速率过快的话，会被封禁1小时
max_requests_per_second = 1
# 115网盘下载目录
download_dir = "/animes"
# 下载完成后是否删除任务, 不会删除文件，只会删除任务
delete_task_on_completion = true
# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"

# qbittorrent 下载器配置 (至少启用一个下载器)
[downloader.qbittorrent]
enabled = false
# qbittorrent 下载目录
download_dir = "/downloads"
# 可选，如果你需要在线播放qb下载的文件，请设置此选项，该目录指向qbittorrent的下载目录
mount_path = "/qb-downloads"
# qbittorrent 用户名
username = "admin"
# qbittorrent 密码
password = "adminadmin"
# qbittorrent API 地址
url = "http://127.0.0.1:8080"
# 下载完成后是否删除任务, 不会删除文件，只会删除任务
delete_task_on_completion = false
# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"

[downloader.transmission]
enabled = false
url = "http://localhost:9091/transmission/rpc"
username = "admin"
password = "123456"
download_dir = "/downloads/complete"
# 可选，如果你需要在线播放下载的文件，请设置此选项，该目录指向transmission的下载目录
mount_path = "/ts-downloads/complete"
max_requests_per_second = 1
max_retry_count = 1
retry_min_interval = "30s"
retry_max_interval = "10m"
download_timeout = "2h"
delete_task_on_completion = false
priority = 0

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

## 下载 Nginx 配置文件

```bash
curl -o nginx.conf https://raw.githubusercontent.com/lyqingye/bangumi-rs/refs/heads/master/nginx.conf
```

## 填写 docker-compose.yml 配置文件

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
      # 可选，如果你需要使用qbittorrent下载器，请设置此选项, 用于在线播放
      - ./qb-downloads:/qb-downloads
      # 可选，如果你需要使用transmission下载器，请设置此选项, 用于在线播放
      - ./ts-downloads:/ts-downloads
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

## 部署 Qbittorrent (可选)

```yaml
qbittorrent:
  image: lscr.io/linuxserver/qbittorrent:latest
  container_name: qbittorrent
  environment:
    - PUID=1000 # 更改为你的用户 ID
    - PGID=1000 # 更改为你的组 ID
    - TZ=Asia/Shanghai # 更改为你所在的时区
    - WEBUI_PORT=8080
  volumes:
    - ./qb-config:/config # 更改为你的配置目录
    - ./qb-downloads:/downloads # 更改为你的下载目录
  ports:
    - 6881:6881
    - 6881:6881/udp
    - 8080:8080 # Web UI 端口，可以根据需要更改
  restart: unless-stopped
```

## 部署 Transmission (可选)

```yaml
transmission:
  image: lscr.io/linuxserver/transmission:latest
  container_name: transmission
  environment:
    - PUID=1000
    - PGID=1000
    - TZ=Etc/UTC
    - USER=admin
    - PASS=123456
  volumes:
    - ./ts-data:/config
    - ./ts-downloader:/downloads
    - ./ts-watch:/watch #optional
  ports:
    # WEB UI 端口
    - 9091:9091
    - 51413:51413
    - 51413:51413/udp
  restart: unless-stopped
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

本教程相关文件:

- [docker-compose.yml](https://github.com/lyqingye/bangumi-rs/blob/master/docker-compose.yml)
- [nginx.conf](https://github.com/lyqingye/bangumi-rs/blob/master/nginx.conf)
- [schema.sql](https://github.com/lyqingye/bangumi-rs/blob/master/develop/schema.sql)

