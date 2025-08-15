# 网络延迟测试工具

**[English](README_EN.md) | 中文**

这个项目是采用 Rust 构建的高性能网络延迟测试工具。可以指定 DNS/DoH 提供商解析目标 URL 地址，测试网络连接性与延迟情况。

## 功能特性

- **多重 DNS 配置**：使用系统 DNS、自定义 DNS 服务器或 DNS-over-HTTPS 提供商进行测试
- **综合统计**：详细的时序指标，包括 DNS 解析、连接和总响应时间
- **网络诊断**：运行测试前内置连接性和健康检查
- **彩色输出**：丰富的终端输出，带有颜色编码的性能指示器
- **灵活配置**：支持环境变量、命令行参数和 .env 文件
- **并发测试**：跨多个 DNS 配置并行执行
- **跨平台**：支持 Linux、macOS 和 Windows
- **多 URL 测试**：同时测试多个目标 URL，结果清晰分组
- **增强的性能分析**：真实时序分解，准确的快/慢分类

## v0.2.0 新增功能 - 完整更新系统

- **🚀 自动更新功能**：全面的版本管理系统，支持升级、降级和智能平台检测
  - **CLI 参数**：`--update` (`-u`)、`--version <版本>` (`-v`)、`--force` (`-f`)
  - **更新模式**：交互式版本选择、直接版本指定、强制降级
  - **多数据源**：智能回退系统（本地缓存 → GitHub Atom 订阅 → REST API）
  - **平台检测**：自动检测并过滤适合当前操作系统/架构的二进制文件
  - **地域优化**：中国大陆用户自动使用加速下载镜像
  - **安全特性**：降级保护、版本验证、备份能力和回滚支持

### 更新功能使用示例

```bash
# 检查可用更新（交互式）
nlt --update

# 更新到指定版本
nlt --update --version v0.1.8

# 强制降级（显示安全警告）
nlt --update --version 0.1.5 --force

# 获取详细更新帮助
nlt --help-topic update
```

## v0.1.6 功能特性

- **改进的 DNS 分组**：结果现在按 DNS 类型组织（系统 DNS → 自定义 DNS → DoH）
- **更短的命令**：二进制文件重命名为 `nlt`，更易使用（原为 `network-latency-tester`）
- **多 URL 支持**：同时测试多个目标，结果分组显示
- **始终可见的 URL**：目标 URL 始终显示，提高清晰度
- **真实时序数据**：修复了之前显示 "N/A" 值的时序测量问题
- **准确的建议**：使用 DNS 配置名称而非混淆的 "test_X" 引用
- **更好的性能分析**：真实分类而非错误的"慢"消息

## 安装

### 从源码安装

```bash
git clone https://github.com/MaurUppi/network-latency-tester
cd network-latency-tester
cargo build --release
```

二进制文件将位于 `target/release/nlt`。


## 快速开始

```bash
# 使用系统 DNS 测试默认目标
./target/release/nlt

# 测试特定 URL
./target/release/nlt --url https://example.com

# 使用自定义 DNS 服务器测试 10 次迭代
./target/release/nlt --count 10 --timeout 5

# 启用调试模式获得详细输出
./target/release/nlt --debug --verbose

# 使用不同 DNS 配置测试多个 URL
./target/release/nlt --url https://httpbin.org,https://example.com --count 3
```

## 配置

### 命令行选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `--url <URL>` | 要测试的目标 URL | `https://bing.com` |
| `--count <N>` | 测试迭代次数 | `5` |
| `--timeout <SECONDS>` | 请求超时时间（秒） | `10` |
| `--no-color` | 禁用彩色输出 | `false` |
| `--verbose` | 启用详细输出 | `false` |
| `--debug` | 启用调试输出 | `false` |
| `--test-original` | 测试原始 target URL | `false` |
| `--update` (`-u`) | 检查更新和管理版本 | `false` |
| `--version <VERSION>` (`-v`) | 指定更新/降级的目标版本 | - |
| `--force` (`-f`) | 强制版本变更，包括降级 | `false` |
| `--help` | 显示帮助信息 | - |

### 环境变量

在项目目录中创建 `.env` 文件（参考 `.env.example`）：

