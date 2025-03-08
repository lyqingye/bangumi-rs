# 资源站点配置

资源站点配置部分控制 Bangumi-rs 如何连接和使用各种资源站点和元数据源，包括 Mikan、Bangumi.tv 和 TMDB 等。

## 配置概述

资源站点配置分布在配置文件的多个部分，每个站点有自己的配置部分：

```toml
[mikan]
endpoint = "https://mikanani.me"

[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"

[tmdb]
api_key = "your-api-key"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "zh-CN"
```

## Mikan 配置

[Mikan Project](https://mikanani.me/) 是一个动漫资源站点，提供番剧的种子下载。

Mikan 配置位于 `[mikan]` 部分：

```toml
[mikan]
endpoint = "https://mikanani.me"
```

### 端点 (endpoint)

- **说明**: Mikan 网站的基础 URL
- **默认值**: `"https://mikanani.me"`
- **格式**: URL 字符串
- **示例**: `endpoint = "https://mikanani.me"`

::: tip 提示
如果 Mikan 的官方地址发生变化，可以通过此设置更新。
:::

### 缓存时间 (cache_ttl)

- **说明**: Mikan 数据的缓存时间
- **默认值**: `"1h"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)、`d`(天)
- **示例**: `cache_ttl = "30m"`

::: tip 提示
适当增加缓存时间可以减少对 Mikan 服务器的请求，但可能导致获取到的信息不是最新的。
:::

### 请求超时 (timeout)

- **说明**: 请求 Mikan 的超时时间
- **默认值**: `"30s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `timeout = "10s"`

### 请求间隔 (request_interval)

- **说明**: 连续请求 Mikan 的最小间隔时间
- **默认值**: `"1s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `request_interval = "2s"`

::: warning 注意
设置过短的请求间隔可能导致被 Mikan 服务器限制访问。
:::

## Bangumi.tv 配置

[Bangumi.tv](https://bgm.tv/) 是一个专注于动漫、游戏的中文社区和数据库，提供丰富的番剧元数据。

Bangumi.tv 配置位于 `[bangumi_tv]` 部分：

```toml
[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"
```

### 端点 (endpoint)

- **说明**: Bangumi.tv API 的基础 URL
- **默认值**: `"https://api.bgm.tv"`
- **格式**: URL 字符串
- **示例**: `endpoint = "https://api.bgm.tv"`

### 图片基础 URL (image_base_url)

- **说明**: Bangumi.tv 图片的基础 URL
- **默认值**: `"https://lain.bgm.tv"`
- **格式**: URL 字符串
- **示例**: `image_base_url = "https://lain.bgm.tv"`

### 缓存时间 (cache_ttl)

- **说明**: Bangumi.tv 数据的缓存时间
- **默认值**: `"24h"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)、`d`(天)
- **示例**: `cache_ttl = "12h"`

### 请求超时 (timeout)

- **说明**: 请求 Bangumi.tv 的超时时间
- **默认值**: `"30s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `timeout = "10s"`

### 请求间隔 (request_interval)

- **说明**: 连续请求 Bangumi.tv 的最小间隔时间
- **默认值**: `"1s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `request_interval = "2s"`

::: warning 注意
Bangumi.tv API 有请求频率限制，设置过短的请求间隔可能导致被限制访问。
:::

## TMDB 配置

[TMDB (The Movie Database)](https://www.themoviedb.org/) 是一个全球性的影视数据库，提供丰富的影视信息。

TMDB 配置位于 `[tmdb]` 部分：

```toml
[tmdb]
api_key = "your-api-key"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "zh-CN"
```

### API 密钥 (api_key)

- **说明**: TMDB API 的密钥
- **格式**: 字符串
- **示例**: `api_key = "1234567890abcdef1234567890abcdef"`

::: tip 提示
你需要在 [TMDB 官网](https://www.themoviedb.org/settings/api) 注册并获取 API 密钥。
:::

::: warning 注意
API 密钥是敏感信息，不要将其提交到版本控制系统。建议使用环境变量注入。
:::

### 基础 URL (base_url)

- **说明**: TMDB API 的基础 URL
- **默认值**: `"https://api.themoviedb.org/3"`
- **格式**: URL 字符串
- **示例**: `base_url = "https://api.themoviedb.org/3"`

### 图片基础 URL (image_base_url)

- **说明**: TMDB 图片的基础 URL
- **默认值**: `"https://image.tmdb.org/t/p/original"`
- **格式**: URL 字符串
- **示例**: `image_base_url = "https://image.tmdb.org/t/p/original"`

### 语言 (language)

- **说明**: 请求 TMDB 数据的首选语言
- **默认值**: `"zh-CN"`
- **格式**: 语言代码
- **可选值**: `"zh-CN"` (简体中文), `"zh-TW"` (繁体中文), `"ja-JP"` (日语), `"en-US"` (英语) 等
- **示例**: `language = "zh-CN"`

### 缓存时间 (cache_ttl)

- **说明**: TMDB 数据的缓存时间
- **默认值**: `"24h"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)、`d`(天)
- **示例**: `cache_ttl = "12h"`

### 请求超时 (timeout)

- **说明**: 请求 TMDB 的超时时间
- **默认值**: `"30s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `timeout = "10s"`

### 请求间隔 (request_interval)

- **说明**: 连续请求 TMDB 的最小间隔时间
- **默认值**: `"0.25s"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `request_interval = "0.5s"`

::: warning 注意
TMDB API 有请求频率限制，设置过短的请求间隔可能导致被限制访问。
:::

## 站点优先级配置

站点优先级配置位于 `[sites.priority]` 部分，控制在获取元数据时各站点的优先级：

```toml
[sites.priority]
metadata = ["tmdb", "bangumi_tv", "mikan"]
images = ["tmdb", "bangumi_tv"]
```

### 元数据优先级 (metadata)

- **说明**: 获取元数据时的站点优先级顺序
- **默认值**: `["tmdb", "bangumi_tv", "mikan"]`
- **格式**: 字符串数组
- **示例**: `metadata = ["bangumi_tv", "tmdb", "mikan"]`

::: tip 提示
列表中靠前的站点优先级更高。系统会先尝试从高优先级站点获取数据，如果失败或数据不完整，再尝试低优先级站点。
:::

### 图片优先级 (images)

- **说明**: 获取图片时的站点优先级顺序
- **默认值**: `["tmdb", "bangumi_tv"]`
- **格式**: 字符串数组
- **示例**: `images = ["tmdb", "bangumi_tv"]`

## 环境变量

你可以使用环境变量覆盖配置文件中的站点设置：

- **Mikan 配置**:

  - `BANGUMI_MIKAN_ENDPOINT`: Mikan 端点
  - `BANGUMI_MIKAN_CACHE_TTL`: 缓存时间
  - `BANGUMI_MIKAN_TIMEOUT`: 请求超时
  - `BANGUMI_MIKAN_REQUEST_INTERVAL`: 请求间隔

- **Bangumi.tv 配置**:

  - `BANGUMI_BANGUMI_TV_ENDPOINT`: Bangumi.tv API 端点
  - `BANGUMI_BANGUMI_TV_IMAGE_BASE_URL`: 图片基础 URL
  - `BANGUMI_BANGUMI_TV_CACHE_TTL`: 缓存时间
  - `BANGUMI_BANGUMI_TV_TIMEOUT`: 请求超时
  - `BANGUMI_BANGUMI_TV_REQUEST_INTERVAL`: 请求间隔

- **TMDB 配置**:
  - `BANGUMI_TMDB_API_KEY`: API 密钥
  - `BANGUMI_TMDB_BASE_URL`: 基础 URL
  - `BANGUMI_TMDB_IMAGE_BASE_URL`: 图片基础 URL
  - `BANGUMI_TMDB_LANGUAGE`: 语言
  - `BANGUMI_TMDB_CACHE_TTL`: 缓存时间
  - `BANGUMI_TMDB_TIMEOUT`: 请求超时
  - `BANGUMI_TMDB_REQUEST_INTERVAL`: 请求间隔

## 最佳实践

1. **API 密钥管理**:

   - 使用环境变量存储 API 密钥
   - 定期轮换 API 密钥，提高安全性
   - 不要在公共代码库中提交 API 密钥

2. **缓存优化**:

   - 根据数据更新频率设置合适的缓存时间
   - 对于不经常变化的数据（如番剧基本信息），可以设置较长的缓存时间
   - 对于经常变化的数据（如最新剧集），设置较短的缓存时间

3. **请求频率控制**:

   - 遵守各站点的 API 使用政策
   - 设置合理的请求间隔，避免被限制访问
   - 实现请求重试机制，处理临时性错误

4. **元数据质量**:
   - 根据需求调整站点优先级
   - 对于中文用户，可以优先使用 Bangumi.tv 获取中文元数据
   - 对于图片质量要求高的场景，优先使用 TMDB

## 配置示例

### 基本配置

```toml
[mikan]
endpoint = "https://mikanani.me"

[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"

[tmdb]
api_key = "${TMDB_API_KEY}"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "zh-CN"
```

### 优化的缓存配置

```toml
[mikan]
endpoint = "https://mikanani.me"
cache_ttl = "30m"
request_interval = "2s"

[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"
cache_ttl = "12h"
request_interval = "1s"

[tmdb]
api_key = "${TMDB_API_KEY}"
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "zh-CN"
cache_ttl = "24h"
request_interval = "0.5s"
```

### 自定义优先级配置

```toml
[sites.priority]
metadata = ["bangumi_tv", "tmdb", "mikan"]
images = ["tmdb", "bangumi_tv"]
```
