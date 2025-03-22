FROM rust:1.84-slim as builder

WORKDIR /usr/src/app

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

# 获取目标架构并设置相应的构建目标
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    "linux/arm64") RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
    *) RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    esac && \
    echo "Building for $TARGETPLATFORM, using Rust target $RUST_TARGET" && \
    rustup target add $RUST_TARGET && \
    echo "$RUST_TARGET" > /tmp/rust_target

# 只复制必要的构建文件
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY libs ./libs
COPY develop/migrations ./develop/migrations

# 构建项目，使用目标平台
RUN RUST_TARGET=$(cat /tmp/rust_target) && \
    cargo build --release --target $RUST_TARGET -p cli-app

FROM debian:bookworm-slim
WORKDIR /app

# 复制构建好的二进制文件，根据目标平台选择正确的路径
ARG TARGETPLATFORM
COPY --from=builder /tmp/rust_target /tmp/rust_target
RUN RUST_TARGET=$(cat /tmp/rust_target) && \
    echo "Target platform: $TARGETPLATFORM, Rust target: $RUST_TARGET"

COPY --from=builder /usr/src/app/target/${RUST_TARGET:-x86_64-unknown-linux-gnu}/release/bangumi /app/

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3001
CMD ["/app/bangumi"] 