| 变量 | 描述 | 示例 |
|------|------|------|
| `TARGET_URLS` | 要测试的 URL 列表（逗号分隔） | `https://example.com,https://google.com` |
| `DNS_SERVERS` | DNS 服务器 IP 列表（逗号分隔） | `8.8.8.8,1.1.1.1,208.67.222.222` |
| `DOH_PROVIDERS` | DoH URL 列表（逗号分隔） | `https://cloudflare-dns.com/dns-query` |
| `TEST_COUNT` | 测试迭代次数（1-100） | `5` |
| `TIMEOUT_SECONDS` | 请求超时时间秒数（1-300） | `10` |
| `ENABLE_COLOR` | 启用彩色输出 | `true` |

### 配置优先级

配置值按以下顺序应用（优先级从高到低）：

1. 命令行参数
2. 环境变量
3. `.env` 文件值
4. 默认值

## 使用示例

### 基本用法

```bash
# 使用默认配置测试
./nlt

# 使用自定义设置测试特定 URL
./nlt --url https://api.github.com --count 10 --timeout 15
```

### 高级配置

```bash
# 创建包含自定义配置的 .env 文件
cat > .env << EOF
TARGET_URLS=https://bing.com,https://api.openai.com,https://www.google.com
DNS_SERVERS=8.8.8.8,1.1.1.1,208.67.222.222,9.9.9.9
DOH_PROVIDERS=https://cloudflare-dns.com/dns-query,https://dns.google/dns-query
TEST_COUNT=10
TIMEOUT_SECONDS=5
ENABLE_COLOR=true
EOF

# 使用环境配置运行测试
./nlt --verbose
```

### 性能测试

```bash
# 高频测试用于性能分析
./nlt --count 20 --timeout 3 --verbose

# 比较不同 DNS 提供商
./nlt --debug --url https://example.com

# 同时测试多个目标
./nlt --url https://httpbin.org,https://example.com,https://google.com --count 5
```

## 输出格式

工具提供详细输出，包括：

- **DNS 验证**：测试前检查 DNS 配置有效性
- **测试进度**：执行期间的实时进度更新
- **性能表格**：颜色编码的响应时间和成功率
- **统计分析**：包括百分位数和置信区间的综合统计
- **网络诊断**：系统健康和连接性评估
- **建议**：性能最佳的 DNS 配置

### 输出示例

```
═════════════════════════════════════
  🎯 网络延迟测试结果  
═════════════════════════════════════

📊 执行摘要
⏱️  持续时间:     1m0.0s
🧪 总测试数:  10
✅ 成功:   10 (100.0%)

🚀 性能结果

🎯 目标: https://httpbin.org
───────────────────────────────────────────────────────────────────────────────────────────────
配置                                        成功率 平均响应时间         最小/最大        等级
───────────────────────────────────────────────────────────────────────────────────────────────
🥇 系统 DNS                                100.0% [████████] 68ms       65ms/71ms      ⚡ 良好
🥈 自定义 DNS (8.8.8.8)                      100.0% [████████] 52ms       49ms/55ms 🚀 优秀  
🥉 自定义 DNS (1.1.1.1)                      100.0% [████████] 58ms       55ms/61ms 🚀 优秀
   DoH (https://cloudflare-dns.com/...)       100.0% [████████] 45ms       42ms/48ms 🚀 优秀

🎯 目标: https://example.com
───────────────────────────────────────────────────────────────────────────────────────────────
配置                                        成功率 平均响应时间         最小/最大        等级
───────────────────────────────────────────────────────────────────────────────────────────────
🏆 系统 DNS                                100.0% [████████] 38ms       35ms/41ms 🚀 优秀
   自定义 DNS (8.8.8.8)                      100.0% [████████] 43ms       40ms/46ms 🚀 优秀
   自定义 DNS (1.1.1.1)                      100.0% [████████] 49ms       46ms/52ms 🚀 优秀
   DoH (https://cloudflare-dns.com/...)       100.0% [████████] 67ms       64ms/70ms      ⚡ 良好

💡 建议
🎯 使用系统 DNS 获得最佳性能
✨ 网络性能看起来不错！
```

## DNS 配置

### 系统 DNS

使用系统的默认 DNS 解析器配置。

```bash
./nlt  # 使用系统 DNS
```

### 自定义 DNS 服务器

通过环境变量指定自定义 DNS 服务器：

```bash
export DNS_SERVERS="8.8.8.8,1.1.1.1,208.67.222.222"
./nlt
```

