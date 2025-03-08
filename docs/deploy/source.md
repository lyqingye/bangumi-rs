# 源码编译部署指南

从源码编译部署 Bangumi-rs 是最灵活的部署方式，适合开发者和高级用户。本指南将详细介绍如何从源代码构建和部署 Bangumi-rs。

## 系统要求

### 基本要求

- **操作系统**:
  - Linux (x86_64, ARM64)
  - macOS (x86_64, ARM64)
  - Windows (x86_64)
- **硬件要求**:
  - CPU: 双核及以上
  - 内存: 2GB 及以上（编译时可能需要更多）
  - 存储: 至少 2GB 可用空间（不包括下载内容存储）

### 开发环境要求

- **Rust 工具链**:
  - Rust 1.70.0 或更高版本
  - Cargo (Rust 包管理器)
- **前端开发环境**:
  - Node.js 18.0.0 或更高版本
  - npm 或 yarn
- **构建工具**:
  - Linux/macOS: gcc/clang, make, git
  - Windows: MSVC (Microsoft Visual C++), git

## 准备开发环境

### 安装 Rust 工具链

使用 rustup 安装 Rust 工具链:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

在 Windows 上，从 [rustup.rs](https://rustup.rs/) 下载并运行安装程序。

安装完成后，确认 Rust 已正确安装:

```bash
rustc --version
cargo --version
```

### 安装 Node.js 和 npm

#### Linux (使用 nvm)

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
source ~/.bashrc  # 或 source ~/.zshrc
nvm install 18
nvm use 18
```

#### macOS (使用 Homebrew)

```bash
brew install node@18
```

#### Windows

从 [Node.js 官网](https://nodejs.org/) 下载并安装 Node.js 18.x LTS 版本。

安装完成后，确认 Node.js 和 npm 已正确安装:

```bash
node --version
npm --version
```

### 安装其他依赖

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev git
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install gcc gcc-c++ make pkgconfig openssl-devel git
```

#### macOS

```bash
brew install openssl pkg-config git
```

#### Windows

安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 和 [Git for Windows](https://git-scm.com/download/win)。

## 获取源代码

### 克隆仓库

```bash
git clone https://github.com/bangumi-rs/bangumi.git
cd bangumi
```

### 切换到稳定版本 (可选)

如果你想使用稳定版本而不是最新的开发版本:

```bash
git fetch --tags
git checkout $(git describe --tags `git rev-list --tags --max-count=1`)
```

## 编译项目

### 编译后端 (Rust)

```bash
cargo build --release
```

编译完成后，二进制文件将位于 `target/release/bangumi` (Linux/macOS) 或 `target\release\bangumi.exe` (Windows)。

### 编译前端 (Vue)

```bash
cd frontend
npm install
npm run build
```

编译完成后，前端文件将位于 `frontend/dist` 目录。

### 整合前后端

将前端构建文件复制到后端资源目录:

```bash
# 在项目根目录执行
mkdir -p assets
cp -r frontend/dist/* assets/
```

在 Windows 上:

```powershell
# 在项目根目录执行
New-Item -ItemType Directory -Force -Path assets
Copy-Item -Path frontend\dist\* -Destination assets -Recurse
```

## 配置 Bangumi-rs

### 创建配置目录和文件

```bash
mkdir -p config downloads
touch config/config.toml
```

在 Windows 上:

```powershell
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

## 运行 Bangumi-rs

### 直接运行

```bash
BANGUMI_CONFIG=./config/config.toml ./target/release/bangumi
```

在 Windows 上:

```powershell
$env:BANGUMI_CONFIG=".\config\config.toml"
.\target\release\bangumi.exe
```

### 设置为系统服务

#### Linux (systemd)

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
WorkingDirectory=/path/to/bangumi
ExecStart=/path/to/bangumi/target/release/bangumi
Environment="BANGUMI_CONFIG=/path/to/bangumi/config/config.toml"
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

#### macOS (launchd)

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
        <string>/path/to/bangumi/target/release/bangumi</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>BANGUMI_CONFIG</key>
        <string>/path/to/bangumi/config/config.toml</string>
    </dict>
    <key>WorkingDirectory</key>
    <string>/path/to/bangumi</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/path/to/bangumi/logs/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/path/to/bangumi/logs/stderr.log</string>
</dict>
</plist>
```

3. 加载服务:

```bash
mkdir -p logs
launchctl load ~/Library/LaunchAgents/com.bangumi.rs.plist
```

#### Windows (服务)

使用 NSSM (Non-Sucking Service Manager) 创建 Windows 服务:

1. 下载 [NSSM](https://nssm.cc/download)
2. 解压并运行:

```powershell
.\nssm.exe install Bangumi
```

3. 在弹出的配置窗口中设置:

   - Path: 完整路径到 `target\release\bangumi.exe`
   - Startup directory: 项目根目录
   - Arguments: 留空
   - 在 "Environment" 选项卡添加: `BANGUMI_CONFIG=配置文件的完整路径`

4. 点击 "Install service"
5. 启动服务:

```powershell
Start-Service Bangumi
```

## 开发模式运行

如果你想在开发过程中运行 Bangumi-rs，可以使用以下命令:

### 后端开发模式

```bash
BANGUMI_CONFIG=./config/config.toml cargo run
```

在 Windows 上:

```powershell
$env:BANGUMI_CONFIG=".\config\config.toml"
cargo run
```

### 前端开发模式

```bash
cd frontend
npm run dev
```

这将启动前端开发服务器，通常在 `http://localhost:5173` 上运行。

## 自定义构建

### 启用特定功能

Bangumi-rs 支持通过 Cargo 特性 (features) 启用或禁用某些功能:

```bash
# 启用所有功能
cargo build --release --features full

# 启用特定功能
cargo build --release --features "feature1,feature2"
```

### 交叉编译

如果你想为其他平台编译 Bangumi-rs，可以使用 Rust 的交叉编译功能:

#### 为 ARM64 Linux 编译 (如树莓派)

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

#### 为 ARM64 macOS 编译 (Apple Silicon)

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

#### 为 Windows 编译 (从 Linux/macOS)

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## 更新源码

要更新到最新版本的源代码:

```bash
git pull
cargo build --release
cd frontend
npm install
npm run build
cd ..
cp -r frontend/dist/* assets/
```

在 Windows 上:

```powershell
git pull
cargo build --release
cd frontend
npm install
npm run build
cd ..
Copy-Item -Path frontend\dist\* -Destination assets -Recurse -Force
```

## 常见问题

### 编译错误

如果遇到编译错误，尝试以下步骤:

1. 更新 Rust 工具链:

```bash
rustup update
```

2. 清理构建缓存:

```bash
cargo clean
```

3. 检查依赖项是否完整:

```bash
# Linux (Debian/Ubuntu)
sudo apt install build-essential pkg-config libssl-dev

# macOS
brew install openssl pkg-config
```

### 前端构建错误

如果遇到前端构建错误，尝试以下步骤:

1. 更新 Node.js 和 npm:

```bash
# 使用 nvm
nvm install 18
nvm use 18

# 或直接更新 npm
npm install -g npm
```

2. 清理 npm 缓存:

```bash
npm cache clean --force
```

3. 重新安装依赖:

```bash
rm -rf node_modules
npm install
```

### 运行时错误

如果应用程序无法启动或运行时出错:

1. 检查日志输出
2. 确认配置文件格式正确
3. 确保资源目录包含前端文件
4. 检查数据库连接是否正确

## 高级开发

### 调试

使用 Rust 的调试工具:

```bash
RUST_BACKTRACE=1 cargo run
```

### 性能分析

使用 Rust 的性能分析工具:

```bash
cargo install flamegraph
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph
```

### 代码格式化

保持代码风格一致:

```bash
# Rust 代码
cargo fmt

# 前端代码
cd frontend
npm run lint
```

### 单元测试

运行测试确保代码质量:

```bash
# Rust 测试
cargo test

# 前端测试
cd frontend
npm run test
```

## 贡献代码

如果你想为 Bangumi-rs 贡献代码:

1. Fork 仓库
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

请确保你的代码遵循项目的代码风格，并包含适当的测试。
