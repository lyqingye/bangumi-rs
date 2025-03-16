# 下载器配置

下载器配置部分控制 Bangumi-rs 的下载行为，包括下载策略、重试机制、存储路径和 115 网盘集成等。

## 配置概述

下载器配置位于配置文件的 `[downloader]` 部分及其子部分：

```toml
# 下载器配置
[downloader]

# 下载最大重试次数
max_retry_count = 5
# 下载超时，避免由于死种导致一直在下载
download_timeout = "30m"
# 重试的最小时间间隔，将逐级递增
retry_min_interval = "30s"
retry_max_interval = "10m"

```

## 基本配置项

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

### 下载目录 (download_dir)

- **说明**: 下载文件的存储目录, Unixlike 文件系统路径，如果你使用的是 115 网盘作为下载器，并且你填写的配置是 `/downloads` 那么则代表下载的文件会放在根目录的 `downloads` 文件夹中
- **默认值**: `"/downloads"`
- **格式**: 文件系统路径, 必须是绝对路径
- **示例**:
  - `download_dir = "/animes"`

::: tip 提示
确保有足够的存储空间。
:::

## 115 网盘配置

115 网盘配置位于 `[downloader.pan115]` 部分，用于配置 115 网盘离线下载功能：

```toml
# 115网盘下载器配置
[downloader.pan115]
enabled = true
cookies = "Your 115 cookies"
max_requests_per_second = 1
# 这里的路径相当于你115网盘根目录下的animes文件夹
download_dir = "/animes"
```

### 启用 (enabled)

- **说明**: 是否启用 115 网盘下载器
- **默认值**: `false`
- **格式**: 布尔值
- **示例**: `enabled = true`

### Cookies (cookies)

- **说明**: 115 网盘的登录 Cookies
- **格式**: 字符串
- **示例**: `cookies = "UID=xxx; CID=xxx; SEID=xxx; KID=xxx"`

::: warning 注意
获取文档可以参考: https://alist.nn.ci/zh/guide/drivers/115.html#cookie%E8%8E%B7%E5%8F%96%E6%96%B9%E5%BC%8F
:::

### 下载目录 (download_dir)

- **说明**: 115 网盘下载目录,这里代表你 115 网盘根目录下的 animes 文件夹
- **默认值**: `"/animes"`
- **格式**: 字符串
- **示例**: `download_dir = "/animes"`

### 下载完成后是否删除任务 (delete_task_on_completion)

- **说明**: 下载完成后是否删除任务, 不会删除文件，只会删除任务
- **默认值**: `true`
- **格式**: 布尔值
- **示例**: `delete_task_on_completion = true`

### 最大请求频率 (max_requests_per_second)

- **说明**: 每秒最大请求 115 API 的次数
- **默认值**: `1`
- **格式**: 整数
- **示例**: `max_requests_per_second = 2`

::: tip 提示
115 网盘 API 有请求频率限制，设置过高可能导致被临时封禁 1 小时。
:::

## qbittorrent 配置

qbittorrent 配置位于 `[downloader.qbittorrent]` 部分，用于配置 qbittorrent 下载功能：

```toml
# qbittorrent 下载器配置
[downloader.qbittorrent]
enabled = true
download_dir = "/downloads"
username = "admin"
password = "adminadmin"
url = "http://127.0.0.1:8080"
```

### 启用 (enabled)

- **说明**: 是否启用 qbittorrent 下载器
- **默认值**: `false`
- **格式**: 布尔值
- **示例**: `enabled = true`

### 下载目录 (download_dir)

- **说明**: qbittorrent 下载目录
- **默认值**: `"/downloads"`
- **格式**: 字符串
- **示例**: `download_dir = "/downloads"`

### 用户名 (username)

- **说明**: qbittorrent 用户名
- **默认值**: `"admin"`
- **格式**: 字符串
- **示例**: `username = "admin"`

### 密码 (password)

- **说明**: qbittorrent 密码
- **默认值**: `"adminadmin"`
- **格式**: 字符串
- **示例**: `password = "adminadmin"`

### API 地址 (url)

- **说明**: qbittorrent API 地址
- **默认值**: `"http://127.0.0.1:8080"`
- **格式**: 字符串
- **示例**: `url = "http://127.0.0.1:8080"`

### 下载完成后是否删除任务 (delete_task_on_completion)

- **说明**: 下载完成后是否删除任务, 不会删除文件，只会删除任务
- **默认值**: `false`
- **格式**: 布尔值
- **示例**: `delete_task_on_completion = true`
