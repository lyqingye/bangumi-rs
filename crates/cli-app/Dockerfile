FROM --platform=$BUILDPLATFORM rust:1.84-slim as builder

WORKDIR /usr/src/app

# 声明构建参数
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETOS
ARG TARGETARCH

# 根据目标架构设置 Rust 目标三元组
RUN case "$TARGETARCH" in \
        "amd64") RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
        "arm64") RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
        *) RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    esac && \
    echo "Building for $TARGETPLATFORM (OS: $TARGETOS, ARCH: $TARGETARCH), using Rust target $RUST_TARGET" && \
    rustup target add $RUST_TARGET && \
    echo "$RUST_TARGET" > /tmp/rust_target

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    make \
    gcc \
    g++ \
    wget \
    xz-utils \
    ca-certificates \
    musl-tools \
    llvm \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# 只复制必要的构建文件
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY libs ./libs
COPY develop/migrations ./develop/migrations

# 构建项目，使用目标平台
RUN RUST_TARGET=$(cat /tmp/rust_target) && \
    cargo build --release --target $RUST_TARGET -p cli-app && \
    cp target/$RUST_TARGET/release/bangumi /usr/src/app/bangumi

# 最终镜像 - 移除冗余的平台声明
FROM debian:bookworm-slim
WORKDIR /app

# 直接复制编译好的二进制文件
COPY --from=builder /usr/src/app/bangumi /app/

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3001
CMD ["/app/bangumi"] 