# 配置概览

Bangumi-rs 通过配置文件提供了丰富的自定义选项，让你可以根据自己的需求调整系统行为。本页面将概述配置文件的结构和主要配置项。

## 配置文件位置

默认情况下，Bangumi-rs 会在以下位置查找配置文件：

- **标准位置**: `./config.toml`（与可执行文件同目录）
- **Docker 环境**: `/app/config.toml`（容器内路径）

你也可以通过填写后端服务启动参数来指定配置文件的路径：

```bash
# docker-compose 中的配置
command: ["/app/bangumi", "--config", "/app/config/config.toml", "start"]
```

## 配置示例

以下是一个完整的配置文件示例：

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
# 115网盘下载器配置
[downloader.pan115]
enabled = true
# 获取文档可以参考: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F
cookies = "Your 115 cookies"
# 限流，写1也足够了，请求速率过快的话，会被封禁1小时
max_requests_per_second = 1
# 这里的路径相当于你115网盘根目录下的animes文件夹
download_dir = "/animes"
# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"
# 下载完成后是否删除任务（不会删除文件）
delete_task_on_completion = true
# 下载优先级
priority = 0

# qBittorrent下载器配置
[downloader.qbittorrent]
enabled = false
url = "http://127.0.0.1:8080"
username = "admin"
password = "adminadmin"
download_dir = "/downloads"
# 可选，如果你需要在线播放qb下载的文件，请设置此选项，该目录指向你本地的qbittorrent的下载目录, 程序需要访问目录用于在线播放
mount_path = "/downloads"
# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"
# 下载完成后是否删除任务（不会删除文件）
delete_task_on_completion = false
# 下载优先级
priority = 0

[downloader.transmission]
enabled = false
url = "http://localhost:9091/transmission/rpc"
username = "admin"
password = "123456"
download_dir = "/downloads/complete"
# 可选，如果你需要在线播放qb下载的文件，请设置此选项，该目录指向qbittorrent的下载目录
mount_path = "/Users/lyqingye/Desktop/docker/ts-downloader/complete"
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

# 基于AI的解析器三选一即可: siliconflow, deepseek, deepbricks

[parser.siliconflow]
enabled = false
api_key = "your_api_key"
base_url = "https://api.siliconflow.com"
model = "gpt-4"

[parser.deepseek]
enabled = false
api_key = "your_api_key"
base_url = "https://api.deepseek.com"
model = "deepseek-chat"

[parser.deepbricks]
enabled = false
api_key = "your_api_key"
base_url = "https://api.deepbricks.com"
model = "gpt-4"

```

## 下一步

查看以下页面了解各个配置部分的详细说明：

- [服务配置](/configuration/server): 服务器和数据库设置
- [站点配置](/configuration/sites): 资源站点和元数据源配置
- [通知配置](/configuration/notification): 通知渠道和参数
- [代理配置](/configuration/proxy): 网络代理设置
- [解析器配置](/configuration/parser): 文件名解析器配置
- [下载器配置](/configuration/downloader): 下载行为和存储配置

