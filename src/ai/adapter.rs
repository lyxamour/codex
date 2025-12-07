//! AI平台适配器模块
//!
//! 提供与多种AI平台的适配和交互功能

use crate::ai::prompt::PromptManager;
use crate::context::{ContextItem, ContextItemType, ContextManager};
use crate::error::AppResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// AI平台类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AIPlatform {
    /// OpenAI
    OpenAI,
    /// Anthropic
    Anthropic,
    /// Google Gemini
    GoogleGemini,
    /// Mistral
    Mistral,
    /// Ollama
    Ollama,
}

/// AI平台操作枚举
#[derive(Debug, Clone, clap::Subcommand)]
pub enum ProviderActions {
    /// 列出所有可用的AI平台
    List,
    /// 切换AI平台
    Switch {
        /// AI模型名称
        model_name: String,
    },
    /// 配置AI平台
    Config {
        /// AI平台类型
        platform: String,
        /// 模型名称
        model_name: String,
        /// API密钥
        api_key: Option<String>,
        /// 基础URL
        base_url: Option<String>,
    },
}

/// AI模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    /// 平台类型
    pub platform: AIPlatform,
    /// 模型名称
    pub model_name: String,
    /// API密钥
    pub api_key: String,
    /// 基础URL
    pub base_url: Option<String>,
}

// OpenAI API 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

// OpenAI API 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChatChoice>,
    usage: Option<OpenAIChatUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatChoice {
    index: usize,
    message: OpenAIChatMessage,
    finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

// Anthropic API 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicChatRequest {
    model: String,
    messages: Vec<AnthropicChatMessage>,
    max_tokens: usize,
    temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicChatMessage {
    role: String,
    content: String,
}

// Anthropic API 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicChatResponse {
    id: String,
    r#type: String,
    model: String,
    role: String,
    content: Vec<AnthropicChatContent>,
    stop_reason: Option<String>,
    usage: AnthropicChatUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicChatContent {
    r#type: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicChatUsage {
    input_tokens: usize,
    output_tokens: usize,
}

// Ollama API 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

// Ollama API 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: OllamaChatMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<usize>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<usize>,
    eval_duration: Option<u64>,
}

/// AI响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// 响应内容
    pub content: String,
    /// 使用的模型
    pub model: String,
    /// 使用的平台
    pub platform: AIPlatform,
    /// 使用的令牌数
    pub tokens_used: Option<usize>,
}

impl AIResponse {
    /// 获取响应内容
    pub fn content(&self) -> &str {
        &self.content
    }

    /// 获取使用的模型
    pub fn model(&self) -> &str {
        &self.model
    }

    /// 获取使用的平台
    pub fn platform(&self) -> AIPlatform {
        self.platform
    }

    /// 获取使用的令牌数
    pub fn tokens_used(&self) -> Option<usize> {
        self.tokens_used
    }
}

/// AI响应缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponseCacheItem {
    /// 缓存键（提示词+模型名）
    pub cache_key: String,
    /// AI响应
    pub response: AIResponse,
    /// 创建时间
    pub created_at: SystemTime,
    /// 访问次数
    pub access_count: u32,
}

/// AI响应缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponseCacheConfig {
    /// 缓存文件路径
    pub cache_path: PathBuf,
    /// 缓存过期时间（秒）
    pub cache_ttl: u64,
    /// 最大缓存项数量
    pub max_cache_items: usize,
    /// 启用缓存
    pub enabled: bool,
}

impl Default for AIResponseCacheConfig {
    fn default() -> Self {
        Self {
            cache_path: Path::new(&dirs::home_dir().unwrap_or(PathBuf::from(".")))
                .join("codex")
                .join("data")
                .join("cache")
                .join("ai_responses.json"),
            cache_ttl: 3600 * 24, // 24小时
            max_cache_items: 1000,
            enabled: true,
        }
    }
}

/// AI响应缓存
#[derive(Clone)]
pub struct AIResponseCache {
    /// 缓存配置
    config: AIResponseCacheConfig,
    /// 缓存映射
    cache: Arc<tokio::sync::RwLock<HashMap<String, AIResponseCacheItem>>>,
    /// 上次保存时间
    last_save: Arc<tokio::sync::RwLock<SystemTime>>,
}

