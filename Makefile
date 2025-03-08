# 项目名称
PROJECT_NAME := bangumi

# 构建目录
BUILD_DIR := ./build
BIN_DIR := $(BUILD_DIR)/bin

# 默认目标
.DEFAULT_GOAL := help

# 创建构建目录
$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

# 开发模式构建
.PHONY: build
build: $(BIN_DIR)
	@cargo build -p cli-app
	@cp target/debug/$(PROJECT_NAME) $(BIN_DIR)/
	@echo "二进制文件已复制到 $(BIN_DIR)/$(PROJECT_NAME)"

.PHONY: build-dev
build-dev: $(BIN_DIR)
	@RUSTFLAGS="--cfg tokio_unstable" cargo build -p cli-app --features "tokio_console"
	@cp target/debug/$(PROJECT_NAME) $(BIN_DIR)/
	@echo "二进制文件已复制到 $(BIN_DIR)/$(PROJECT_NAME)"

# 发布模式构建
.PHONY: build-release
build-release: $(BIN_DIR)
	@cargo build --release -p cli-app
	@cp target/release/$(PROJECT_NAME) $(BIN_DIR)/
	@echo "二进制文件已复制到 $(BIN_DIR)/$(PROJECT_NAME)"

# 运行项目
.PHONY: run
run: build
	@$(BIN_DIR)/$(PROJECT_NAME) start

.PHONY: run-dev
run-dev: build-dev
	@$(BIN_DIR)/$(PROJECT_NAME) start

# 运行测试
.PHONY: test
test:
	@cargo test

# 清理构建文件
.PHONY: clean
clean:
	@cargo clean
	@rm -rf $(BUILD_DIR)
	@echo "已清理构建目录 $(BUILD_DIR)"

# 格式化代码
.PHONY: fmt
fmt:
	@cargo fmt

# 运行clippy检查
.PHONY: clippy
clippy:
	@cargo clippy --all-targets --all-features -- -D warnings 

# 检查代码（不编译）
.PHONY: check
check:
	@cargo check

# 生成文档
.PHONY: doc
doc:
	@cargo doc --no-deps --open

# 运行所有检查
.PHONY: all
all: fmt clippy test build

# 生成 SeaORM entity
.PHONY: gen-entity
gen-entity:
	@if [ ! -f .env ]; then \
		echo "错误：找不到 .env 文件"; \
		exit 1; \
	fi
	@set -a && source .env && sea-orm-cli generate entity \
		-u "$$DATABASE_URL" \
		-o ./crates/model/src/entity \
		--with-serde both \
		--date-time-crate chrono

# 在 help 中添加新命令说明
.PHONY: help
help:
	@echo "可用命令:"
	@echo "  build       - 编译项目（开发模式）"
	@echo "  build-release - 编译项目（发布模式）"
	@echo "  run         - 运行项目"
	@echo "  test        - 运行测试"
	@echo "  clean       - 清理构建文件"
	@echo "  fmt         - 格式化代码"
	@echo "  clippy      - 运行clippy检查"
	@echo "  check       - 检查代码（不编译）"
	@echo "  doc         - 生成文档"
	@echo "  all         - 运行fmt, clippy, test, build"
	@echo "  help        - 显示此帮助信息"