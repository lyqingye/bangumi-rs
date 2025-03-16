# Bangumi CLI 应用

## 编译优化指南

本项目针对不同场景配置了不同的编译优化选项：

### 开发环境编译

开发环境优化主要关注编译速度，减少开发过程中的等待时间。使用以下命令编译：

```bash
cargo build
```

开发环境编译优化：

- 最小化优化级别 (opt-level = 0)
- 禁用链接时间优化 (LTO)
- 最大化并行编译单元 (codegen-units = 256)
- 启用增量编译
- 禁用调试符号以加快编译
- 禁用内联优化
- 共享泛型实例

### 生产环境编译

生产环境优化追求极致性能，不考虑二进制文件体积和编译耗时。使用以下命令编译：

```bash
cargo build --release
```

生产环境编译优化：

- 最高级别的优化 (opt-level = 3)
- 启用全局链接时间优化 (LTO = "fat")
- 最小化并行编译单元 (codegen-units = 1)
- 使用 abort 而非 unwind 进行 panic 处理
- 针对当前 CPU 架构优化 (target-cpu=native)
- 提高内联阈值
- 激进向量化
- 禁用溢出检查

### 高级编译优化 (需要 nightly 编译器)

如果您使用 nightly 编译器，本项目支持配置文件引导优化 (Profile Guided Optimization)：

1. 首先编译生成 instrumented 二进制文件：

```bash
rustup override set nightly
cargo build --release
```

2. 运行程序生成性能分析数据：

```bash
./target/release/bangumi <典型工作负载>
```

3. 使用收集的分析数据重新编译：

```bash
cargo build --release
```

4. 恢复到稳定版编译器（可选）：

```bash
rustup override unset
```

## 性能注意事项

- 二进制文件使用 jemalloc 内存分配器，提供更好的性能和内存管理
- 生产环境构建已针对性能做了极致优化，可能会增加编译时间和二进制体积
- 为特定 CPU 架构优化，可能在不同硬件上有不同表现

