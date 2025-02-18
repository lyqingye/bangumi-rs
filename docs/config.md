# 配置说明

项目使用 TOML 格式的配置文件,默认路径为 `config.toml`。

## 基础配置

```toml
[server]
listen_addr = "127.0.0.1:3001"  # 服务监听地址
database_url = "mysql://user:pass@localhost:3306/bangumi"  # 数据库连接 URL
assets_path = "assets"          # 资源文件路径

[log]
level = "debug"                 # 日志级别
```

## 代理配置

```toml
[proxy]
enabled = false
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

## 外部 API 配置

```toml
[tmdb]
api_key = "your_api_key"        # TMDB API 密钥
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p"
language = "zh-CN"

[bangumi_tv]
endpoint = "https://api.bgm.tv"
image_base_url = "https://lain.bgm.tv"
[mikan]
endpoint = "https://mikanani.me"
```

## 下载器配置

```toml
[pan115]
cookies = "your_cookies"         # 115 网盘 Cookie
download_dir = "/downloads"      # 下载目录
max_requests_per_second = 2      # 最大请求速率
```

## 通知配置

```toml
[notify.telegram]
enabled = true                   # 是否启用 Telegram 通知
token = "bot_token"             # Bot Token
chat_id = "chat_id"             # 聊天 ID
```

## 解析器配置

```toml
[parser.siliconflow]            # 使用 SiliconFlow API 解析文件名
enabled = true
api_key = "your_api_key"
base_url = "https://api.siliconflow.com"
model = "gpt-4"
```

