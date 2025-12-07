# Codex AI API 参考

## 1. 简介

本文档提供了 Codex AI 的 API 参考，包括核心 API、工具 API、AI API 等。这些 API 可以用于扩展 Codex 的功能、开发插件或与其他系统集成。

## 2. 核心 API

### 2.1 App 结构体

`App` 是 Codex 应用程序的核心结构体，负责管理所有组件的生命周期。

#### 定义

```rust
pub struct App {
    pub config: Config,
    pub plugin_manager: PluginManager,
    pub tool_registry: ToolRegistry,
    pub ai_provider_factory: AIProviderFactory,
    pub knowledge_base: KnowledgeBase,
    pub prompt_manager: PromptManager,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `new()` | 创建新的 App 实例 | `config: Config` | `Result<Self, AppError>` |
| `initialize()` | 初始化应用程序 | 无 | `Result<(), AppError>` |
| `shutdown()` | 关闭应用程序 | 无 | `Result<(), AppError>` |
| `get_tool_registry()` | 获取工具注册表 | 无 | `&ToolRegistry` |
| `get_ai_provider()` | 获取 AI 平台提供者 | `platform: &str` | `Result<Box<dyn AIProvider>, AppError>` |
| `get_knowledge_base()` | 获取知识库 | 无 | `&KnowledgeBase` |

### 2.2 Config 结构体

`Config` 包含 Codex 的所有配置信息。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ai: AIConfig,
    pub knowledge: KnowledgeConfig,
    pub ui: UIConfig,
    pub logging: LoggingConfig,
    pub tools: ToolsConfig,
    // 其他字段...
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `ai` | `AIConfig` | AI 平台配置 |
| `knowledge` | `KnowledgeConfig` | 知识库配置 |
| `ui` | `UIConfig` | 用户界面配置 |
| `logging` | `LoggingConfig` | 日志配置 |
| `tools` | `ToolsConfig` | 工具系统配置 |

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `load()` | 从文件加载配置 | `path: Option<&Path>` | `Result<Self, ConfigError>` |
| `save()` | 保存配置到文件 | `path: Option<&Path>` | `Result<(), ConfigError>` |
| `validate()` | 验证配置有效性 | 无 | `Result<(), ConfigError>` |

## 3. 工具系统 API

### 3.1 ToolRegistry 结构体

`ToolRegistry` 管理所有可用的工具。

#### 定义

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `register_tool()` | 注册工具 | `tool: Box<dyn Tool>` | `Result<(), ToolError>` |
| `unregister_tool()` | 注销工具 | `name: &str` | `Result<(), ToolError>` |
| `get_tool()` | 获取工具 | `name: &str` | `Result<&Box<dyn Tool>, ToolError>` |
| `list_tools()` | 列出所有工具 | 无 | `Vec<&Box<dyn Tool>>` |
| `execute_tool()` | 执行工具 | `name: &str, params: &Value` | `Result<Value, ToolError>` |

### 3.2 Tool trait

`Tool` 是所有工具必须实现的 trait。

#### 定义

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn category(&self) -> ToolCategory;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn execute(&self, params: &Value) -> Result<Value, ToolError>;
}
```

#### 方法

| 方法 | 描述 | 返回值 |
|------|------|--------|
| `name()` | 获取工具名称 | `&str` |
| `description()` | 获取工具描述 | `&str` |
| `version()` | 获取工具版本 | `&str` |
| `category()` | 获取工具类别 | `ToolCategory` |
| `parameters()` | 获取工具参数 | `Vec<ToolParameter>` |
| `execute()` | 执行工具 | `Result<Value, ToolError>` |

### 3.3 ToolParameter 结构体

`ToolParameter` 定义工具的参数信息。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub description: String,
    pub default: Option<Value>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | `String` | 参数名称 |
| `type` | `String` | 参数类型 |
| `required` | `bool` | 是否必填 |
| `description` | `String` | 参数描述 |
| `default` | `Option<Value>` | 默认值 |

