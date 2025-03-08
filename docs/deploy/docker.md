# Docker 部署指南

Docker 是部署 Bangumi-rs 最简单、最推荐的方式。本指南将详细介绍如何使用 Docker 和 Docker Compose 部署 Bangumi-rs。

## 前提条件

在开始之前，请确保你的系统已安装以下软件：

- Docker 20.10.0 或更高版本
- Docker Compose v2.0.0 或更高版本（可选，但推荐）

### 安装 Docker

如果你尚未安装 Docker，可以参考以下指南：

- [Docker 官方安装指南](https://docs.docker.com/get-docker/)

#### Linux 快速安装

```bash
curl -fsSL https://get.docker.com | sh
sudo systemctl enable --now docker
```

#### 验证 Docker 安装

```bash
docker --version
```

### 安装 Docker Compose

Docker Compose 可以简化多容器应用的部署和管理。

#### Linux 安装 Docker Compose

```bash
sudo curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

#### 验证 Docker Compose 安装

```bash
docker-compose --version
```

## 使用 Docker 运行 Bangumi-rs

### 方法一：使用 Docker 命令

1. **创建配置文件目录和下载目录**

```bash
mkdir -p ~/bangumi/config
mkdir -p ~/bangumi/downloads
```

2. **创建配置文件**

```bash
nano ~/bangumi/config/config.toml
```

添加以下基本配置（根据需要修改）：

```toml
[log]
level = "info"

[server]
assets_path = "/app/assets"
listen_addr = "0.0.0.0:3001"
database_url = "sqlite:/app/data/bangumi.db"

[downloader]
download_dir = "/downloads"
```

3. **拉取并运行 Docker 镜像**

```bash
docker run -d \
  --name bangumi \
  -p 3001:3001 \
  -v ~/bangumi/config:/app/config \
  -v ~/bangumi/downloads:/downloads \
  ghcr.io/bangumi-rs/bangumi:latest
```

### 方法二：使用 Docker Compose（推荐）

1. **创建项目目录**

```bash
mkdir -p ~/bangumi
cd ~/bangumi
```

2. **创建 Docker Compose 配置文件**

```bash
nano docker-compose.yml
```

添加以下内容：

```yaml
version: "3"

services:
  bangumi:
    image: ghcr.io/bangumi-rs/bangumi:latest
    container_name: bangumi
    restart: unless-stopped
    ports:
      - "3001:3001"
    volumes:
      - ./config:/app/config
      - ./data:/app/data
      - ./downloads:/downloads
    environment:
      - TZ=Asia/Shanghai
```

3. **创建配置文件目录和下载目录**

```bash
mkdir -p config data downloads
```

4. **创建配置文件**

```bash
nano config/config.toml
```

添加以下基本配置（根据需要修改）：

```toml
[log]
level = "info"

[server]
assets_path = "/app/assets"
listen_addr = "0.0.0.0:3001"
database_url = "sqlite:/app/data/bangumi.db"

[downloader]
download_dir = "/downloads"
```

5. **启动服务**

```bash
docker-compose up -d
```

## 高级配置

### 使用外部数据库

如果你想使用外部数据库（如 MySQL 或 PostgreSQL）而不是默认的 SQLite，可以修改配置文件：

```toml
[server]
database_url = "mysql://username:password@mysql:3306/bangumi"
```

然后在 Docker Compose 文件中添加数据库服务：

```yaml
version: "3"

services:
  bangumi:
    # ... 其他配置 ...
    depends_on:
      - mysql

  mysql:
    image: mysql:8.0
    container_name: bangumi-mysql
    restart: unless-stopped
    environment:
      - MYSQL_ROOT_PASSWORD=your_root_password
      - MYSQL_DATABASE=bangumi
      - MYSQL_USER=username
      - MYSQL_PASSWORD=password
    volumes:
      - ./mysql:/var/lib/mysql
```

### 配置代理

如果你需要通过代理访问某些资源，可以在配置文件中设置：

```toml
[proxy]
enabled = true
http = "http://proxy:7890"
https = "http://proxy:7890"
```

然后在 Docker Compose 文件中添加代理服务（如使用 Clash）：

```yaml
version: "3"

services:
  bangumi:
    # ... 其他配置 ...
    depends_on:
      - proxy

  proxy:
    image: dreamacro/clash
    container_name: clash
    restart: unless-stopped
    volumes:
      - ./clash:/root/.config/clash
    ports:
      - "7890:7890"
      - "7891:7891"
```

### 使用 Traefik 反向代理

如果你想通过域名访问 Bangumi-rs，可以使用 Traefik 作为反向代理：

```yaml
version: "3"

services:
  bangumi:
    # ... 其他配置 ...
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.bangumi.rule=Host(`bangumi.example.com`)"
      - "traefik.http.routers.bangumi.entrypoints=websecure"
      - "traefik.http.routers.bangumi.tls.certresolver=myresolver"
    networks:
      - traefik_network
      - default

networks:
  traefik_network:
    external: true
```

## 更新 Bangumi-rs

### 使用 Docker 命令更新

```bash
docker pull ghcr.io/bangumi-rs/bangumi:latest
docker stop bangumi
docker rm bangumi
# 然后重新运行上面的 docker run 命令
```

### 使用 Docker Compose 更新

```bash
cd ~/bangumi
docker-compose pull
docker-compose down
docker-compose up -d
```

## 查看日志

### 使用 Docker 命令查看日志

```bash
docker logs -f bangumi
```

### 使用 Docker Compose 查看日志

```bash
cd ~/bangumi
docker-compose logs -f
```

## 常见问题

### 容器无法启动

检查配置文件是否正确，可以查看容器日志：

```bash
docker logs bangumi
```

### 无法访问 Web 界面

确保端口映射正确，并检查防火墙设置：

```bash
docker ps
# 检查端口映射是否为 3001:3001
```

### 数据持久化问题

确保正确挂载了卷，特别是配置目录和数据目录：

```bash
docker inspect bangumi
# 检查 Mounts 部分
```

## 性能优化

### 资源限制

可以在 Docker Compose 文件中添加资源限制：

```yaml
services:
  bangumi:
    # ... 其他配置 ...
    deploy:
      resources:
        limits:
          cpus: "2"
          memory: 2G
        reservations:
          cpus: "0.5"
          memory: 512M
```

### 存储优化

对于大量下载，建议使用高性能存储：

```yaml
services:
  bangumi:
    # ... 其他配置 ...
    volumes:
      - ./config:/app/config
      - ./data:/app/data
      - /mnt/fast-storage:/downloads
```

## 安全建议

1. **不要暴露服务到公网**：除非你已配置了适当的身份验证和加密
2. **使用非 root 用户**：在 Docker Compose 文件中添加 `user: "1000:1000"`
3. **定期更新镜像**：保持软件为最新版本
4. **使用 HTTPS**：通过反向代理启用 HTTPS

## 完整示例

以下是一个完整的 Docker Compose 配置示例，包含了常用的设置：

```yaml
version: "3"

services:
  bangumi:
    image: ghcr.io/bangumi-rs/bangumi:latest
    container_name: bangumi
    restart: unless-stopped
    user: "1000:1000" # 使用非 root 用户
    ports:
      - "127.0.0.1:3001:3001" # 只监听本地接口
    volumes:
      - ./config:/app/config
      - ./data:/app/data
      - ./downloads:/downloads
    environment:
      - TZ=Asia/Shanghai
      - BANGUMI_LOG_LEVEL=info
    deploy:
      resources:
        limits:
          cpus: "2"
          memory: 2G
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/api/health"]
      interval: 1m
      timeout: 10s
      retries: 3
      start_period: 30s
```
