# 配置概览

Bangumi-rs 通过配置文件提供了丰富的自定义选项，让你可以根据自己的需求调整系统行为。本页面将概述配置文件的结构和主要配置项。

## 配置文件位置

默认情况下，Bangumi-rs 会在以下位置查找配置文件：

- **标准位置**: `./config.toml`（与可执行文件同目录）
- **Docker 环境**: `/app/config.toml`（容器内路径）

你也可以通过环境变量 `BANGUMI_CONFIG` 指定配置文件的路径：

```bash
export BANGUMI_CONFIG=/path/to/your/config.toml
```

## 配置文件格式

Bangumi-rs 使用 [TOML](https://toml.io/) 格式作为配置文件格式。TOML 是一种易于阅读和编写的配置文件格式，类似于 INI 但功能更强大。

配置文件由多个部分组成，每个部分对应系统的一个功能模块：

```toml
[section_name]
key1 = "value1"
key2 = 123

[section_name.subsection]
key3 = true
```

## 配置示例

以下是一个基本的配置文件示例：

```toml
[log]
level = "info"

[server]
assets_path = "./assets"
listen_addr = "127.0.0.1:3001"
database_url = "sqlite:bangumi.db"

[mikan]
endpoint = "https://mikanani.me"

[downloader]
max_retry_count = 5
download_dir = "/animes"

[proxy]
enabled = false
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

## 主要配置部分

Bangumi-rs 的配置文件包含以下主要部分：

### 日志配置 [log]

控制日志输出级别和格式。

### 服务器配置 [server]

设置服务器监听地址、数据库连接和资源路径。

### 资源站点配置 [mikan], [bangumi_tv], [tmdb]

配置各个资源站点和元数据源的 API 端点和认证信息。

### 解析器配置 [parser.*]

配置文件名解析器，包括传统解析器和 AI 解析器。

### 下载器配置 [downloader]

设置下载行为、重试策略和存储路径。

### 通知配置 [notify.*]

配置通知渠道和参数。

### 代理配置 [proxy]

设置网络代理，用于访问被限制的资源。

## 配置热重载

Bangumi-rs 支持配置热重载，这意味着你可以在不重启应用程序的情况下修改配置文件。系统会自动检测配置文件的变化并应用新的设置。

某些配置项（如数据库连接、监听地址等）需要重启应用程序才能生效。

## 环境变量覆盖

你可以使用环境变量覆盖配置文件中的设置。环境变量的命名规则为：

```
BANGUMI_SECTION_KEY=value
```

例如，要覆盖 `[server]` 部分的 `listen_addr` 设置，可以使用：

```bash
export BANGUMI_SERVER_LISTEN_ADDR="0.0.0.0:3001"
```

## 敏感信息处理

配置文件可能包含敏感信息，如 API 密钥和认证令牌。建议：

1. 不要将包含敏感信息的配置文件提交到版本控制系统
2. 使用环境变量或密钥管理系统管理敏感信息
3. 限制配置文件的访问权限

## 下一步

查看以下页面了解各个配置部分的详细说明：

- [服务器配置](/configuration/server): 服务器和数据库设置
- [资源站点配置](/configuration/sites): 资源站点和元数据源配置
- [解析器配置](/configuration/parser): 文件名解析器配置
- [下载器配置](/configuration/downloader): 下载行为和存储配置
- [通知配置](/configuration/notification): 通知渠道和参数
- [代理配置](/configuration/proxy): 网络代理设置