impl AIResponseCache {
    /// 创建新的AI响应缓存
    pub async fn new() -> AppResult<Self> {
        let config = AIResponseCacheConfig::default();
        let mut cache = HashMap::new();

        // 确保缓存目录存在
        if let Some(parent) = config.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 从文件加载缓存
        if config.cache_path.exists() {
            let mut file = File::open(&config.cache_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let cached_items: Vec<AIResponseCacheItem> = serde_json::from_str(&content)?;

            // 过滤掉过期的缓存项
            let now = SystemTime::now();
            let ttl = Duration::from_secs(config.cache_ttl);

            cache = cached_items
                .into_iter()
                .filter(|item| {
                    if let Ok(elapsed) = now.duration_since(item.created_at) {
                        elapsed < ttl
                    } else {
                        false
                    }
                })
                .map(|item| (item.cache_key.clone(), item))
                .collect();
        }

        Ok(Self {
            config,
            cache: Arc::new(tokio::sync::RwLock::new(cache)),
            last_save: Arc::new(tokio::sync::RwLock::new(SystemTime::now())),
        })
    }

    /// 生成缓存键
    fn generate_cache_key(prompt: &str, model_name: &str) -> String {
        format!("{}-{}", prompt, model_name)
    }

    /// 获取缓存项
    pub async fn get(&self, prompt: &str, model_name: &str) -> Option<AIResponse> {
        if !self.config.enabled {
            return None;
        }

        let cache_key = Self::generate_cache_key(prompt, model_name);
        let mut cache = self.cache.write().await;

        if let Some(item) = cache.get_mut(&cache_key) {
            // 更新访问次数
            item.access_count += 1;

            // 检查是否过期
            let now = SystemTime::now();
            let ttl = Duration::from_secs(self.config.cache_ttl);

            if let Ok(elapsed) = now.duration_since(item.created_at) {
                if elapsed < ttl {
                    Some(item.response.clone())
                } else {
                    // 过期，移除缓存项
                    cache.remove(&cache_key);
                    None
                }
            } else {
                // 时间异常，移除缓存项
                cache.remove(&cache_key);
                None
            }
        } else {
            None
        }
    }

    /// 设置缓存项
    pub async fn set(
        &self,
        prompt: &str,
        model_name: &str,
        response: &AIResponse,
    ) -> AppResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_key = Self::generate_cache_key(prompt, model_name);
        let now = SystemTime::now();

        let cache_item = AIResponseCacheItem {
            cache_key: cache_key.clone(),
            response: response.clone(),
            created_at: now,
            access_count: 1,
        };

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, cache_item);

        // 检查是否超出最大缓存项数量
        if cache.len() > self.config.max_cache_items {
            // 移除访问次数最少的缓存项
            let mut items: Vec<_> = cache.values().cloned().collect();
            items.sort_by_key(|item| item.access_count);

            // 移除超出的缓存项
            let excess = cache.len() - self.config.max_cache_items;
            for item in items.iter().take(excess) {
                cache.remove(&item.cache_key);
            }
        }

        // 更新上次保存时间
        let mut last_save = self.last_save.write().await;
        *last_save = now;

        // 异步保存缓存到文件
        self.save().await?;

        Ok(())
    }

    /// 保存缓存到文件
    pub async fn save(&self) -> AppResult<()> {
        let cache = self.cache.read().await;
        let items: Vec<_> = cache.values().cloned().collect();

        let cache_json = serde_json::to_string_pretty(&items)?;
        let mut file = File::create(&self.config.cache_path)?;
        file.write_all(cache_json.as_bytes())?;

        Ok(())
    }

    /// 清理过期缓存
    pub async fn cleanup(&self) -> AppResult<()> {
        let now = SystemTime::now();
        let ttl = Duration::from_secs(self.config.cache_ttl);

        let mut cache = self.cache.write().await;

        // 过滤掉过期的缓存项
        cache.retain(|_, item| {
            if let Ok(elapsed) = now.duration_since(item.created_at) {
                elapsed < ttl
            } else {
                false
            }
        });

        // 保存清理后的缓存
        self.save().await?;

        Ok(())
    }

    /// 清空缓存
    pub async fn clear(&self) -> AppResult<()> {
        let mut cache = self.cache.write().await;
        cache.clear();

        // 删除缓存文件
        if self.config.cache_path.exists() {
            fs::remove_file(&self.config.cache_path)?;
        }

        Ok(())
    }
}

