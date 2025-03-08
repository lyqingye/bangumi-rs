# 通知配置

通知配置部分控制 Bangumi-rs 的通知系统，包括通知渠道、事件类型和通知格式等。

## 配置概述

通知配置位于配置文件的 `[notify]` 部分及其子部分：

```toml
[notify.telegram]
enabled = true
token = "your-telegram-bot-token"
chat_id = "your-chat-id"
```

目前，Bangumi-rs 主要支持 Telegram 作为通知渠道，未来将支持更多渠道。

## Telegram 通知配置

Telegram 通知配置位于 `[notify.telegram]` 部分：

```toml
[notify.telegram]
enabled = true
token = "1234567890:ABCDEFGHIJKLMNOPQRSTUVWXYZ"
chat_id = "123456789"
template = "📺 {event_type}\n番剧: {title}\n剧集: {episode}\n状态: {status}\n时间: {time}"
```

### 启用状态 (enabled)

- **说明**: 是否启用 Telegram 通知
- **默认值**: `false`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `enabled = true`

### Bot Token (token)

- **说明**: Telegram Bot 的 API Token
- **格式**: 字符串
- **示例**: `token = "1234567890:ABCDEFGHIJKLMNOPQRSTUVWXYZ"`

::: tip 提示
你需要通过 [@BotFather](https://t.me/BotFather) 创建一个 Telegram Bot 并获取 Token。
:::

::: warning 注意
Bot Token 是敏感信息，不要将其提交到版本控制系统。建议使用环境变量注入。
:::

### 聊天 ID (chat_id)

- **说明**: 接收通知的聊天 ID
- **格式**: 字符串
- **示例**: `chat_id = "123456789"`

::: tip 提示
可以是个人聊天 ID、群组 ID 或频道 ID。你可以通过 [@userinfobot](https://t.me/userinfobot) 获取你的个人 ID。
:::

### 通知模板 (template)

- **说明**: 通知消息的模板
- **默认值**: `"📺 {event_type}\n番剧: {title}\n剧集: {episode}\n状态: {status}\n时间: {time}"`
- **格式**: 字符串，支持以下变量:
  - `{event_type}`: 事件类型
  - `{title}`: 番剧标题
  - `{episode}`: 剧集信息
  - `{status}`: 状态信息
  - `{time}`: 事件时间
  - `{details}`: 详细信息
- **示例**: `template = "🔔 {title} - {episode} {status}"`

### 静默时间 (quiet_hours)

- **说明**: 不发送通知的时间段
- **格式**: 字符串数组，每个元素格式为 `"HH:MM-HH:MM"`
- **示例**: `quiet_hours = ["23:00-07:00"]`

::: tip 提示
在静默时间内，通知会被缓存，并在静默时间结束后发送。
:::

### 通知频率限制 (rate_limit)

- **说明**: 通知发送频率限制
- **默认值**: `"0s"` (不限制)
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
- **示例**: `rate_limit = "1m"`

::: tip 提示
设置后，相同类型的通知在指定时间内只会发送一次，避免通知过多。
:::

## 事件过滤配置

事件过滤配置位于 `[notify.events]` 部分，控制哪些事件会触发通知：

```toml
[notify.events]
new_episode = true
download_start = true
download_complete = true
download_fail = true
system_error = true
```

### 新剧集事件 (new_episode)

- **说明**: 是否通知新剧集发布
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `new_episode = true`

### 下载开始事件 (download_start)

- **说明**: 是否通知下载开始
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `download_start = true`

### 下载完成事件 (download_complete)

- **说明**: 是否通知下载完成
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `download_complete = true`

### 下载失败事件 (download_fail)

- **说明**: 是否通知下载失败
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `download_fail = true`

### 系统错误事件 (system_error)

- **说明**: 是否通知系统错误
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `system_error = true`

### 系统启动事件 (system_start)

- **说明**: 是否通知系统启动
- **默认值**: `false`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `system_start = true`

### 系统更新事件 (system_update)

- **说明**: 是否通知系统更新
- **默认值**: `true`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `system_update = true`

## 高级配置

### 通知历史 (history)

通知历史配置位于 `[notify.history]` 部分：

```toml
[notify.history]
enabled = true
max_entries = 100
```

- **启用状态 (enabled)**

  - **说明**: 是否记录通知历史
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

- **最大条目数 (max_entries)**
  - **说明**: 保存的最大历史记录数
  - **默认值**: `100`
  - **格式**: 整数
  - **示例**: `max_entries = 200`

### 通知分组 (grouping)

通知分组配置位于 `[notify.grouping]` 部分：

```toml
[notify.grouping]
enabled = true
max_group_size = 5
group_timeout = "5m"
```

- **启用状态 (enabled)**

  - **说明**: 是否启用通知分组
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

- **最大分组大小 (max_group_size)**

  - **说明**: 单个分组中的最大通知数
  - **默认值**: `5`
  - **格式**: 整数
  - **示例**: `max_group_size = 10`

- **分组超时 (group_timeout)**
  - **说明**: 分组的最大等待时间
  - **默认值**: `"5m"`
  - **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)
  - **示例**: `group_timeout = "10m"`

## 环境变量

你可以使用环境变量覆盖配置文件中的通知设置：

- **Telegram 配置**:

  - `BANGUMI_NOTIFY_TELEGRAM_ENABLED`: 是否启用 Telegram 通知
  - `BANGUMI_NOTIFY_TELEGRAM_TOKEN`: Bot Token
  - `BANGUMI_NOTIFY_TELEGRAM_CHAT_ID`: 聊天 ID
  - `BANGUMI_NOTIFY_TELEGRAM_TEMPLATE`: 通知模板

- **事件过滤**:
  - `BANGUMI_NOTIFY_EVENTS_NEW_EPISODE`: 是否通知新剧集
  - `BANGUMI_NOTIFY_EVENTS_DOWNLOAD_START`: 是否通知下载开始
  - `BANGUMI_NOTIFY_EVENTS_DOWNLOAD_COMPLETE`: 是否通知下载完成
  - `BANGUMI_NOTIFY_EVENTS_DOWNLOAD_FAIL`: 是否通知下载失败
  - `BANGUMI_NOTIFY_EVENTS_SYSTEM_ERROR`: 是否通知系统错误

## 最佳实践

1. **通知管理**:

   - 只启用真正需要的通知类型，避免通知过多
   - 使用静默时间避免夜间打扰
   - 对于大量下载任务，考虑使用通知分组

2. **安全性**:

   - 使用环境变量存储 Bot Token 等敏感信息
   - 避免在公共群组中使用，以防泄露下载内容
   - 定期检查 Bot 的安全性

3. **自定义**:
   - 根据个人偏好调整通知模板
   - 为不同类型的事件设置不同的通知格式
   - 考虑使用 Telegram 的格式化功能（如 Markdown 或 HTML）

## 配置示例

### 基本配置

```toml
[notify.telegram]
enabled = true
token = "1234567890:ABCDEFGHIJKLMNOPQRSTUVWXYZ"
chat_id = "123456789"

[notify.events]
new_episode = true
download_start = false
download_complete = true
download_fail = true
system_error = true
```

### 详细配置

```toml
[notify.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
chat_id = "${TELEGRAM_CHAT_ID}"
template = "📺 *{event_type}*\n🎬 *{title}*\n📝 {episode}\n⏱️ {time}\n\n{details}"
quiet_hours = ["23:00-07:00"]
rate_limit = "1m"

[notify.events]
new_episode = true
download_start = true
download_complete = true
download_fail = true
system_error = true
system_start = true
system_update = true

[notify.history]
enabled = true
max_entries = 200

[notify.grouping]
enabled = true
max_group_size = 10
group_timeout = "3m"
```

### 最小配置

```toml
[notify.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
chat_id = "${TELEGRAM_CHAT_ID}"

[notify.events]
new_episode = true
download_complete = true
download_fail = true
```
