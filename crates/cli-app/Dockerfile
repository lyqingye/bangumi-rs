# 简单的多阶段构建，仅用于复制预编译的二进制文件
FROM alpine:3.19 AS base
WORKDIR /app
RUN apk add --no-cache \
    ca-certificates \
    curl
EXPOSE 3001

# 这个ARG将从外部传入
ARG TARGETARCH

# 根据目标架构选择正确的预编译二进制文件
COPY ./target/${TARGETARCH}/release/bangumi /app/

CMD ["/app/bangumi"] 