FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl AS builder-amd64
WORKDIR /app
COPY . .
RUN cargo build --release -p cli-app

FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl AS builder-arm64
WORKDIR /app
COPY . .
RUN cargo build --release -p cli-app

FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache ca-certificates curl

EXPOSE 3001

ARG TARGETARCH
COPY --from=builder-amd64 /app/target/x86_64-unknown-linux-musl/release/bangumi /app/bangumi-amd64
COPY --from=builder-arm64 /app/target/aarch64-unknown-linux-musl/release/bangumi /app/bangumi-arm64
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        cp /app/bangumi-amd64 /app/bangumi; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        cp /app/bangumi-arm64 /app/bangumi; \
    fi && \
    chmod +x /app/bangumi && \
    rm -f /app/bangumi-amd64 /app/bangumi-arm64

CMD ["/app/bangumi"] 