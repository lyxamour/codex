# Codex AI 常见问题解答

## 安装问题

### Q: 安装 Codex 时出现权限错误怎么办？

A: 如果你在安装过程中遇到权限错误，可以尝试以下解决方案：

1. **使用 sudo 权限**：在 Unix/Linux/macOS 系统上，可以在安装命令前加上 `sudo` 来获取管理员权限。
2. **使用自定义安装目录**：使用 `--install-dir` 选项将 Codex 安装到你有写入权限的目录，例如：
   ```bash
   curl -sSL https://raw.githubusercontent.com/lyxamour/codex/main/scripts/install/install.sh | bash -s -- --install-dir ~/.local/bin
   ```
3. **手动添加到 PATH**：将 Codex 二进制文件所在目录添加到你的 PATH 环境变量中。

### Q: 如何验证 Codex 安装成功？

A: 你可以通过以下命令验证 Codex 是否安装成功：

```bash
codex --version
```

如果安装成功，会显示当前 Codex 的版本信息，例如：
```
codex 1.0.0
```

## 配置问题

### Q: 如何设置 AI 平台的 API 密钥？

A: 你可以通过以下两种方式设置 AI 平台的 API 密钥：

1. **环境变量**：
   ```bash
   # OpenAI
   export OPENAI_API_KEY="your-openai-api-key"
   
   # Anthropic
   export ANTHROPIC_API_KEY="your-anthropic-api-key"
   ```

2. **配置文件**：编辑 `~/.codex/config.yaml` 文件，添加 API 密钥：
   ```yaml
   ai:
     platforms:
       openai:
         api_key: "your-openai-api-key"
       anthropic:
         api_key: "your-anthropic-api-key"
   ```

### Q: 如何切换默认 AI 平台？

A: 你可以通过以下方式切换默认 AI 平台：

1. **命令行选项**：使用 `--platform` 选项临时切换平台：
   ```bash
   codex ask --platform anthropic "你的问题"
   ```

2. **配置文件**：修改 `~/.codex/config.yaml` 文件，设置默认平台：
   ```yaml
   ai:
     default_platform: anthropic
   ```

## 使用问题

### Q: Codex 支持哪些编程语言？

A: Codex 目前支持以下编程语言：
- Rust
- Python
- JavaScript
- TypeScript
- C/C++
- Java
- Go

Codex 可以自动检测文件类型，并提供相应的代码分析和生成功能。

### Q: 如何使用 Codex 索引代码库？

A: 你可以使用以下命令索引代码库：

```bash
# 索引当前目录
codex index .

# 递归索引目录
codex index --recursive /path/to/your/codebase

# 排除特定目录
codex index --exclude target --exclude node_modules .
```

### Q: 如何提高 Codex 的响应质量？

A: 你可以通过以下方式提高 Codex 的响应质量：

1. **提供清晰的问题描述**：尽可能详细地描述你的问题或需求
2. **添加上下文**：使用 `--context` 选项添加相关文件或目录作为上下文
3. **使用适当的模型**：根据任务复杂度选择合适的模型
4. **使用提示词模板**：使用 Codex 内置的提示词模板来引导 AI 生成更准确的响应

## 性能问题

### Q: 索引大型代码库时速度很慢怎么办？

A: 索引大型代码库时，你可以尝试以下优化：

1. **排除不必要的目录**：使用 `--exclude` 选项排除不必要的目录，如 `target`、`node_modules`、`venv` 等
2. **限制文件大小**：在配置文件中设置 `max_file_size`，跳过过大的文件
3. **使用增量索引**：Codex 支持增量索引，只会索引新增或修改的文件

### Q: Codex 使用过程中内存占用很高怎么办？

A: 如果 Codex 使用过程中内存占用很高，你可以尝试以下解决方案：

1. **减少上下文大小**：使用 `--context` 选项时，只添加必要的文件或目录
2. **使用更小的模型**：在配置文件中设置更小的模型，如 `gpt-3.5-turbo` 而不是 `gpt-4o`
3. **关闭不必要的功能**：在配置文件中关闭不必要的功能，如 `enable_animations: false`

## 技术问题

### Q: Codex 支持本地模型吗？

A: 是的，Codex 支持使用 Ollama 运行的本地模型。你需要先安装 Ollama 并启动服务，然后在 Codex 配置文件中添加 Ollama 平台配置：

```yaml
ai:
  platforms:
    ollama:
      base_url: "http://localhost:11434"
      default_model: llama3
```

### Q: 如何开发自定义工具？

A: 你可以通过以下步骤开发自定义工具：

1. 创建一个 YAML 工具定义文件
2. 在文件中定义工具的名称、描述、参数和执行方式
3. 将工具定义文件放置在 `~/.codex/tools/custom/` 目录下
4. 重启 Codex 或使用 `codex plugin reload` 命令重新加载工具

### Q: 如何贡献代码？

A: 欢迎你为 Codex 贡献代码！贡献流程如下：

1. Fork Codex 仓库
2. 创建一个新的特性分支
3. 实现你的功能或修复
4. 编写测试用例
5. 提交代码并创建 Pull Request
6. 等待代码审查和合并

## 其他问题

### Q: Codex 是开源的吗？

A: 是的，Codex 是一个开源项目，使用 MIT 许可证。你可以在 GitHub 上找到 Codex 的源代码：https://github.com/lyxamour/codex

### Q: 如何获取支持？

A: 如果你在使用 Codex 过程中遇到问题，可以通过以下方式获取支持：

1. 查看 [用户指南](user_guide.md) 和 [开发者指南](dev_guide.md)
2. 检查 [常见问题解答](faq.md) 是否有相关解决方案
3. 在 GitHub Issues 上提交问题：https://github.com/lyxamour/codex/issues
4. 加入 Codex 社区讨论

### Q: 如何更新 Codex？

A: 你可以使用以下命令更新 Codex：

```bash
# 使用安装脚本更新
curl -sSL https://raw.githubusercontent.com/lyxamour/codex/main/scripts/install/install.sh | bash

# 或者使用 Codex 内置的更新命令
codex update
```

## 最佳实践

### Q: 如何有效地使用 Codex 进行代码解释？

A: 以下是一些有效使用 Codex 进行代码解释的技巧：

1. **提供完整的文件**：尽可能提供完整的文件，而不仅仅是片段
2. **指定语言**：明确指定代码的编程语言
3. **提出具体问题**：例如，"这个函数的主要功能是什么？" 或 "这段代码有什么潜在的性能问题？"
4. **添加上下文**：如果代码依赖其他文件，提供相关文件作为上下文

### Q: 如何使用 Codex 生成高质量代码？

A: 以下是一些生成高质量代码的技巧：

1. **详细描述需求**：尽可能详细地描述你的需求，包括输入、输出和预期行为
2. **指定语言和框架**：明确指定你想要使用的编程语言和框架
3. **提出代码质量要求**：例如，"请生成符合 Rust 最佳实践的代码" 或 "请生成带有完整测试用例的代码"
4. **提供示例输入输出**：如果可能，提供示例输入和预期输出
5. **迭代改进**：如果生成的代码不符合预期，可以进一步细化你的需求或提供反馈
