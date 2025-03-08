# 解析器配置

解析器配置部分控制 Bangumi-rs 如何解析动漫文件名，包括传统解析器和 AI 解析器的设置。

## 配置概述

Bangumi-rs 支持两种类型的解析器：

1. **传统解析器 (Raw Parser)**: 基于正则表达式和规则的文件名解析
2. **AI 解析器**: 利用大型语言模型进行更准确的解析，支持多个 API 服务提供商

解析器配置位于配置文件的 `[parser]` 部分及其子部分：

```toml
[parser.raw]
enabled = true

[parser.siliconflow]
enabled = false
api_key = "your-api-key"
model = "deepseek-ai/DeepSeek-V3"
base_url = "https://api.siliconflow.com/v1"
```

## 传统解析器配置

传统解析器基于预定义的规则和正则表达式，能够处理大多数标准命名的文件。

### 配置项

传统解析器配置位于 `[parser.raw]` 部分：

```toml
[parser.raw]
enabled = true
priority = 1
```

- **启用状态 (enabled)**

  - **说明**: 是否启用传统解析器
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

- **优先级 (priority)**

  - **说明**: 设置传统解析器的优先级，数字越小优先级越高
  - **默认值**: `1`
  - **格式**: 整数

- **自定义规则 (custom_rules)**
  - **说明**: 添加自定义的正则表达式规则
  - **格式**: 字符串数组
  - **示例**:
    ```toml
    custom_rules = [
      "(?P<title>.+?)\\s+S(?P<season>\\d+)E(?P<episode>\\d+)",
      "\\[(?P<subgroup>.+?)\\]\\s*(?P<title>.+?)\\s+(?P<episode>\\d+)"
    ]
    ```

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

- **优先级 (priority)**

  - **说明**: 设置该 AI 解析器的优先级，数字越小优先级越高
  - **默认值**: 根据解析器类型不同而不同
  - **格式**: 整数

- **超时 (timeout)**
  - **说明**: API 请求的超时时间
  - **默认值**: `"30s"`
  - **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
  - **示例**: `timeout = "60s"`

### SiliconFlow 解析器

SiliconFlow 解析器配置位于 `[parser.siliconflow]` 部分：

```toml
[parser.siliconflow]
enabled = false
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "deepseek-ai/DeepSeek-V3"
base_url = "https://api.siliconflow.com/v1"
priority = 2
timeout = "30s"
```

### DeepSeek 解析器

DeepSeek 解析器配置位于 `[parser.deepseek]` 部分：

```toml
[parser.deepseek]
enabled = false
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1"
priority = 3
timeout = "30s"
```

### OpenAI 解析器

OpenAI 解析器配置位于 `[parser.openai]` 部分：

```toml
[parser.openai]
enabled = false
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "gpt-4"
base_url = "https://api.openai.com/v1"
priority = 4
timeout = "30s"
```

### Claude 解析器

Claude 解析器配置位于 `[parser.claude]` 部分：

```toml
[parser.claude]
enabled = false
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "claude-3-opus-20240229"
base_url = "https://api.anthropic.com/v1"
priority = 5
timeout = "30s"
```

## 解析策略配置

解析策略控制多个解析器之间的协作方式，配置位于 `[parser.strategy]` 部分：

```toml
[parser.strategy]
mode = "priority"
cache_ttl = "24h"
fallback = true
```

- **模式 (mode)**

  - **说明**: 解析器选择模式
  - **默认值**: `"priority"`
  - **可选值**:
    - `"priority"`: 按优先级顺序使用解析器
    - `"all"`: 使用所有启用的解析器并合并结果
    - `"vote"`: 使用所有启用的解析器并选择最多数的结果

- **缓存时间 (cache_ttl)**

  - **说明**: 解析结果的缓存时间
  - **默认值**: `"24h"`
  - **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)、`d`(天)

- **失败回退 (fallback)**
  - **说明**: 当高优先级解析器失败时是否尝试低优先级解析器
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

## 环境变量

你可以使用环境变量覆盖配置文件中的解析器设置：

- **传统解析器**:

  - `BANGUMI_PARSER_RAW_ENABLED`: 是否启用传统解析器
  - `BANGUMI_PARSER_RAW_PRIORITY`: 传统解析器优先级

- **SiliconFlow 解析器**:

  - `BANGUMI_PARSER_SILICONFLOW_ENABLED`: 是否启用 SiliconFlow 解析器
  - `BANGUMI_PARSER_SILICONFLOW_API_KEY`: API 密钥
  - `BANGUMI_PARSER_SILICONFLOW_MODEL`: 模型名称
  - `BANGUMI_PARSER_SILICONFLOW_BASE_URL`: 基础 URL
  - `BANGUMI_PARSER_SILICONFLOW_PRIORITY`: 优先级
  - `BANGUMI_PARSER_SILICONFLOW_TIMEOUT`: 超时时间

- **其他解析器**:
  - 类似的环境变量格式适用于其他解析器

## 最佳实践

1. **解析器选择**:

   - 对于标准命名的文件，启用传统解析器即可
   - 对于复杂或非标准命名的文件，启用 AI 解析器提高准确率
   - 如果需要最高的准确率，可以启用多个解析器并使用 `vote` 模式

2. **API 密钥管理**:

   - 不要在配置文件中直接存储 API 密钥，使用环境变量注入
   - 定期轮换 API 密钥，提高安全性

3. **性能优化**:
   - 适当增加缓存时间，减少 API 请求
   - 设置合理的超时时间，避免请求卡住
   - 优先使用传统解析器，仅在必要时使用 AI 解析器

## 配置示例

### 基本配置 (仅传统解析器)

```toml
[parser.raw]
enabled = true
priority = 1

[parser.strategy]
mode = "priority"
cache_ttl = "24h"
fallback = true
```

### 高准确率配置 (多解析器)

```toml
[parser.raw]
enabled = true
priority = 1

[parser.siliconflow]
enabled = true
api_key = "${SILICONFLOW_API_KEY}"
model = "deepseek-ai/DeepSeek-V3"
base_url = "https://api.siliconflow.com/v1"
priority = 2

[parser.openai]
enabled = true
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
base_url = "https://api.openai.com/v1"
priority = 3

[parser.strategy]
mode = "vote"
cache_ttl = "48h"
fallback = true
```

### 低资源消耗配置

```toml
[parser.raw]
enabled = true
priority = 1

[parser.siliconflow]
enabled = true
api_key = "${SILICONFLOW_API_KEY}"
model = "deepseek-ai/DeepSeek-V3"
base_url = "https://api.siliconflow.com/v1"
priority = 2
timeout = "10s"

[parser.strategy]
mode = "priority"
cache_ttl = "72h"
fallback = true
```
