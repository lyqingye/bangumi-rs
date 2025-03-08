# 构建说明

## 环境要求

- Rust 1.75+
- Node.js 18+
- MySQL 8.0+

## 后端构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/lyqingye/bangumi.git
cd bangumi

# 配置文件
cp config.example.toml config.toml

# 使用 Makefile 命令进行构建和运行
make build              # 开发模式构建
make build-release      # 发布模式构建
make run               # 运行开发版本
make run-dev           # 运行开发版本(支持 tokio console)
```

## 前端构建

```bash
# 进入前端目录
cd web

# 安装依赖
npm install

# 构建
npm run build

# 本地运行
npm run dev
```

