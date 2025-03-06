# 快速开始

## 环境要求

- Docker (推荐)
- 或者
  - Rust 1.75+
  - Node.js 18+
  - MySQL 8.0+

## Docker 部署（推荐）

1. 克隆项目

```bash
git clone https://github.com/lyqingye/bangumi-rs.git
cd bangumi-rs
```

2. 配置文件

```bash
cp config.example.toml config.toml
```

3. 修改配置文件
   根据你的需求修改 `config.toml` 文件，详细配置说明请参考[配置文档](/config/)。

4. 启动服务

```bash
docker-compose up -d
```

5. 访问服务
   打开浏览器访问 http://localhost:80

## 手动部署

### 后端部署

1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 构建后端

```bash
make build-release  # 发布模式构建
```

3. 运行后端

```bash
./target/release/bangumi-rs
```

### 前端部署

1. 安装依赖

```bash
cd web
npm install
```

2. 构建前端

```bash
npm run build
```

3. 配置 Nginx

```nginx
server {
    listen 80;
    server_name localhost;

    location / {
        root /path/to/bangumi-rs/web/dist;
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://localhost:3001;
    }
}
```

## 下一步

- 查看[配置说明](/config/)了解如何配置系统
- 了解[功能特性](/guide/features.html)
- 参与[项目开发](/development/)

