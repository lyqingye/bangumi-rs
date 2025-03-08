# 本地二进制部署指南

本地二进制部署是通过下载预编译的二进制文件直接运行 Bangumi-rs 的方法，适合不想使用 Docker 或资源有限的环境。本指南将详细介绍如何使用预编译的二进制文件部署 Bangumi-rs。

## 系统要求

- **支持的操作系统**:
  - Linux (x86_64, ARM64)
  - macOS (x86_64, ARM64)
  - Windows (x86_64)
- **最低硬件要求**:
  - CPU: 双核及以上
  - 内存: 1GB 及以上
  - 存储: 至少 1GB 可用空间（不包括下载内容存储）

## 下载二进制文件

### 从 GitHub Releases 下载

1. 访问 [Bangumi-rs GitHub Releases](https://github.com/bangumi-rs/bangumi/releases) 页面
2. 下载适合你系统的最新版本:
   - Linux: `bangumi-linux-x86_64.tar.gz` 或 `bangumi-linux-aarch64.tar.gz`
   - macOS: `bangumi-macos-x86_64.tar.gz` 或 `bangumi-macos-aarch64.tar.gz`
   - Windows: `bangumi-windows-x86_64.zip`

### 使用命令行下载

#### Linux (x86_64)

```bash
# 创建安装目录
mkdir -p ~/bangumi
cd ~/bangumi

# 下载最新版本
LATEST_VERSION=$(curl -s https://api.github.com/repos/bangumi-rs/bangumi/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
curl -L "https://github.com/bangumi-rs/bangumi/releases/download/${LATEST_VERSION}/bangumi-linux-x86_64.tar.gz" -o bangumi.tar.gz

# 解压文件
tar -xzf bangumi.tar.gz
rm bangumi.tar.gz
```

#### macOS (x86_64)

```bash
# 创建安装目录
mkdir -p ~/bangumi
cd ~/bangumi

# 下载最新版本
LATEST_VERSION=$(curl -s https://api.github.com/repos/bangumi-rs/bangumi/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
curl -L "https://github.com/bangumi-rs/bangumi/releases/download/${LATEST_VERSION}/bangumi-macos-x86_64.tar.gz" -o bangumi.tar.gz

# 解压文件
tar -xzf bangumi.tar.gz
rm bangumi.tar.gz
```

#### Windows (PowerShell)

```powershell
# 创建安装目录
New-Item -ItemType Directory -Force -Path C:\bangumi
cd C:\bangumi

# 下载最新版本
$LatestVersion = (Invoke-RestMethod -Uri "https://api.github.com/repos/bangumi-rs/bangumi/releases/latest").tag_name
Invoke-WebRequest -Uri "https://github.com/bangumi-rs/bangumi/releases/download/$LatestVersion/bangumi-windows-x86_64.zip" -OutFile "bangumi.zip"

# 解压文件
Expand-Archive -Path bangumi.zip -DestinationPath .
Remove-Item bangumi.zip
```

## 配置 Bangumi-rs

### 创建配置文件

在 Bangumi-rs 安装目录中创建配置文件:

#### Linux/macOS

```bash
cd ~/bangumi
mkdir -p config downloads
touch config/config.toml
```

#### Windows

```powershell
cd C:\bangumi
New-Item -ItemType Directory -Force -Path config
New-Item -ItemType Directory -Force -Path downloads
New-Item -ItemType File -Force -Path config\config.toml
```

### 编辑配置文件

使用文本编辑器编辑 `config/config.toml` 文件，添加以下基本配置:

```toml
[log]
level = "info"

[server]
assets_path = "./assets"
listen_addr = "127.0.0.1:3001"
database_url = "sqlite:bangumi.db"

[downloader]
download_dir = "./downloads"
```

根据需要调整配置，特别是:

- `listen_addr`: 如果需要从其他设备访问，可以设置为 `"0.0.0.0:3001"`
- `download_dir`: 下载文件的存储路径

## 运行 Bangumi-rs

### Linux/macOS

```bash
cd ~/bangumi
BANGUMI_CONFIG=./config/config.toml ./bangumi
```

### Windows

```powershell
cd C:\bangumi
$env:BANGUMI_CONFIG=".\config\config.toml"
.\bangumi.exe
```

## 设置为系统服务

### Linux (systemd)

1. 创建 systemd 服务文件:

```bash
sudo nano /etc/systemd/system/bangumi.service
```

2. 添加以下内容:

```ini
[Unit]
Description=Bangumi-rs Service
After=network.target

[Service]
Type=simple
User=your_username
WorkingDirectory=/home/your_username/bangumi
ExecStart=/home/your_username/bangumi/bangumi
Environment="BANGUMI_CONFIG=/home/your_username/bangumi/config/config.toml"
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

3. 启用并启动服务:

```bash
sudo systemctl daemon-reload
sudo systemctl enable bangumi
sudo systemctl start bangumi
```

4. 检查服务状态:

```bash
sudo systemctl status bangumi
```

### macOS (launchd)

1. 创建 launchd 配置文件:

```bash
nano ~/Library/LaunchAgents/com.bangumi.rs.plist
```

2. 添加以下内容:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.bangumi.rs</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/your_username/bangumi/bangumi</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>BANGUMI_CONFIG</key>
        <string>/Users/your_username/bangumi/config/config.toml</string>
    </dict>
    <key>WorkingDirectory</key>
    <string>/Users/your_username/bangumi</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/your_username/bangumi/logs/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/your_username/bangumi/logs/stderr.log</string>
</dict>
</plist>
```

3. 加载服务:

```bash
mkdir -p ~/bangumi/logs
launchctl load ~/Library/LaunchAgents/com.bangumi.rs.plist
```

4. 检查服务状态:

```bash
launchctl list | grep com.bangumi.rs
```

### Windows (服务)

使用 NSSM (Non-Sucking Service Manager) 创建 Windows 服务:

1. 下载 [NSSM](https://nssm.cc/download)
2. 解压并运行:

```powershell
.\nssm.exe install Bangumi
```

3. 在弹出的配置窗口中设置:

   - Path: `C:\bangumi\bangumi.exe`
   - Startup directory: `C:\bangumi`
   - Arguments: 留空
   - 在 "Environment" 选项卡添加: `BANGUMI_CONFIG=C:\bangumi\config\config.toml`

4. 点击 "Install service"
5. 启动服务:

```powershell
Start-Service Bangumi
```

## 更新 Bangumi-rs

更新 Bangumi-rs 二进制文件的步骤:

### Linux/macOS

```bash
# 停止服务
sudo systemctl stop bangumi  # Linux
launchctl unload ~/Library/LaunchAgents/com.bangumi.rs.plist  # macOS

# 备份当前版本
cd ~/bangumi
mv bangumi bangumi.old

# 下载并安装新版本
LATEST_VERSION=$(curl -s https://api.github.com/repos/bangumi-rs/bangumi/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
curl -L "https://github.com/bangumi-rs/bangumi/releases/download/${LATEST_VERSION}/bangumi-linux-x86_64.tar.gz" -o bangumi.tar.gz  # Linux
curl -L "https://github.com/bangumi-rs/bangumi/releases/download/${LATEST_VERSION}/bangumi-macos-x86_64.tar.gz" -o bangumi.tar.gz  # macOS
tar -xzf bangumi.tar.gz
rm bangumi.tar.gz

# 启动服务
sudo systemctl start bangumi  # Linux
launchctl load ~/Library/LaunchAgents/com.bangumi.rs.plist  # macOS
```

### Windows

```powershell
# 停止服务
Stop-Service Bangumi

# 备份当前版本
cd C:\bangumi
Rename-Item -Path bangumi.exe -NewName bangumi.old.exe

# 下载并安装新版本
$LatestVersion = (Invoke-RestMethod -Uri "https://api.github.com/repos/bangumi-rs/bangumi/releases/latest").tag_name
Invoke-WebRequest -Uri "https://github.com/bangumi-rs/bangumi/releases/download/$LatestVersion/bangumi-windows-x86_64.zip" -OutFile "bangumi.zip"
Expand-Archive -Path bangumi.zip -DestinationPath .
Remove-Item bangumi.zip

# 启动服务
Start-Service Bangumi
```

## 查看日志

### Linux (systemd)

```bash
sudo journalctl -u bangumi -f
```

### macOS (launchd)

```bash
tail -f ~/bangumi/logs/stdout.log
tail -f ~/bangumi/logs/stderr.log
```

### Windows (Event Viewer)

1. 打开事件查看器 (Event Viewer)
2. 导航到 "Windows Logs" > "Application"
3. 查找来源为 "Bangumi" 的事件

## 常见问题

### 权限问题

确保运行 Bangumi-rs 的用户对配置文件和下载目录有读写权限:

```bash
# Linux/macOS
chmod 755 ~/bangumi/bangumi
chmod -R 755 ~/bangumi/config
chmod -R 755 ~/bangumi/downloads
```

### 端口占用

如果 3001 端口已被占用，可以在配置文件中修改 `listen_addr` 使用其他端口。

### 数据库错误

如果遇到数据库错误，可能是数据库文件损坏，尝试备份并重新创建:

```bash
# Linux/macOS
cd ~/bangumi
mv bangumi.db bangumi.db.bak
```

```powershell
# Windows
cd C:\bangumi
Rename-Item -Path bangumi.db -NewName bangumi.db.bak
```

### 内存不足

对于低内存设备，可以调整配置以减少内存使用:

```toml
[downloader]
max_concurrent_tasks = 1  # 减少并行下载任务数
```

## 高级配置

### 使用外部数据库

默认情况下，Bangumi-rs 使用 SQLite 数据库，但你也可以配置使用 MySQL 或 PostgreSQL:

```toml
[server]
database_url = "mysql://username:password@localhost:3306/bangumi"
# 或
database_url = "postgres://username:password@localhost:5432/bangumi"
```

### 配置代理

如果需要通过代理访问网络资源:

```toml
[proxy]
enabled = true
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

### 配置通知

设置 Telegram 通知:

```toml
[notify.telegram]
enabled = true
token = "your-telegram-bot-token"
chat_id = "your-chat-id"
```

## 性能优化

### 调整并发任务数

根据系统性能调整并发下载任务数:

```toml
[downloader]
max_concurrent_tasks = 3  # 默认值，可根据系统性能调整
```

### 使用 SSD 存储

将数据库和下载目录放在 SSD 上可以提高性能:

```toml
[server]
database_url = "sqlite:/path/to/ssd/bangumi.db"

[downloader]
download_dir = "/path/to/ssd/downloads"
```

## 安全建议

1. **不要暴露服务到公网**：默认配置只监听本地地址 (127.0.0.1)
2. **使用非特权用户运行**：不要使用 root 或管理员账户运行服务
3. **定期备份数据**：特别是数据库文件
4. **保护配置文件**：配置文件可能包含敏感信息，确保适当的文件权限

```

```
