# 服务器配置

服务器配置部分控制 Bangumi-rs 的基本运行参数，包括监听地址、数据库连接和资源路径等。

## 配置项

服务器配置位于配置文件的 `[server]` 部分：

```toml
[server]
assets_path = "./assets"
listen_addr = "127.0.0.1:3001"
database_url = "sqlite:bangumi.db"
```

### 监听地址 (listen_addr)

- **说明**: 设置 Bangumi-rs Web 服务器的监听地址和端口
- **默认值**: `"127.0.0.1:3001"`
- **格式**: `"IP地址:端口号"`
- **示例**:
  - `"127.0.0.1:3001"`: 仅本地访问，端口 3001
  - `"0.0.0.0:80"`: 允许所有网络接口访问，端口 80

::: warning 注意
如果设置为 `0.0.0.0`，服务将对外网开放。在公网环境中，建议配置反向代理和身份验证。
:::

### 资源路径 (assets_path)

- **说明**: 设置静态资源文件的存储路径
- **默认值**: `"./assets"`
- **格式**: 文件系统路径
- **示例**:
  - `"./assets"`: 相对于应用程序的路径
  - `"/var/lib/bangumi/assets"`: 绝对路径

::: tip 提示
此路径用于存储应用程序的静态资源，如图片、CSS、JavaScript 文件等。确保应用程序对此路径有读写权限。
:::

### 数据库 URL (database_url)

- **说明**: 设置数据库连接字符串
- **默认值**: `"sqlite:bangumi.db"`
- **格式**: 数据库连接 URL
- **支持的数据库**:
  - SQLite: `"sqlite:文件名.db"`
  - MySQL: `"mysql://用户名:密码@主机:端口/数据库名"`
  - PostgreSQL: `"postgres://用户名:密码@主机:端口/数据库名"`

**示例**:

```toml
# SQLite (文件数据库)
database_url = "sqlite:bangumi.db"

# MySQL
database_url = "mysql://root:password@localhost:3306/bangumi"

# PostgreSQL
database_url = "postgres://postgres:password@localhost:5432/bangumi"
```

::: warning 注意
更改数据库连接需要重启应用程序才能生效。确保数据库服务器已经运行，并且用户有足够的权限。
:::

### 最大连接数 (max_connections)

- **说明**: 设置数据库连接池的最大连接数
- **默认值**: `10`
- **格式**: 整数
- **示例**: `max_connections = 20`

::: tip 提示
对于高负载系统，可以适当增加此值。但过多的连接可能会导致数据库服务器压力过大。
:::

### 会话超时 (session_timeout)

- **说明**: 设置用户会话的超时时间
- **默认值**: `"24h"`
- **格式**: 时间字符串，支持 `s`(秒)、`m`(分)、`h`(小时)、`d`(天)
- **示例**: `session_timeout = "48h"`

## 高级配置

### CORS 设置 (cors)

- **说明**: 配置跨域资源共享 (CORS) 设置
- **默认值**: 仅允许同源请求
- **示例**:

```toml
[server.cors]
allowed_origins = ["https://example.com", "https://sub.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allow_credentials = true
```

### TLS 配置 (tls)

- **说明**: 配置 HTTPS 支持
- **默认值**: 禁用 TLS
- **示例**:

```toml
[server.tls]
enabled = true
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
```

::: tip 提示
在生产环境中，建议使用 HTTPS 保护数据传输安全。你也可以使用 Nginx 或 Caddy 等反向代理服务器处理 TLS 终结。
:::

## 环境变量

你可以使用环境变量覆盖配置文件中的服务器设置：

- `BANGUMI_SERVER_LISTEN_ADDR`: 监听地址
- `BANGUMI_SERVER_ASSETS_PATH`: 资源路径
- `BANGUMI_SERVER_DATABASE_URL`: 数据库 URL
- `BANGUMI_SERVER_MAX_CONNECTIONS`: 最大连接数
- `BANGUMI_SERVER_SESSION_TIMEOUT`: 会话超时

## 最佳实践

1. **生产环境配置**:

   - 使用反向代理 (如 Nginx) 处理 TLS 和访问控制
   - 将监听地址设置为 `127.0.0.1` 并通过反向代理访问
   - 使用专用数据库用户，并限制其权限

2. **开发环境配置**:

   - 使用 SQLite 简化配置
   - 设置详细的日志级别便于调试

3. **Docker 环境配置**:
   - 监听地址设置为 `0.0.0.0` 以允许容器外访问
   - 使用环境变量注入敏感信息
   - 使用数据卷持久化数据库和资源

## 配置示例

### 基本配置

```toml
[server]
listen_addr = "127.0.0.1:3001"
assets_path = "./assets"
database_url = "sqlite:bangumi.db"
```

### 生产环境配置

```toml
[server]
listen_addr = "127.0.0.1:3001"
assets_path = "/var/lib/bangumi/assets"
database_url = "mysql://bangumi_user:password@db.example.com:3306/bangumi_prod"
max_connections = 30
session_timeout = "12h"
```

### Docker 环境配置

```toml
[server]
listen_addr = "0.0.0.0:3001"
assets_path = "/app/assets"
database_url = "sqlite:/app/data/bangumi.db"
```
