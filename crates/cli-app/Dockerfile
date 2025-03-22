FROM rust:1.84-slim as builder

WORKDIR /usr/src/app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    make \
    gcc \
    g++ \
    wget \
    xz-utils \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 下载并安装 UPX
RUN wget https://github.com/upx/upx/releases/download/v5.0.0/upx-5.0.0-amd64_linux.tar.xz \
    && tar -xf upx-5.0.0-amd64_linux.tar.xz \
    && cp upx-5.0.0-amd64_linux/upx /usr/local/bin/ \
    && rm -rf upx-5.0.0-amd64_linux*

# 只复制必要的构建文件
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY libs ./libs
COPY develop/migrations ./develop/migrations

# 构建项目
RUN cargo build --release -p cli-app

# 使用 UPX 压缩二进制文件
RUN upx --best --lzma /usr/src/app/target/release/bangumi

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/bangumi /app/

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3001
CMD ["/app/bangumi"] 