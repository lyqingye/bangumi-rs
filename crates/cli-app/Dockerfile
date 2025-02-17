FROM rust:1.84-slim as builder

WORKDIR /usr/src/app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    make \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# 只复制必要的构建文件
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY libs ./libs

# 构建项目
RUN cargo build --release -p cli-app

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/bangumi /app/
COPY --from=builder /usr/src/app/config.toml /app/

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3001
CMD ["/app/bangumi"] 