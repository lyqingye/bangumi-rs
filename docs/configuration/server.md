# 服务器配置

服务器配置部分控制 Bangumi-rs 的基本运行参数，包括监听地址、数据库连接和资源路径等。

## 配置项

服务器配置位于配置文件的 `[server]` 部分：

```toml
# 服务器配置
[server]
listen_addr = "0.0.0.0:3001"
database_url = "mysql://user:pass@mysql:3306/bangumi"
# 该目录用来存放下载的番剧封面
assets_path = "/app/assets"

# 日志配置
[log]
level = "info" # debug, info, warn, error
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
- **默认值**: `"mysql://user:pass@mysql:3306/bangumi"`
- **格式**: 数据库连接 URL
- **支持的数据库**:
  - MySQL: `"mysql://用户名:密码@主机:端口/数据库名"`
  - PostgreSQL: `"postgres://用户名:密码@主机:端口/数据库名"`

**示例**:

```toml
# MySQL
database_url = "mysql://root:password@localhost:3306/bangumi"

# PostgreSQL
database_url = "postgres://postgres:password@localhost:5432/bangumi"
```

::: warning 注意
更改数据库连接需要重启应用程序才能生效。确保数据库服务器已经运行，并且用户有足够的权限。
:::