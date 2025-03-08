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