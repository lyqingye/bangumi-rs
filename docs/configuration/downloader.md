# 下载器配置

下载器配置部分控制 Bangumi-rs 的下载行为，包括下载策略、重试机制、存储路径和 115 网盘集成等。

## 配置概述

下载器配置位于配置文件的 `[downloader]` 部分及其子部分：

```toml
[downloader]
max_retry_count = 5
retry_min_interval = "30s"
retry_max_interval = "10m"
download_timeout = "30m"
download_dir = "/animes"

[downloader.pan115]
cookies = "your-115-cookies"
max_requests_per_second = 1
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

- **说明**: 下载文件的存储目录
- **默认值**: `"./downloads"`
- **格式**: 文件系统路径
- **示例**:
  - `download_dir = "/animes"`
  - `download_dir = "D:\\Animes"` (Windows)

::: tip 提示
确保应用程序对此目录有读写权限，并且有足够的存储空间。
:::

### 最大并行任务数 (max_concurrent_tasks)

- **说明**: 最大同时下载的任务数
- **默认值**: `3`
- **格式**: 整数
- **示例**: `max_concurrent_tasks = 5`

::: tip 提示
根据你的网络带宽和系统性能调整此值。设置过高可能导致网络拥塞或系统负载过高。
:::

### 下载速度限制 (speed_limit)

- **说明**: 全局下载速度限制
- **默认值**: 不限制
- **格式**: 字符串，支持 `K`(KB/s)、`M`(MB/s)、`G`(GB/s)
- **示例**:
  - `speed_limit = "5M"` (限制为 5MB/s)
  - `speed_limit = "500K"` (限制为 500KB/s)

## 115 网盘配置

115 网盘配置位于 `[downloader.pan115]` 部分，用于配置 115 网盘离线下载功能：

```toml
[downloader.pan115]
cookies = "UID=xxx; CID=xxx; SEID=xxx; KID=xxx"
max_requests_per_second = 1
```

### Cookies (cookies)

- **说明**: 115 网盘的登录 Cookies
- **格式**: 字符串
- **示例**: `cookies = "UID=xxx; CID=xxx; SEID=xxx; KID=xxx"`

::: warning 注意
Cookies 包含敏感信息，不要将其提交到版本控制系统。建议使用环境变量注入。
:::

### 最大请求频率 (max_requests_per_second)

- **说明**: 每秒最大请求 115 API 的次数
- **默认值**: `1`
- **格式**: 整数
- **示例**: `max_requests_per_second = 2`

::: tip 提示
115 网盘 API 有请求频率限制，设置过高可能导致被临时封禁。
:::

### 下载目录 (download_dir)

- **说明**: 115 网盘中的下载目录
- **默认值**: 根目录
- **格式**: 字符串
- **示例**: `download_dir = "动漫/Bangumi-rs"`

### 自动转存 (auto_transfer)

- **说明**: 是否自动将离线下载完成的文件转存到指定目录
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `auto_transfer = true`

## 下载策略配置

下载策略配置位于 `[downloader.strategy]` 部分，控制如何选择和管理下载任务：

```toml
[downloader.strategy]
preferred_resolution = "1080p"
preferred_subgroups = ["ANi", "喵萌奶茶屋", "LoliHouse"]
preferred_languages = ["CHS", "CHT"]
score_weights = { resolution = 0.4, subgroup = 0.3, language = 0.3 }
```

### 首选分辨率 (preferred_resolution)

- **说明**: 首选的视频分辨率
- **默认值**: `"1080p"`
- **格式**: 字符串
- **可选值**: `"480p"`, `"720p"`, `"1080p"`, `"2160p"` (4K)
- **示例**: `preferred_resolution = "1080p"`

### 首选字幕组 (preferred_subgroups)

- **说明**: 首选的字幕组，按优先级排序
- **格式**: 字符串数组
- **示例**: `preferred_subgroups = ["ANi", "喵萌奶茶屋", "LoliHouse"]`

### 首选字幕语言 (preferred_languages)

- **说明**: 首选的字幕语言，按优先级排序
- **格式**: 字符串数组
- **可选值**: `"CHS"` (简体中文), `"CHT"` (繁体中文), `"JPN"` (日语), `"ENG"` (英语)
- **示例**: `preferred_languages = ["CHS", "CHT"]`

### 评分权重 (score_weights)

- **说明**: 各因素在种子评分中的权重
- **格式**: 表格
- **示例**:
  ```toml
  score_weights = { resolution = 0.4, subgroup = 0.3, language = 0.3 }
  ```

## 文件组织配置

文件组织配置位于 `[downloader.organization]` 部分，控制下载文件的命名和目录结构：

```toml
[downloader.organization]
naming_template = "{title}/Season {season}/{title} - S{season}E{episode} [{resolution}]"
create_season_folders = true
group_by_title = true
```

### 命名模板 (naming_template)

- **说明**: 下载文件的命名模板
- **默认值**: `"{title}/Season {season}/{title} - S{season}E{episode} [{resolution}]"`
- **格式**: 字符串，支持以下变量:
  - `{title}`: 番剧标题
  - `{season}`: 季度编号
  - `{episode}`: 剧集编号
  - `{resolution}`: 分辨率
  - `{subgroup}`: 字幕组
  - `{language}`: 字幕语言
- **示例**: `naming_template = "{title}/{title} - {episode} [{subgroup}]"`

### 创建季度文件夹 (create_season_folders)

- **说明**: 是否为不同季度创建单独的文件夹
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `create_season_folders = true`

### 按标题分组 (group_by_title)

- **说明**: 是否按番剧标题分组创建文件夹
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `group_by_title = true`

## 环境变量

你可以使用环境变量覆盖配置文件中的下载器设置：

- **基本配置**:

  - `BANGUMI_DOWNLOADER_MAX_RETRY_COUNT`: 最大重试次数
  - `BANGUMI_DOWNLOADER_RETRY_MIN_INTERVAL`: 最小重试间隔
  - `BANGUMI_DOWNLOADER_RETRY_MAX_INTERVAL`: 最大重试间隔
  - `BANGUMI_DOWNLOADER_DOWNLOAD_TIMEOUT`: 下载超时
  - `BANGUMI_DOWNLOADER_DOWNLOAD_DIR`: 下载目录
  - `BANGUMI_DOWNLOADER_MAX_CONCURRENT_TASKS`: 最大并行任务数
  - `BANGUMI_DOWNLOADER_SPEED_LIMIT`: 下载速度限制

- **115 网盘配置**:
  - `BANGUMI_DOWNLOADER_PAN115_COOKIES`: 115 网盘 Cookies
  - `BANGUMI_DOWNLOADER_PAN115_MAX_REQUESTS_PER_SECOND`: 最大请求频率

## 最佳实践

1. **存储管理**:

   - 选择有足够空间的存储位置
   - 定期清理不需要的下载内容
   - 考虑使用外部存储设备或 NAS 存储大量内容

2. **网络优化**:

   - 根据网络带宽调整并行任务数和速度限制
   - 对于不稳定的网络，增加重试次数和超时时间
   - 考虑使用代理或 VPN 提高下载速度和稳定性

3. **115 网盘使用**:
   - 定期更新 Cookies 避免过期
   - 注意 115 网盘的存储空间限制
   - 使用自动转存功能保持离线空间整洁

## 配置示例

### 基本配置

```toml
[downloader]
max_retry_count = 5
retry_min_interval = "30s"
retry_max_interval = "10m"
download_timeout = "30m"
download_dir = "/animes"
max_concurrent_tasks = 3
```

### 高速下载配置

```toml
[downloader]
max_retry_count = 3
retry_min_interval = "10s"
retry_max_interval = "1m"
download_timeout = "1h"
download_dir = "/animes"
max_concurrent_tasks = 10
speed_limit = "50M"

[downloader.strategy]
preferred_resolution = "1080p"
preferred_subgroups = ["ANi", "LoliHouse"]
preferred_languages = ["CHS"]
```

### 115 网盘配置

```toml
[downloader]
max_retry_count = 3
download_dir = "/animes"

[downloader.pan115]
cookies = "${PAN115_COOKIES}"
max_requests_per_second = 1
download_dir = "动漫/Bangumi-rs"
auto_transfer = true
```