## 4. AI 系统 API

### 4.1 AIProvider trait

`AIProvider` 是所有 AI 平台必须实现的 trait。

#### 定义

```rust
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    fn chat_completion(&self, messages: &[ChatMessage]) -> Result<ChatResponse, AIError>;
    fn completion(&self, prompt: &str) -> Result<String, AIError>;
    fn embedding(&self, text: &str) -> Result<Vec<f32>, AIError>;
    fn list_models(&self) -> Result<Vec<ModelInfo>, AIError>;
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `name()` | 获取 AI 平台名称 | 无 | `&str` |
| `chat_completion()` | 生成聊天完成 | `messages: &[ChatMessage]` | `Result<ChatResponse, AIError>` |
| `completion()` | 生成文本完成 | `prompt: &str` | `Result<String, AIError>` |
| `embedding()` | 生成文本嵌入 | `text: &str` | `Result<Vec<f32>, AIError>` |
| `list_models()` | 列出可用模型 | 无 | `Result<Vec<ModelInfo>, AIError>` |

### 4.2 ChatMessage 结构体

`ChatMessage` 表示聊天消息。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `role` | `MessageRole` | 消息角色 |
| `content` | `String` | 消息内容 |
| `name` | `Option<String>` | 消息发送者名称 |

### 4.3 ChatResponse 结构体

`ChatResponse` 表示聊天响应。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub role: MessageRole,
    pub model: String,
    pub usage: Option<UsageInfo>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `content` | `String` | 响应内容 |
| `role` | `MessageRole` | 响应角色 |
| `model` | `String` | 使用的模型 |
| `usage` | `Option<UsageInfo>` | 模型使用信息 |

### 4.4 MessageRole 枚举

`MessageRole` 表示消息的角色。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "function")]
    Function,
}
```

## 5. 知识库 API

### 5.1 KnowledgeBase 结构体

`KnowledgeBase` 管理代码索引和搜索。

#### 定义

```rust
pub struct KnowledgeBase {
    indexer: Indexer,
    searcher: Searcher,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `index()` | 索引代码 | `path: &Path, options: &IndexOptions` | `Result<IndexStats, KnowledgeError>` |
| `search()` | 搜索代码 | `query: &str, options: &SearchOptions` | `Result<Vec<SearchResult>, KnowledgeError>` |
| `get_file()` | 获取文件内容 | `path: &Path` | `Result<FileInfo, KnowledgeError>` |
| `list_files()` | 列出索引文件 | `options: &ListOptions` | `Result<Vec<FileInfo>, KnowledgeError>` |
| `clear_index()` | 清除索引 | `path: Option<&Path>` | `Result<(), KnowledgeError>` |

### 5.2 IndexOptions 结构体

`IndexOptions` 定义索引选项。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexOptions {
    pub recursive: bool,
    pub exclude: Vec<String>,
    pub max_file_size: Option<u64>,
    pub force: bool,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `recursive` | `bool` | 是否递归索引 |
| `exclude` | `Vec<String>` | 排除的目录 |
| `max_file_size` | `Option<u64>` | 最大文件大小 |
| `force` | `bool` | 是否强制重新索引 |

### 5.3 SearchOptions 结构体

`SearchOptions` 定义搜索选项。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchOptions {
    pub path: Option<String>,
    pub file_pattern: Option<String>,
    pub language: Option<String>,
    pub max_results: Option<usize>,
    pub case_sensitive: bool,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `path` | `Option<String>` | 搜索路径 |
| `file_pattern` | `Option<String>` | 文件模式 |
| `language` | `Option<String>` | 编程语言 |
| `max_results` | `Option<usize>` | 最大结果数 |
| `case_sensitive` | `bool` | 是否区分大小写 |

### 5.4 SearchResult 结构体

`SearchResult` 表示搜索结果。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub score: f32,
    pub lines: Vec<SearchResultLine>,
    pub language: Option<String>,
    pub file_size: u64,
    pub modified_time: u64,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `path` | `String` | 文件路径 |
| `score` | `f32` | 匹配分数 |
| `lines` | `Vec<SearchResultLine>` | 匹配的行 |
| `language` | `Option<String>` | 编程语言 |
| `file_size` | `u64` | 文件大小 |
| `modified_time` | `u64` | 修改时间 |

## 6. 插件系统 API

### 6.1 PluginManager 结构体

`PluginManager` 管理所有插件。

#### 定义

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `load_plugins()` | 加载插件 | `paths: &[PathBuf]` | `Result<(), PluginError>` |
| `register_plugin()` | 注册插件 | `plugin: Box<dyn Plugin>` | `Result<(), PluginError>` |
| `unregister_plugin()` | 注销插件 | `name: &str` | `Result<(), PluginError>` |
| `get_plugin()` | 获取插件 | `name: &str` | `Result<&Box<dyn Plugin>, PluginError>` |
| `list_plugins()` | 列出所有插件 | 无 | `Vec<&Box<dyn Plugin>>` |
| `initialize_plugins()` | 初始化所有插件 | `app: &App` | `Result<(), PluginError>` |
| `shutdown_plugins()` | 关闭所有插件 | 无 | `Result<(), PluginError>` |

### 6.2 Plugin trait

`Plugin` 是所有插件必须实现的 trait。

#### 定义

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn initialize(&self, app: &App) -> Result<(), PluginError>;
    fn shutdown(&self) -> Result<(), PluginError>;
    fn get_hooks(&self) -> Vec<Box<dyn Hook>>;
    fn get_commands(&self) -> Vec<Box<dyn Command>>;
    fn get_tools(&self) -> Vec<Box<dyn Tool>>;
}
```

