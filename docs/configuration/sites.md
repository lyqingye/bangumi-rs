# 资源站点配置

资源站点配置部分控制 Bangumi-rs 如何连接和使用各种资源站点和元数据源，包括 Mikan、Bangumi.tv 和 TMDB 等。

## 配置概述

资源站点配置分布在配置文件的多个部分，每个站点有自己的配置部分：

```toml
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