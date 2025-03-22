FROM --platform=$BUILDPLATFORM node:20-slim as builder

WORKDIR /app
COPY . .

# 安装依赖并构建
RUN npm install
RUN npm run build

# 使用多架构支持的基础镜像
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
# 添加默认 nginx 配置
COPY ./nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"] 