/// AI平台提供trait，定义了所有AI平台必须实现的方法
#[async_trait::async_trait]
pub trait AIProvider {
    /// 获取AI平台类型
    fn get_platform(&self) -> AIPlatform;

    /// 获取模型名称
    fn get_model_name(&self) -> &str;

    /// 生成AI响应
    async fn generate_response(&self, prompt: &str) -> AppResult<AIResponse>;

    /// 生成代码
    async fn generate_code(&self, prompt: &str, language: &str) -> AppResult<String>;

    /// 解释代码
    async fn explain_code(&self, code: &str, language: &str) -> AppResult<String>;
}

/// AI平台工厂，用于创建不同AI平台的实例
#[derive(Clone)]
pub struct AIProviderFactory {
    /// HTTP客户端
    client: Arc<Client>,
}

impl AIProviderFactory {
    /// 创建新的AI平台工厂
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
        }
    }

    /// 根据模型配置创建AI平台实例
    pub fn create_provider(&self, model: &AIModel) -> AppResult<Box<dyn AIProvider + Send + Sync>> {
        match model.platform {
            AIPlatform::OpenAI => {
                // TODO: 主人~ 这里需要实现OpenAI平台的具体实现
                Ok(Box::new(OpenAIProvider {
                    client: self.client.clone(),
                    model: model.clone(),
                }))
            }
            AIPlatform::Anthropic => {
                // TODO: 主人~ 这里需要实现Anthropic平台的具体实现
                Ok(Box::new(AnthropicProvider {
                    client: self.client.clone(),
                    model: model.clone(),
                }))
            }
            AIPlatform::GoogleGemini => {
                // TODO: 主人~ 这里需要实现Google Gemini平台的具体实现
                Err(crate::error::AppError::ai("Google Gemini平台尚未实现"))
            }
            AIPlatform::Mistral => {
                // TODO: 主人~ 这里需要实现Mistral平台的具体实现
                Err(crate::error::AppError::ai("Mistral平台尚未实现"))
            }
            AIPlatform::Ollama => {
                // TODO: 主人~ 这里需要实现Ollama平台的具体实现
                Ok(Box::new(OllamaProvider {
                    client: self.client.clone(),
                    model: model.clone(),
                }))
            }
        }
    }
}

/// OpenAI平台实现
#[derive(Clone)]
pub struct OpenAIProvider {
    /// HTTP客户端
    client: Arc<Client>,
    /// 模型配置
    model: AIModel,
}

#[async_trait::async_trait]
impl AIProvider for OpenAIProvider {
    fn get_platform(&self) -> AIPlatform {
        AIPlatform::OpenAI
    }

    fn get_model_name(&self) -> &str {
        &self.model.model_name
    }

    async fn generate_response(&self, prompt: &str) -> AppResult<AIResponse> {
        // 构建OpenAI API请求
        let chat_request = OpenAIChatRequest {
            model: self.model.model_name.clone(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        // 发送请求到OpenAI API
        let base_url = self
            .model
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let response = self
            .client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", self.model.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;
            return Err(crate::error::AppError::ai(&format!(
                "OpenAI API请求失败: {}",
                error_body
            )));
        }

        // 解析响应
        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;

        // 提取响应内容
        let choice = openai_response
            .choices
            .first()
            .ok_or(crate::error::AppError::ai("OpenAI API响应中没有选择项"))?;

        // 构建AIResponse
        Ok(AIResponse {
            content: choice.message.content.clone(),
            model: openai_response.model,
            platform: self.model.platform,
            tokens_used: openai_response.usage.map(|usage| usage.total_tokens),
        })
    }

    async fn generate_code(&self, prompt: &str, language: &str) -> AppResult<String> {
        // 这里实现OpenAI的代码生成逻辑
        // TODO: 主人~ 这里需要实现具体的OpenAI代码生成逻辑
        Ok(format!(
            "// OpenAI生成的{}代码\n// 提示词: {}",
            language, prompt
        ))
    }

    async fn explain_code(&self, code: &str, language: &str) -> AppResult<String> {
        // 这里实现OpenAI的代码解释逻辑
        // TODO: 主人~ 这里需要实现具体的OpenAI代码解释逻辑
        Ok(format!(
            "// OpenAI解释的{}代码\n// 代码: {}",
            language, code
        ))
    }
}

/// Anthropic平台实现
#[derive(Clone)]
pub struct AnthropicProvider {
    /// HTTP客户端
    client: Arc<Client>,
    /// 模型配置
    model: AIModel,
}

#[async_trait::async_trait]
impl AIProvider for AnthropicProvider {
    fn get_platform(&self) -> AIPlatform {
        AIPlatform::Anthropic
    }