#### 方法

| 方法 | 描述 | 返回值 |
|------|------|--------|
| `name()` | 获取插件名称 | `&str` |
| `version()` | 获取插件版本 | `&str` |
| `description()` | 获取插件描述 | `&str` |
| `initialize()` | 初始化插件 | `Result<(), PluginError>` |
| `shutdown()` | 关闭插件 | `Result<(), PluginError>` |
| `get_hooks()` | 获取插件钩子 | `Vec<Box<dyn Hook>>` |
| `get_commands()` | 获取插件命令 | `Vec<Box<dyn Command>>` |
| `get_tools()` | 获取插件工具 | `Vec<Box<dyn Tool>>` |

### 6.3 Hook trait

`Hook` 是插件钩子必须实现的 trait。

#### 定义

```rust
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn hook_point(&self) -> &str;
    fn priority(&self) -> u32;
    fn execute(&self, ctx: &mut HookContext) -> Result<(), HookError>;
}
```

#### 方法

| 方法 | 描述 | 返回值 |
|------|------|--------|
| `name()` | 获取钩子名称 | `&str` |
| `hook_point()` | 获取钩子点 | `&str` |
| `priority()` | 获取钩子优先级 | `u32` |
| `execute()` | 执行钩子 | `Result<(), HookError>` |

## 7. 任务系统 API

### 7.1 TaskManager 结构体

`TaskManager` 管理所有任务。

#### 定义