### DNS-over-HTTPS (DoH)

配置 DoH 提供商以增强隐私：

```bash
export DOH_PROVIDERS="https://cloudflare-dns.com/dns-query,https://dns.google/dns-query"
./nlt
```

### 常用 DNS 提供商

| 提供商 | IP 地址 | DoH URL |
|--------|---------|---------|
| Google | `8.8.8.8`, `8.8.4.4` | `https://dns.google/dns-query` |
| Cloudflare | `1.1.1.1`, `1.0.0.1` | `https://cloudflare-dns.com/dns-query` |
| Quad9 | `9.9.9.9`, `149.112.112.112` | `https://dns.quad9.net/dns-query` |
| OpenDNS | `208.67.222.222`, `208.67.220.220` | - |
| 阿里巴巴 | `223.5.5.5`, `223.6.6.6` | - |

## 错误处理

工具提供有用的错误消息和建议：

### 配置错误
- 检查 .env 文件格式
- 验证 URL 格式（必须以 http:// 或 https:// 开头）
- 确保 DNS 服务器 IP 有效
- DoH URL 必须使用 HTTPS

### 网络错误（必然如下有一个有问题）
- 检查互联网连接
- 尝试不同的 DNS 服务器
- 验证防火墙设置
- 使用不同的目标 URL 进行测试

### DNS 解析错误
- 尝试使用公共 DNS 服务器（8.8.8.8, 1.1.1.1）
- 检查域名是否存在
- 使用 `nslookup` 或 `dig @1.1.1.1 apple.com` 手动测试 DNS 解析

## 开发

### 先决条件

- Rust 1.70+（用于 async/await 支持）
- Cargo 包管理器

### 构建

```bash
# 调试构建
cargo build

# 发布构建（优化）
cargo build --release

# 运行测试
cargo test

# 带日志运行
RUST_LOG=debug cargo run -- --debug
```

### 项目结构

```
src/
├── main.rs              # CLI 应用程序入口点
├── lib.rs               # 库导出和常量
├── cli.rs               # 命令行界面定义
├── app.rs               # 应用程序核心逻辑
├── error.rs             # 错误处理系统
├── types.rs             # 核心类型定义
├── models/              # 数据模型和结构
│   ├── mod.rs
│   ├── config.rs        # 配置模型
│   └── metrics.rs       # 计时和测量模型
├── config/              # 配置管理
│   ├── mod.rs
│   ├── parser.rs        # 配置解析和合并
│   └── validation.rs    # 配置验证
├── dns.rs               # DNS 配置和解析
├── client.rs            # 带时序测量的 HTTP 客户端
├── executor.rs          # 测试执行引擎
├── stats.rs             # 统计分析和计算
├── diagnostics.rs       # 网络诊断和健康检查
└── output/              # 输出格式和显示
    ├── mod.rs
    ├── formatter.rs     # 纯文本格式
    └── colored.rs       # 颜色编码格式
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试模块
cargo test config::parser::tests

# 运行集成测试
cargo test --test integration_tests
```

### 贡献

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature-name`)
3. 进行更改
4. 为新功能添加测试
5. 确保所有测试通过 (`cargo test`)
6. 运行格式化 (`cargo fmt`) 和检查 (`cargo clippy`)
7. 创建拉取请求

## 许可证

本项目采用 MIT 许可证 - 详情请见 LICENSE 文件。

## 致谢

- 使用 [Rust](https://www.rust-lang.org/) 构建，注重性能和安全性
- 使用 [tokio](https://tokio.rs/) 进行异步网络处理
- HTTP 请求由 [reqwest](https://github.com/seanmonstar/reqwest) 提供支持
- CLI 界面使用 [clap](https://github.com/clap-rs/clap) 构建
- 终端颜色通过 [colored](https://github.com/mackwic/colored) 实现

## 迁移说明

这个 Rust 实现与原始 bash 脚本 `check_ctok-v2.sh` 提供功能对等，同时提供：

- **更好的性能**：并发执行和优化的网络处理
- **增强的可靠性**：全面的错误处理和验证
- **改进的可用性**：丰富的终端输出和配置选项
- **跨平台支持**：在不同操作系统上工作一致
- **可维护性**：类型安全的代码和全面的测试覆盖

该工具保持与原始脚本输出格式的向后兼容性，同时提供额外的功能和改进。