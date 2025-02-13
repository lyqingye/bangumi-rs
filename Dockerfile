FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

# 构建release版本
RUN cargo build --release

FROM debian:bookworm-slim

# 安装必要的运行时依赖
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从builder阶段复制编译好的二进制文件
COPY --from=builder /usr/src/app/target/release/cli-app /app/cli-app

# 设置入口点
ENTRYPOINT ["/app/cli-app"] 