```rust
pub struct TaskManager {
    tasks: HashMap<Uuid, Task>,
    scheduler: Scheduler,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `add_task()` | 添加任务 | `task: Task` | `Result<Uuid, TaskError>` |
| `get_task()` | 获取任务 | `id: &Uuid` | `Result<&Task, TaskError>` |
| `update_task()` | 更新任务 | `id: &Uuid, task: Task` | `Result<(), TaskError>` |
| `delete_task()` | 删除任务 | `id: &Uuid` | `Result<(), TaskError>` |
| `list_tasks()` | 列出任务 | `filter: Option<TaskFilter>` | `Result<Vec<&Task>, TaskError>` |
| `start_task()` | 启动任务 | `id: &Uuid` | `Result<(), TaskError>` |
| `stop_task()` | 停止任务 | `id: &Uuid` | `Result<(), TaskError>` |
| `complete_task()` | 完成任务 | `id: &Uuid` | `Result<(), TaskError>` |

### 7.2 Task 结构体

`Task` 表示一个任务。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: u64,
    pub updated_at: u64,
    pub completed_at: Option<u64>,
    pub dependencies: Vec<Uuid>,
    pub metadata: HashMap<String, Value>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | `Uuid` | 任务 ID |
| `name` | `String` | 任务名称 |
| `description` | `String` | 任务描述 |
| `status` | `TaskStatus` | 任务状态 |
| `priority` | `TaskPriority` | 任务优先级 |
| `created_at` | `u64` | 创建时间 |
| `updated_at` | `u64` | 更新时间 |
| `completed_at` | `Option<u64>` | 完成时间 |
| `dependencies` | `Vec<Uuid>` | 依赖任务 |
| `metadata` | `HashMap<String, Value>` | 任务元数据 |

### 7.3 TaskStatus 枚举

`TaskStatus` 表示任务状态。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}
```

### 7.4 TaskPriority 枚举

`TaskPriority` 表示任务优先级。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum TaskPriority {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "urgent")]
    Urgent,
}
```

## 8. 提示词系统 API

### 8.1 PromptManager 结构体

`PromptManager` 管理所有提示词。

#### 定义

```rust
pub struct PromptManager {
    prompts: HashMap<String, PromptTemplate>,
    // 其他字段...
}
```

#### 方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `load_prompts()` | 加载提示词 | `paths: &[PathBuf]` | `Result<(), PromptError>` |
| `register_prompt()` | 注册提示词 | `prompt: PromptTemplate` | `Result<(), PromptError>` |
| `unregister_prompt()` | 注销提示词 | `name: &str` | `Result<(), PromptError>` |
| `get_prompt()` | 获取提示词 | `name: &str` | `Result<&PromptTemplate, PromptError>` |
| `list_prompts()` | 列出提示词 | `category: Option<&str>` | `Vec<&PromptTemplate>` |
| `render_prompt()` | 渲染提示词 | `name: &str, variables: &HashMap<String, Value>` | `Result<String, PromptError>` |

### 8.2 PromptTemplate 结构体

`PromptTemplate` 表示一个提示词模板。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub variables: Vec<PromptVariable>,
    pub template: String,
    pub examples: Vec<PromptExample>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | `String` | 提示词名称 |
| `description` | `String` | 提示词描述 |
| `version` | `String` | 提示词版本 |
| `category` | `String` | 提示词类别 |
| `tags` | `Vec<String>` | 提示词标签 |
| `variables` | `Vec<PromptVariable>` | 提示词变量 |
| `template` | `String` | 提示词模板 |
| `examples` | `Vec<PromptExample>` | 示例输入输出 |

### 8.3 PromptVariable 结构体

`PromptVariable` 表示提示词变量。

#### 定义

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptVariable {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<Value>,
}
```

#### 字段

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | `String` | 变量名称 |
| `type` | `String` | 变量类型 |
| `description` | `String` | 变量描述 |
| `required` | `bool` | 是否必填 |
| `default` | `Option<Value>` | 默认值 |

## 9. CLI API

### 9.1 Cli 结构体

`Cli` 是命令行参数的根结构体。

#### 定义

