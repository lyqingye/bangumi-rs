# 下载器配置

下载器配置部分控制 Bangumi-rs 的下载行为，包括下载策略、重试机制、存储路径、115 网盘和 qBittorrent 和 transmission 集成等。

## 配置概述

下载器配置位于配置文件的 `[downloader.pan115]` 和 `[downloader.qbittorrent]` 和 `[downloader.transmission]` 部分：

```toml
# 115网盘下载器配置
[downloader.pan115]
enabled = true
cookies = "Your 115 cookies"
max_requests_per_second = 1
download_dir = "/animes"
max_retry_count = 5
download_timeout = "30m"
retry_min_interval = "30s"
retry_max_interval = "10m"
delete_task_on_completion = true
priority = 0

# qBittorrent下载器配置
[downloader.qbittorrent]
enabled = false
url = "http://127.0.0.1:8080"
username = "admin"
password = "adminadmin"
download_dir = "/downloads"
mount_path = "/downloads"
max_retry_count = 5
download_timeout = "30m"
retry_min_interval = "30s"
retry_max_interval = "10m"
delete_task_on_completion = false
priority = 0

# transmission下载器配置
[downloader.transmission]
enabled = false
url = "http://localhost:9091/transmission/rpc"
username = "admin"
password = "123456"
download_dir = "/downloads/complete"
mount_path = "/downloads/complete"
max_requests_per_second = 1
max_retry_count = 1
retry_min_interval = "30s"
retry_max_interval = "10m"
download_timeout = "2h"
delete_task_on_completion = false
priority = 0
```

## 通用配置项说明

以下配置项在两个下载器中都有：

### 启用 (enabled)

- **说明**: 是否启用该下载器
- **默认值**: `false`
- **格式**: 布尔值
- **示例**: `enabled = true`

### 下载目录 (download_dir)

- **说明**: 下载文件的存储目录
- **格式**: 文件系统路径，必须是绝对路径
- **示例**:
  - 115 网盘: `download_dir = "/animes"` (相对于 115 网盘根目录)
  - qBittorrent: `download_dir = "/downloads"` (本地文件系统路径)

### 最大重试次数 (max_retry_count)

- **说明**: 下载失败后的最大重试次数
- **默认值**: `5`
- **格式**: 整数
- **示例**: `max_retry_count = 3`

::: tip 提示
设置为 `0` 表示不进行重试。对于不稳定的网络环境，建议设置较高的重试次数。
:::

### 最小重试间隔 (retry_min_interval)

- **说明**: 下载失败后的最小重试间隔时间
- **默认值**: `"30s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `retry_min_interval = "1m"`

### 最大重试间隔 (retry_max_interval)

- **说明**: 下载失败后的最大重试间隔时间
- **默认值**: `"10m"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `retry_max_interval = "30m"`

::: tip 提示
Bangumi-rs 使用指数退避算法计算重试间隔，从最小间隔开始，每次失败后增加，但不超过最大间隔。
:::

### 下载超时 (download_timeout)

- **说明**: 单个下载任务的超时时间
- **默认值**: `"30m"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `download_timeout = "1h"`

::: warning 注意
设置过短的超时时间可能导致大文件下载失败，设置过长则可能导致卡住的下载任务长时间占用资源。
:::

### 下载完成后是否删除任务 (delete_task_on_completion)

- **说明**: 下载完成后是否删除任务（不会删除文件）
- **默认值**: 115 网盘为 `true`，qBittorrent 为 `false`
- **格式**: 布尔值
- **示例**: `delete_task_on_completion = true`

### 优先级 (priority)

- **说明**: 下载任务的优先级，数字越大优先级越高
- **默认值**: `0`
- **格式**: 整数
- **示例**: `priority = 1`

## 115 网盘特有配置

### Cookies (cookies)

- **说明**: 115 网盘的登录 Cookies
- **格式**: 字符串
- **示例**: `cookies = "UID=xxx; CID=xxx; SEID=xxx; KID=xxx"`

::: warning 注意
获取文档可以参考: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F
:::

### 最大请求频率 (max_requests_per_second)

- **说明**: 每秒最大请求 115 API 的次数
- **默认值**: `1`
- **格式**: 整数
- **示例**: `max_requests_per_second = 1`

::: tip 提示
115 网盘 API 有请求频率限制，设置过高可能导致被临时封禁 1 小时。
:::

## qBittorrent 特有配置

### API 地址 (url)

- **说明**: qBittorrent WebUI 的地址
- **默认值**: `"http://127.0.0.1:8080"`
- **格式**: URL 字符串
- **示例**: `url = "http://127.0.0.1:8080"`

### 用户名 (username)

- **说明**: qBittorrent WebUI 的登录用户名
- **默认值**: `"admin"`
- **格式**: 字符串
- **示例**: `username = "admin"`

### 密码 (password)

- **说明**: qBittorrent WebUI 的登录密码
- **默认值**: `"adminadmin"`
- **格式**: 字符串
- **示例**: `password = "adminadmin"`

### 挂载路径 (mount_path)

- **说明**: 可选，如果你需要在线播放qb下载的文件，请设置此选项，该目录指向你本地的qbittorrent的下载目录(可以去qbittorrent的web界面查看下载目录), 程序需要访问目录用于在线播放
- **默认值**: `"/downloads"`
- **格式**: 字符串
- **示例**: `mount_path = "/downloads"`

## Transmission 特有配置

### API 地址 (url)

- **说明**: Transmission RPC API 的地址
- **默认值**: `"http://localhost:9091/transmission/rpc"`
- **格式**: URL 字符串
- **示例**: `url = "http://localhost:9091/transmission/rpc"`

### 用户名 (username)

- **说明**: Transmission RPC API 的登录用户名
- **默认值**: `"admin"`
- **格式**: 字符串
- **示例**: `username = "admin"`

### 密码 (password)

- **说明**: Transmission RPC API 的登录密码
- **默认值**: `"123456"`
- **格式**: 字符串
- **示例**: `password = "123456"`

### 挂载路径 (mount_path)

- **说明**: 可选，如果你需要在线播放qb下载的文件，请设置此选项，该目录指向你本地的transmission的下载目录(可以去transmission的web界面查看下载目录), 程序需要访问目录用于在线播放
- **默认值**: `"/downloads/complete"`
- **格式**: 字符串
- **示例**: `mount_path = "/downloads/complete"`


## Alist 特有配置

### 配置

```toml
[[downloader.alist]]
enabled = true
url = "http://127.0.0.1:5244"
username = "admin"
password = "123456"
tool = "115 Cloud"
download_dir = "/downloads"
max_retry_count = 1
retry_min_interval = "30s"
retry_max_interval = "10m"
download_timeout = "1h"
priority = 10
```

### Enabled

- **说明**: 是否启用 alist 下载器
- **默认值**: `false`
- **格式**: 布尔值
- **示例**: `enabled = true`

### URL

- **说明**: alist 的地址
- **默认值**: `"http://127.0.0.1:5244"`
- **格式**: URL 字符串
- **示例**: `url = "http://127.0.0.1:5244"`

### Username

- **说明**: alist 的用户名
- **默认值**: `"admin"`
- **格式**: 字符串
- **示例**: `username = "admin"`

### Password

- **说明**: alist 的密码
- **默认值**: `"123456"`
- **格式**: 字符串
- **示例**: `password = "123456"`

### Tool

- **说明**: 下载器类型，可选值为 `115 Cloud`、`qBittorrent`、`Transmission`、`PikPak`
- **默认值**: `"115 Cloud"`
- **格式**: 字符串
- **示例**: `tool = "115 Cloud"`








