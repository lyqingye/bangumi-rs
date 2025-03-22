FROM node:20-slim as builder

WORKDIR /app
COPY . .

RUN npm install
RUN npm run build

# 使用多架构支持的基础镜像
FROM nginx:alpine
ARG TARGETPLATFORM
RUN echo "Building for platform: $TARGETPLATFORM"

COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"] 