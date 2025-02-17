# Bangumi 下载管理器

<div align="center">
  <img src="docs/screenshot/home.png" width="80%" />
</div>

基于 Rust + Vue 3 开发的动漫追番,支持订阅番剧、自动下载, 在线播放等功能。

## 功能特性

- 🎯 番剧订阅

  - 支持订阅/取消订阅番剧
  - 可配置分辨率、字幕组、语言等过滤条件
  - 支持自定义订阅更新间隔

- 🔍 资源解析

  - 采用 ChatGPT 进行解析，支持多个 API 服务提供商 (SiliconFlow、OpenAI、Claude、DeepSeek)

- 📥 自动下载

  - 自动选择最佳种子 (根据分辨率以及语言字幕优先选择)
  - 支持 115 网盘 离线下载

- 🎬 在线播放

  - 支持在线播放, 唤起 IINA 播放器

- 📚 元数据管理

  - 自动获取番剧信息
  - 支持从多个数据源获取(TMDB、Bangumi.tv、Mikan)
  - 支持手动刷新元数据
  - 支持剧集、海报等信息管理

- 🔔 通知提醒
  - 支持 Telegram 通知

## 技术栈

### 后端

- Rust
- tokio (异步运行时)
- sea-orm (ORM)
- actix-web (Web 框架)
- MySQL (数据库)

### 前端

- Vue 3
- TypeScript
- Vuetify 3
- Vite

## 构建说明

### 环境要求

- Rust 1.75+
- Node.js 18+
- MySQL 8.0+

### 后端构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/your-username/bangumi.git
cd bangumi

# 创建配置文件
cp config.example.toml config.toml
# 修改配置文件中的数据库连接等信息

# 使用 Makefile 命令进行构建和运行
make build              # 开发模式构建
make build-release      # 发布模式构建
make run               # 运行开发版本
make run-dev           # 运行开发版本(支持 tokio console)

# 代码维护命令
make fmt               # 格式化代码
make clippy           # 运行 clippy 检查
make test             # 运行测试
make check            # 检查代码(不编译)
make doc              # 生成文档
make clean            # 清理构建文件

# 数据库相关
make gen-entity       # 生成 SeaORM entity (需要配置 .env 文件)

# 运行所有检查
make all              # 运行 fmt, clippy, test, build
```

### 前端构建

```bash
# 进入前端目录
cd web

# 安装依赖
npm install

# 构建
npm run build
```

## 配置说明

项目使用 TOML 格式的配置文件,默认路径为 `config.toml`。

### 基础配置

```toml
[server]
listen_addr = "127.0.0.1:3001"  # 服务监听地址
database_url = "mysql://user:pass@localhost:3306/bangumi"  # 数据库连接 URL
assets_path = "assets"          # 资源文件路径

[log]
level = "debug"                 # 日志级别
```

### 外部 API 配置

```toml
[tmdb]
api_key = "your_api_key"        # TMDB API 密钥
base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p"
language = "zh-CN"

[bangumi_tv]
endpoint = "https://api.bgm.tv/v0"

[mikan]
endpoint = "https://mikanani.me"
```

### 下载器配置

```toml
[pan115]
cookies = "your_cookies"         # 115 网盘 Cookie
download_dir = "/downloads"      # 下载目录
max_requests_per_second = 2      # 最大请求速率
```

### 通知配置

```toml
[notify.telegram]
enabled = true                   # 是否启用 Telegram 通知
token = "bot_token"             # Bot Token
chat_id = "chat_id"             # 聊天 ID
```

### 解析器配置

```toml
[parser.siliconflow]            # 使用 SiliconFlow API 解析文件名
enabled = true
api_key = "your_api_key"
base_url = "https://api.siliconflow.com"
model = "gpt-4"
```

## 使用说明

1. 复制 `config.example.toml` 为 `config.toml` 并修改配置
2. 运行后端服务:
   ```bash
   ./target/release/bangumi-server
   ```
3. 访问 Web 界面: http://localhost:3001

## 许可证

MIT License