```rust
#[derive(Parser)]
#[command(name = "codex")]
#[command(about = "AI 编程工具")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

### 9.2 Commands 枚举

`Commands` 表示所有可用的命令。

#### 定义

```rust
#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "索引代码库")]
    Index(IndexArgs),
    
    #[command(about = "询问问题")]
    Ask(AskArgs),
    
    #[command(about = "解释代码")]
    Explain(ExplainArgs),
    
    #[command(about = "生成代码")]
    Generate(GenerateArgs),
    
    #[command(about = "交互式聊天")]
    Chat(ChatArgs),
    
    #[command(about = "使用工具")]
    Tool(ToolArgs),
    
    #[command(about = "管理插件")]
    Plugin(PluginArgs),
    
    #[command(about = "管理任务")]
    Task(TaskArgs),
    
    #[command(about = "Solo 模式")]
    Solo(SoloArgs),
    
    #[command(about = "更新 Codex")]
    Update,
    
    #[command(about = "显示版本信息")]
    Version,
}
```

## 10. 错误类型

### 10.1 AppError 枚举

`AppError` 表示应用程序错误。

#### 定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("工具错误: {0}")]
    ToolError(#[from] ToolError),
    
    #[error("AI 错误: {0}")]
    AIError(#[from] AIError),
    
    #[error("知识库错误: {0}")]
    KnowledgeError(#[from] KnowledgeError),
    
    #[error("插件错误: {0}")]
    PluginError(#[from] PluginError),
    
    #[error("任务错误: {0}")]
    TaskError(#[from] TaskError),
    
    #[error("提示词错误: {0}")]
    PromptError(#[from] PromptError),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("其他错误: {0}")]
    Other(String),
}
```

### 10.2 其他错误类型

| 错误类型 | 描述 |
|----------|------|
| `ConfigError` | 配置相关错误 |
| `ToolError` | 工具相关错误 |
| `AIError` | AI 相关错误 |
| `KnowledgeError` | 知识库相关错误 |
| `PluginError` | 插件相关错误 |
| `TaskError` | 任务相关错误 |
| `PromptError` | 提示词相关错误 |
| `HookError` | 钩子相关错误 |
| `CliError` | 命令行相关错误 |
| `UiError` | UI 相关错误 |

## 11. 示例

### 11.1 插件开发示例

```rust
use codex::plugin::Plugin;
use codex::app::App;
use codex::tools::Tool;
use codex::tools::ToolParameter;
use serde_json::Value;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "我的第一个插件"
    }
    
    fn initialize(&self, app: &App) -> Result<(), codex::plugin::PluginError> {
        // 初始化插件
        Ok(())
    }
    
    fn shutdown(&self) -> Result<(), codex::plugin::PluginError> {
        // 关闭插件
        Ok(())
    }
    
    fn get_hooks(&self) -> Vec<Box<dyn codex::plugin::Hook>> {
        // 返回插件钩子
        Vec::new()
    }
    
    fn get_commands(&self) -> Vec<Box<dyn codex::commands::Command>> {
        // 返回插件命令
        Vec::new()
    }
    
    fn get_tools(&self) -> Vec<Box<dyn Tool>> {
        // 返回插件工具
        Vec::new()
    }
}

#[no_mangle]
pub fn get_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin)
}
```

### 11.2 工具开发示例

```rust
use codex::tools::{Tool, ToolCategory, ToolParameter, ToolError};
use serde_json::Value;

struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }
    
    fn description(&self) -> &str {
        "我的第一个工具"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Builtin
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "input".to_string(),
                r#type: "string".to_string(),
                required: true,
                description: "输入参数".to_string(),
                default: None,
            },
        ]
    }
    
    fn execute(&self, params: &Value) -> Result<Value, ToolError> {
        let input = params.get("input")
            .ok_or_else(|| ToolError::MissingParameter("input".to_string()))?
            .as_str()
            .ok_or_else(|| ToolError::InvalidParameterType("input".to_string(), "string".to_string()))?;
        
        // 工具逻辑
        Ok(Value::String(format!("Hello, {}", input)))
    }
}
```

## 12. 联系方式

如果你在使用 Codex API 过程中遇到问题，可以通过以下方式获取帮助：

- GitHub Issues: https://github.com/lyxamour/codex/issues
- 社区讨论: https://github.com/lyxamour/codex/discussions
- 邮件: contact@lyxamour.com

## 13. 许可证

Codex 使用 MIT 许可证，详细信息请查看 LICENSE 文件。
