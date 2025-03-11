# 解析器配置

解析器配置部分控制 Bangumi-rs 如何解析动漫文件名，包括传统解析器和 AI 解析器的设置。

## 配置概述

Bangumi-rs 支持两种类型的解析器：

1. **传统解析器 (Raw Parser)**: 基于正则表达式和规则的文件名解析
2. **AI 解析器**: 利用大型语言模型进行更准确的解析，支持多个 API 服务提供商

解析器配置位于配置文件的 `[parser]` 部分及其子部分：

```toml
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

## 传统解析器配置

传统解析器基于预定义的规则和正则表达式，能够处理大多数标准命名的文件。

### 配置项

传统解析器配置位于 `[parser.raw]` 部分：

```toml
[parser.raw]
enabled = true
```

- **启用状态 (enabled)**

  - **说明**: 是否启用传统解析器
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

## AI 解析器配置

Bangumi-rs 支持多种 AI 服务提供商的解析器，每个提供商有自己的配置部分。

### 通用配置项

每个 AI 解析器都有以下通用配置项：

- **启用状态 (enabled)**

  - **说明**: 是否启用该 AI 解析器
  - **默认值**: `false`
  - **格式**: 布尔值 (`true` 或 `false`)

- **API 密钥 (api_key)**

  - **说明**: 服务提供商的 API 密钥
  - **格式**: 字符串
  - **示例**: `api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"`

- **模型 (model)**

  - **说明**: 要使用的 AI 模型名称
  - **格式**: 字符串
  - **示例**: `model = "gpt-4"`

- **基础 URL (base_url)**

  - **说明**: API 服务的基础 URL
  - **格式**: 字符串
  - **示例**: `base_url = "https://api.openai.com/v1"`