//! AI平台适配器模块
//!
//! 提供与多种AI平台的适配和交互功能

use crate::ai::prompt::PromptManager;
use crate::error::AppResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
}

impl Default for AIClient {
    fn default() -> Self {
        Self::new().expect("Failed to create AI client")
    }
}

impl AIClient {
    /// 创建新的AI客户端实例
    pub fn new() -> AppResult<Self> {
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

        Ok(Self {
            client,
            models,
            prompt_manager,
            default_model,
        })
    }

    /// 生成代码
    pub async fn generate_code(
        &mut self,
        prompt: &str,
        language: Option<&str>,
    ) -> AppResult<String> {
        let language = language.unwrap_or("rust");

        // 准备提示词变量
        let mut variables = HashMap::new();
        variables.insert("requirement".to_string(), prompt.to_string());
        variables.insert("language".to_string(), language.to_string());

        // 渲染提示词模板
        let rendered_prompt = self
            .prompt_manager
            .render_template("generate_code", &variables)?;

        // 生成代码 - TODO: 主人~ 这里需要实现实际的AI API调用逻辑
        Ok(format!(
            "// 生成的{}代码\n// 提示词: {}\n// TODO: 实现具体的AI API调用",
            language, rendered_prompt
        ))
    }

    /// 解释代码
    pub async fn explain_code(&mut self, code: &str, language: Option<&str>) -> AppResult<String> {
        let language = language.unwrap_or("rust");

        // 准备提示词变量
        let mut variables = HashMap::new();
        variables.insert("code".to_string(), code.to_string());
        variables.insert("language".to_string(), language.to_string());

        // 渲染提示词模板
        let rendered_prompt = self
            .prompt_manager
            .render_template("explain_code", &variables)?;

        // 解释代码 - TODO: 主人~ 这里需要实现实际的AI API调用逻辑
        Ok(format!(
            "// {}代码解释\n// 提示词: {}\n// TODO: 实现具体的AI API调用",
            language, rendered_prompt
        ))
    }

    /// 生成AI响应
    pub async fn generate_response(
        &self,
        prompt: &str,
        model_name: Option<&str>,
    ) -> AppResult<AIResponse> {
        // 获取模型配置
        let model_config = self
            .models
            .get(model_name.unwrap_or(&self.default_model))
            .ok_or(crate::error::AppError::AI("未找到指定模型配置".to_string()))?;

        // 构建OpenAI API请求
        let chat_request = OpenAIChatRequest {
            model: model_config.model_name.clone(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        // 发送请求到OpenAI API
        let base_url = model_config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let response = self
            .client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", model_config.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| crate::error::AppError::AI(e.to_string()))?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_body = response
                .text()
                .await
                .map_err(|e| crate::error::AppError::AI(e.to_string()))?;
            return Err(crate::error::AppError::AI(format!(
                "OpenAI API请求失败: {}",
                error_body
            )));
        }

        // 解析响应
        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| crate::error::AppError::AI(e.to_string()))?;

        // 提取响应内容
        let choice = openai_response
            .choices
            .first()
            .ok_or(crate::error::AppError::AI(
                "OpenAI API响应中没有选择项".to_string(),
            ))?;

        // 构建AIResponse
        Ok(AIResponse {
            content: choice.message.content.clone(),
            model: openai_response.model,
            platform: model_config.platform,
            tokens_used: openai_response.usage.map(|usage| usage.total_tokens),
        })
    }
}
