# 代理配置

代理配置部分控制 Bangumi-rs 如何通过代理服务器访问网络资源，特别是在某些网络环境下无法直接访问资源站点的情况。

## 配置概述

代理配置位于配置文件的 `[proxy]` 部分：

```toml
[proxy]
enabled = false
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

## 基本配置项

### 启用状态 (enabled)

- **说明**: 是否启用代理
- **默认值**: `false`
- **格式**: 布尔值 (`true` 或 `false`)
- **示例**: `enabled = true`

::: tip 提示
只有在设置为 `true` 时，代理设置才会生效。这允许你在配置文件中保留代理设置，但只在需要时启用。
:::

### HTTP 代理 (http)

- **说明**: HTTP 协议的代理服务器地址
- **格式**: URL 字符串
- **示例**: `http = "http://127.0.0.1:7890"`

::: tip 提示
HTTP 代理用于访问 HTTP 协议的网站和 API。
:::

### HTTPS 代理 (https)

- **说明**: HTTPS 协议的代理服务器地址
- **格式**: URL 字符串
- **示例**: `https = "http://127.0.0.1:7890"`

::: tip 提示
HTTPS 代理用于访问 HTTPS 协议的网站和 API。通常与 HTTP 代理设置为相同的地址。
:::

### 不使用代理的地址 (no_proxy)

- **说明**: 不使用代理的地址列表
- **格式**: 字符串数组
- **示例**: `no_proxy = ["localhost", "127.0.0.1", ".local"]`

::: tip 提示
对于这些地址的请求将直接发送，不通过代理服务器。这对于访问本地服务或内部网络很有用。
:::

## 高级配置

### 代理认证 (auth)

代理认证配置位于 `[proxy.auth]` 部分：

```toml
[proxy.auth]
username = "user"
password = "pass"
```

- **用户名 (username)**

  - **说明**: 代理服务器的认证用户名
  - **格式**: 字符串
  - **示例**: `username = "proxyuser"`

- **密码 (password)**
  - **说明**: 代理服务器的认证密码
  - **格式**: 字符串
  - **示例**: `password = "proxypass"`

::: warning 注意
代理认证信息是敏感数据，不要将其提交到版本控制系统。建议使用环境变量注入。
:::

### SOCKS5 代理 (socks5)

SOCKS5 代理配置位于 `[proxy.socks5]` 部分：

```toml
[proxy.socks5]
enabled = false
address = "127.0.0.1:1080"
username = "user"
password = "pass"
```

- **启用状态 (enabled)**

  - **说明**: 是否启用 SOCKS5 代理
  - **默认值**: `false`
  - **格式**: 布尔值 (`true` 或 `false`)

- **地址 (address)**

  - **说明**: SOCKS5 代理服务器地址
  - **格式**: `"主机:端口"`
  - **示例**: `address = "127.0.0.1:1080"`

- **用户名 (username)**

  - **说明**: SOCKS5 代理服务器的认证用户名
  - **格式**: 字符串
  - **示例**: `username = "socksuser"`

- **密码 (password)**
  - **说明**: SOCKS5 代理服务器的认证密码
  - **格式**: 字符串
  - **示例**: `password = "sockspass"`

## 代理选择策略

代理选择策略配置位于 `[proxy.strategy]` 部分：

```toml
[proxy.strategy]
mode = "all"
fallback = true
```

- **模式 (mode)**

  - **说明**: 代理使用模式
  - **默认值**: `"all"`
  - **可选值**:
    - `"all"`: 所有请求都使用代理
    - `"selective"`: 仅特定站点使用代理
  - **示例**: `mode = "selective"`

- **失败回退 (fallback)**
  - **说明**: 代理失败时是否尝试直接连接
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)
  - **示例**: `fallback = true`

### 选择性代理配置

当代理模式设置为 `"selective"` 时，可以配置哪些站点使用代理：

```toml
[proxy.sites]
mikan = true
bangumi_tv = false
tmdb = false
```

- **Mikan (mikan)**

  - **说明**: 是否对 Mikan 站点使用代理
  - **默认值**: `true`
  - **格式**: 布尔值 (`true` 或 `false`)

- **Bangumi.tv (bangumi_tv)**

  - **说明**: 是否对 Bangumi.tv 站点使用代理
  - **默认值**: `false`
  - **格式**: 布尔值 (`true` 或 `false`)

- **TMDB (tmdb)**
  - **说明**: 是否对 TMDB 站点使用代理
  - **默认值**: `false`
  - **格式**: 布尔值 (`true` 或 `false`)

## 环境变量

你可以使用环境变量覆盖配置文件中的代理设置：

- **基本配置**:

  - `BANGUMI_PROXY_ENABLED`: 是否启用代理
  - `BANGUMI_PROXY_HTTP`: HTTP 代理地址
  - `BANGUMI_PROXY_HTTPS`: HTTPS 代理地址
  - `BANGUMI_PROXY_NO_PROXY`: 不使用代理的地址列表，用逗号分隔

- **认证配置**:

  - `BANGUMI_PROXY_AUTH_USERNAME`: 代理认证用户名
  - `BANGUMI_PROXY_AUTH_PASSWORD`: 代理认证密码

- **SOCKS5 配置**:
  - `BANGUMI_PROXY_SOCKS5_ENABLED`: 是否启用 SOCKS5 代理
  - `BANGUMI_PROXY_SOCKS5_ADDRESS`: SOCKS5 代理地址
  - `BANGUMI_PROXY_SOCKS5_USERNAME`: SOCKS5 认证用户名
  - `BANGUMI_PROXY_SOCKS5_PASSWORD`: SOCKS5 认证密码

## 常见代理类型

### HTTP 代理

HTTP 代理是最常见的代理类型，支持 HTTP 和 HTTPS 协议：

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

### SOCKS5 代理

SOCKS5 代理提供更通用的代理功能，支持多种协议：

```toml
[proxy]
enabled = true

[proxy.socks5]
enabled = true
address = "127.0.0.1:1080"
```

### Clash 代理

[Clash](https://github.com/Dreamacro/clash) 是一个流行的代理工具，默认配置如下：

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

### V2Ray 代理

[V2Ray](https://github.com/v2fly/v2ray-core) 是另一个流行的代理工具，默认配置如下：

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:10809"
https = "http://127.0.0.1:10809"
```

## 最佳实践

1. **安全性**:

   - 使用环境变量存储代理认证信息
   - 避免在公共网络上使用不安全的代理
   - 定期更新代理服务器和客户端

2. **性能优化**:

   - 选择地理位置接近资源站点的代理服务器
   - 使用选择性代理模式，只对需要的站点启用代理
   - 配置 `no_proxy` 避免对本地资源使用代理

3. **可靠性**:
   - 启用失败回退功能，确保在代理不可用时仍能访问资源
   - 定期测试代理连接
   - 准备备用代理服务器

## 配置示例

### 基本 HTTP 代理

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

### 带认证的代理

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"

[proxy.auth]
username = "${PROXY_USERNAME}"
password = "${PROXY_PASSWORD}"
```

### 选择性代理

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"

[proxy.strategy]
mode = "selective"
fallback = true

[proxy.sites]
mikan = true
bangumi_tv = false
tmdb = false
```

### SOCKS5 代理

```toml
[proxy]
enabled = true

[proxy.socks5]
enabled = true
address = "127.0.0.1:1080"
username = "${SOCKS_USERNAME}"
password = "${SOCKS_PASSWORD}"
```