    fn get_model_name(&self) -> &str {
        &self.model.model_name
    }

    async fn generate_response(&self, prompt: &str) -> AppResult<AIResponse> {
        // 构建Anthropic API请求
        let chat_request = AnthropicChatRequest {
            model: self.model.model_name.clone(),
            messages: vec![AnthropicChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 4096,
            temperature: Some(0.7),
        };

        // 发送请求到Anthropic API
        let base_url = self
            .model
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());
        let response = self
            .client
            .post(format!("{}/messages", base_url))
            .header("x-api-key", &self.model.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;
            return Err(crate::error::AppError::ai(&format!(
                "Anthropic API请求失败: {}",
                error_body
            )));
        }

        // 解析响应
        let anthropic_response: AnthropicChatResponse = response
            .json()
            .await
            .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;

        // 提取响应内容
        let content = anthropic_response
            .content
            .iter()
            .map(|c| c.text.clone())
            .collect::<String>();

        // 构建AIResponse
        Ok(AIResponse {
            content,
            model: anthropic_response.model,
            platform: self.model.platform,
            tokens_used: Some(
                anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            ),
        })
    }

    async fn generate_code(&self, prompt: &str, language: &str) -> AppResult<String> {
        // 使用Claude生成代码
        let full_prompt =
            format!("Generate {language} code for the following requirement:\n{prompt}");
        let response = self.generate_response(&full_prompt).await?;
        Ok(response.content)
    }

    async fn explain_code(&self, code: &str, language: &str) -> AppResult<String> {
        // 使用Claude解释代码
        let full_prompt = format!("Explain the following {language} code:\n{code}");
        let response = self.generate_response(&full_prompt).await?;
        Ok(response.content)
    }
}

/// Ollama平台实现
#[derive(Clone)]
pub struct OllamaProvider {
    /// HTTP客户端
    client: Arc<Client>,
    /// 模型配置
    model: AIModel,
}

#[async_trait::async_trait]
impl AIProvider for OllamaProvider {
    fn get_platform(&self) -> AIPlatform {
        AIPlatform::Ollama
    }

    fn get_model_name(&self) -> &str {
        &self.model.model_name
    }

    async fn generate_response(&self, prompt: &str) -> AppResult<AIResponse> {
        // 构建Ollama API请求
        let chat_request = OllamaChatRequest {
            model: self.model.model_name.clone(),
            messages: vec![OllamaChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        // 发送请求到Ollama API（默认本地地址）
        let base_url = self
            .model
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434/api".to_string());
        let response = self
            .client
            .post(format!("{}/chat/completions", base_url))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| {
                crate::error::AppError::ai(&format!("Ollama API连接失败: {}", e.to_string()))
            })?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;
            return Err(crate::error::AppError::ai(&format!(
                "Ollama API请求失败: {}",
                error_body
            )));
        }

        // 解析响应
        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| crate::error::AppError::ai(&e.to_string()))?;

        // 计算使用的令牌数
        let tokens_used = ollama_response.prompt_eval_count.map(|prompt_count| {
            let eval_count = ollama_response.eval_count.unwrap_or(0);
            prompt_count + eval_count
        });

        // 构建AIResponse
        Ok(AIResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            platform: self.model.platform,
            tokens_used,
        })
    }

    async fn generate_code(&self, prompt: &str, language: &str) -> AppResult<String> {
        // 使用Ollama生成代码
        let full_prompt =
            format!("Generate {language} code for the following requirement:\n{prompt}");
        let response = self.generate_response(&full_prompt).await?;
        Ok(response.content)
    }

    async fn explain_code(&self, code: &str, language: &str) -> AppResult<String> {
        // 使用Ollama解释代码
        let full_prompt = format!("Explain the following {language} code:\n{code}");
        let response = self.generate_response(&full_prompt).await?;
        Ok(response.content)
    }
}

