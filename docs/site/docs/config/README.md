# 配置说明

Bangumi-rs 使用 TOML 格式的配置文件，默认配置文件路径为 `config.toml`。本节将详细介绍各个配置项的含义和用法。

## 配置文件结构

配置文件主要包含以下几个部分：

- [服务器配置](./server.md)
- [代理配置](./proxy.md)
- [API 配置](./api.md)
- [下载器配置](./downloader.md)
- [通知配置](./notify.md)
- [解析器配置](./parser.md)

## 配置示例

```toml
[server]
listen_addr = "127.0.0.1:3001"  # 服务监听地址
database_url = "mysql://user:pass@localhost:3306/bangumi"  # 数据库连接 URL
assets_path = "assets"          # 资源文件路径

[log]
level = "debug"                 # 日志级别

[proxy]
enabled = false
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"

# 更多配置项请参考各个配置章节
```

## 配置优先级

1. 命令行参数
2. 环境变量
3. 配置文件
4. 默认值

## 环境变量

所有配置项都可以通过环境变量进行覆盖，环境变量名称规则为：将配置项路径中的 `.` 替换为 `_`，并转换为大写。例如：

- `server.listen_addr` -> `SERVER_LISTEN_ADDR`
- `proxy.http` -> `PROXY_HTTP`

## 配置验证

启动时，系统会自动验证配置文件的正确性。如果配置有误，会输出详细的错误信息。

