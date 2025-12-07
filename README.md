# Codex AI 编程工具

Codex 是一个类似于 Claude Code、Codex 和 Droid 的基于 CLI 的 AI 编程工具，旨在提供高效的编程辅助和知识管理功能。

## 项目概述

Codex 是一个基于 Rust 开发的 CLI 工具，提供以下核心功能：

- **AI 编程辅助**：支持多种 AI 平台，包括 OpenAI、Anthropic、Google Gemini 等
- **本地代码知识库**：使用 Tantivy 全文搜索索引本地代码，支持快速查询
- **网页抓取**：支持远程知识抓取和索引
- **交互式 UI**：使用 Ratatui 提供终端交互式界面
- **工具系统**：基于 YAML 的工具系统，支持内置工具和外部工具扩展
- **提示词管理**：支持自定义提示词模板和变量替换

## 技术栈

- **主语言**: Rust
- **CLI 框架**: Clap
- **UI 框架**: Ratatui + Crossterm
- **知识库**: Tantivy (全文搜索) + Sled (元数据存储)
- **AI 集成**: Reqwest + Serde
- **网页抓取**: Scraper
- **代码解析**: Tree-sitter
- **异步运行时**: Tokio
- **配置管理**: Serde + TOML/YAML
- **工具系统**: YAML 格式调用 + 内置工具集

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-username/codex.git
cd codex

# 构建项目
cargo build --release

# 将二进制文件添加到 PATH
sudo cp target/release/codex /usr/local/bin/
```

### 安装依赖

Codex 需要以下依赖：

- Rust 1.70 或更高版本
- Git
- OpenAI API 密钥（可选，用于 AI 功能）

## 快速开始

### 设置环境变量

如果要使用 AI 功能，需要设置 API 密钥：

```bash
# OpenAI API 密钥
export OPENAI_API_KEY="your-openai-api-key"
```

### 基本用法

#### 启动交互式界面

```bash
codex
```

#### 直接提问

```bash
codex ask "如何使用 Rust 编写一个 HTTP 服务器？"
```

#### 解释代码

```bash
codex explain main.rs
```

#### 生成代码

```bash
codex generate "创建一个简单的 Rust 函数，计算斐波那契数列"
```

#### 索引代码库

```bash
codex index .
```

## 命令行选项

```
codex --help
```

## 配置

Codex 使用 YAML 配置文件，默认路径为 `~/.codex/config.yaml`。

### 配置示例

```yaml
app:
  name: codex
  version: "0.2.0"
  data_dir: ~/.codex
  log_level: info

ai:
  default_platform: openai
  openai:
    api_key: "your-openai-api-key"
    default_model: gpt-4o
    base_url: "https://api.openai.com/v1"

knowledge:
  index_dir: ~/.codex/index
  metadata_dir: ~/.codex/metadata
  exclude_patterns: ["target", ".git", "node_modules"]
  supported_extensions: ["rs", "py", "js", "ts", "json", "yaml", "toml", "md"]
```

## 架构

Codex 采用模块化设计，主要组件包括：

1. **CLI 层**: 处理命令行参数和输入输出
2. **Core 层**: 应用核心逻辑和生命周期管理
3. **AI 层**: AI 平台适配器和提示词管理
4. **Knowledge 层**: 代码索引和搜索
5. **Tools 层**: 工具系统和内置工具集
6. **UI 层**: 终端交互式界面
7. **Config 层**: 配置管理

## 开发

### 项目结构

```
codex/
├── src/
│   ├── cli/           # 命令行处理
│   ├── config/        # 配置管理
│   ├── core/          # 核心功能
│   ├── tools/         # 工具系统
│   ├── knowledge/     # 知识库
│   ├── ai/            # AI 集成
│   └── ui/            # 用户界面
├── templates/         # 提示词和工具模板
├── tests/             # 测试目录
├── Cargo.toml         # 项目配置
└── README.md          # 项目文档
```

### 构建命令

```bash
# 构建调试版本
cargo build

# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 运行 clippy
cargo clippy

# 运行格式化
cargo fmt
```

## 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献指南

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/fooBar`)
3. 提交更改 (`git commit -am 'Add some fooBar'`)
4. 推送到分支 (`git push origin feature/fooBar`)
5. 创建 Pull Request

## 许可证

MIT

## 联系方式

如有问题或建议，欢迎通过以下方式联系：

- GitHub Issues: https://github.com/your-username/codex/issues
- Email: your-email@example.com