/// AI客户端
#[derive(Clone)]
pub struct AIClient {
    /// HTTP客户端
    client: Arc<Client>,
    /// 模型配置映射
    models: HashMap<String, AIModel>,
    /// 提示词管理器
    prompt_manager: Arc<PromptManager>,
    /// 默认模型
    default_model: String,
    /// AI响应缓存
    response_cache: Arc<AIResponseCache>,
    /// AI平台工厂
    provider_factory: AIProviderFactory,
    /// 当前使用的AI平台实例
    current_provider: Option<Arc<dyn AIProvider + Send + Sync>>,
    /// 上下文管理器（使用RwLock保护可变状态）
    context_manager: Arc<std::sync::RwLock<ContextManager>>,
}

impl Default for AIClient {
    fn default() -> Self {
        // 为了保持 Default trait 的同步性，我们使用阻塞方式初始化
        // 这在某些情况下可能会导致 "Cannot start a runtime from within a runtime" 错误
        // 建议在异步上下文中直接使用 AIClient::new().await
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { Self::new().await.expect("Failed to create AI client") })
    }
}

impl AIClient {
    /// 创建新的AI客户端实例
    pub async fn new() -> AppResult<Self> {
        let client = Arc::new(Client::new());
        let mut models = HashMap::new();

        // 初始化提示词管理器
        let prompt_manager = Arc::new(PromptManager::new()?);

        // 加载环境变量中的API密钥，初始化默认模型
        let default_model = "openai-gpt4o".to_string();

        // 从环境变量读取OpenAI API密钥
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            models.insert(
                default_model.clone(),
                AIModel {
                    platform: AIPlatform::OpenAI,
                    model_name: "gpt-4o".to_string(),
                    api_key,
                    base_url: Some("https://api.openai.com/v1".to_string()),
                },
            );
        }

        // 从环境变量读取Anthropic API密钥
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let anthropic_model = "claude-3-opus-20240229".to_string();
            models.insert(
                format!("anthropic-{}", anthropic_model),
                AIModel {
                    platform: AIPlatform::Anthropic,
                    model_name: anthropic_model,
                    api_key,
                    base_url: Some("https://api.anthropic.com/v1".to_string()),
                },
            );
        }

        // 添加默认的Ollama本地模型配置
        // 无需API密钥，默认使用本地Ollama服务
        let ollama_model = "llama3".to_string();
        models.insert(
            format!("ollama-{}", ollama_model),
            AIModel {
                platform: AIPlatform::Ollama,
                model_name: ollama_model,
                api_key: "".to_string(), // Ollama本地模型无需API密钥
                base_url: Some("http://localhost:11434/api".to_string()),
            },
        );

        // 创建AI响应缓存（直接在当前运行时中初始化）
        let response_cache = AIResponseCache::new().await?;

        // 创建AI平台工厂
        let provider_factory = AIProviderFactory::new();

        // 创建当前AI平台实例（如果有默认模型）
        let current_provider = if models.contains_key(&default_model) {
            let model = models.get(&default_model).unwrap();
            let provider = provider_factory.create_provider(model)?;
            Some(Arc::from(provider)) // 将Box转换为Arc
        } else {
            None
        };

        // 创建上下文管理器
        let context_manager = Arc::new(std::sync::RwLock::new(ContextManager::default()));

        Ok(Self {
            client,
            models,
            prompt_manager,
            default_model,
            response_cache: Arc::new(response_cache),
            provider_factory,
            current_provider,
            context_manager,
        })
    }

    /// 生成代码
    pub async fn generate_code(&self, prompt: &str, language: Option<&str>) -> AppResult<String> {
        let language = language.unwrap_or("rust");

        // 准备提示词变量
        let mut variables = HashMap::new();
        variables.insert("requirement".to_string(), prompt.to_string());
        variables.insert("language".to_string(), language.to_string());

        // 渲染提示词模板
        let rendered_prompt = self
            .prompt_manager
            .render_template("generate_code", &variables)?;

        // 获取当前AI平台实例
        let provider = self.get_current_provider()?;

        // 生成代码
        provider.generate_code(&rendered_prompt, language).await
    }

    /// 解释代码
    pub async fn explain_code(&self, code: &str, language: Option<&str>) -> AppResult<String> {
        let language = language.unwrap_or("rust");

        // 准备提示词变量
        let mut variables = HashMap::new();
        variables.insert("code".to_string(), code.to_string());
        variables.insert("language".to_string(), language.to_string());

        // 渲染提示词模板
        let rendered_prompt = self
            .prompt_manager
            .render_template("explain_code", &variables)?;

        // 获取当前AI平台实例
        let provider = self.get_current_provider()?;

        // 解释代码
        provider.explain_code(&rendered_prompt, language).await
    }

    /// 生成AI响应
    pub async fn generate_response(
        &self,
        prompt: &str,
        model_name: Option<&str>,
    ) -> AppResult<AIResponse> {
        let model_name = model_name.unwrap_or(&self.default_model);

        // 先检查缓存
        if let Some(cached_response) = self.response_cache.get(prompt, model_name).await {
            return Ok(cached_response);
        }

        // 获取模型配置
        let model_config = self
            .models
            .get(model_name)
            .ok_or(crate::error::AppError::ai("未找到指定模型配置"))?;

        // 创建AI平台实例
        let provider = self.provider_factory.create_provider(model_config)?;

        // 将用户提示添加到上下文
        self.context_manager
            .write()
            .expect("RwLock poisoned")
            .add_user_message(prompt);

        // 获取上下文
        let context_items = self
            .context_manager
            .read()
            .expect("RwLock poisoned")
            .get_context();

        // 构建带有上下文的完整提示
        let full_prompt = self.build_prompt_with_context(prompt, &context_items);

        // 调用AI平台生成响应
        let response = provider.generate_response(&full_prompt).await?;

        // 将AI响应添加到上下文
        self.context_manager
            .write()
            .expect("RwLock poisoned")
            .add_ai_message(&response.content);

        // 保存到缓存
        self.response_cache
            .set(prompt, model_name, &response)
            .await?;

        Ok(response)
    }

    /// 构建带有上下文的完整提示
    fn build_prompt_with_context(&self, prompt: &str, context_items: &[ContextItem]) -> String {
        let mut full_prompt = String::new();

        // 添加上下文
        full_prompt.push_str("# 上下文信息\n");
        full_prompt.push_str("以下是相关上下文信息，用于理解当前请求：\n\n");

        for item in context_items {
            let item_type: &str = item.item_type.into();
            full_prompt.push_str(&format!("## {} (重要性: {})\n", item_type, item.importance));
            full_prompt.push_str(&format!("{}\n\n", item.content));
        }

        // 添加当前请求
        full_prompt.push_str("# 当前请求\n");
        full_prompt.push_str(prompt);
        full_prompt.push_str("\n");

        full_prompt
    }

    /// 获取当前上下文
    pub fn get_context(&self) -> Vec<ContextItem> {
        self.context_manager.read().unwrap().get_context()
    }

    /// 清除上下文
    pub fn clear_context(&self) {
        self.context_manager.write().unwrap().clear();
    }

    /// 设置上下文最大令牌数
    pub fn set_context_max_tokens(&self, max_tokens: usize) {
        self.context_manager
            .write()
            .unwrap()
            .set_max_tokens(max_tokens);
    }

    /// 获取上下文统计信息
    pub fn get_context_summary(&self) -> String {
        let summary = self.context_manager.read().unwrap().get_summary();
        format!(
            "上下文项目数: {}, 总令牌数: {}, 平均重要性: {:.2}",
            summary.total_items, summary.total_tokens, summary.avg_importance
        )
    }

    /// 获取当前AI平台实例
    fn get_current_provider(&self) -> AppResult<Arc<dyn AIProvider + Send + Sync>> {
        self.current_provider
            .clone()
            .ok_or(crate::error::AppError::ai("未配置AI平台"))
    }

    /// 切换AI平台
    pub fn switch_provider(&mut self, model_name: &str) -> AppResult<()> {
        let model_config = self
            .models
            .get(model_name)
            .ok_or(crate::error::AppError::ai("未找到指定模型配置"))?;

        let provider = self.provider_factory.create_provider(model_config)?;
        self.current_provider = Some(Arc::from(provider)); // 将Box转换为Arc
        self.default_model = model_name.to_string();

        Ok(())
    }

    /// 添加AI模型配置
    pub fn add_model(&mut self, model_name: &str, model: AIModel) -> AppResult<()> {
        self.models.insert(model_name.to_string(), model);
        Ok(())
    }

    /// 移除AI模型配置
    pub fn remove_model(&mut self, model_name: &str) -> AppResult<()> {
        self.models.remove(model_name);
        // 如果移除的是当前模型，重置当前模型
        if self.default_model == model_name {
            self.current_provider = None;
            self.default_model = "".to_string();
        }
        Ok(())
    }

    /// 获取所有可用模型
    pub fn get_available_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
}
