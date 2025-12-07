//! 应用配置定义
//!
//! 定义应用程序的配置结构和默认值

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    /// 应用基本配置
    pub app: AppSettings,
    /// AI平台配置
    pub ai: AIConfig,
    /// 工具配置
    pub tools: ToolsConfig,
    /// UI配置
    pub ui: UIConfig,
    /// 知识库配置
    pub knowledge: KnowledgeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings::default(),
            ai: AIConfig::default(),
            tools: ToolsConfig::default(),
            ui: UIConfig::default(),
            knowledge: KnowledgeConfig::default(),
        }
    }
}

/// 应用基本设置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppSettings {
    /// 应用名称
    pub name: String,
    /// 应用版本
    pub version: String,
    /// 数据目录路径
    pub data_dir: PathBuf,
    /// 日志级别
    pub log_level: String,
    /// 界面语言
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "codex".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: dirs::home_dir().unwrap_or_default().join(".codex/data"),
            log_level: "info".to_string(),
            language: "en".to_string(),
        }
    }
}

/// AI平台配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AIConfig {
    /// 默认AI平台
    pub default_platform: String,
    /// OpenAI配置
    pub openai: Option<OpenAIConfig>,
    /// 响应缓存配置
    pub cache: AICacheConfig,
}

/// OpenAI配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct OpenAIConfig {
    /// API密钥
    pub api_key: String,
    /// 默认模型
    pub default_model: String,
    /// API基础URL
    pub base_url: String,
}

/// AI响应缓存配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AICacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存目录
    pub dir: PathBuf,
    /// 缓存过期时间（秒）
    pub expiration: u64,
}

/// 工具配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ToolsConfig {
    /// 工具定义目录
    pub tools_dir: PathBuf,
    /// 默认超时时间（秒）
    pub default_timeout: u32,
    /// 是否启用MCP工具
    pub mcp_enabled: bool,
}

/// UI配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct UIConfig {
    /// 是否启用彩色输出
    pub colored: bool,
    /// 默认主题
    pub theme: String,
    /// 是否显示动画
    pub animations: bool,
    /// 字体大小
    pub font_size: u8,
}

/// 知识库配置
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct KnowledgeConfig {
    /// 索引目录
    pub index_dir: PathBuf,
    /// 元数据目录
    pub metadata_dir: PathBuf,
    /// 排除的文件模式
    pub exclude_patterns: Vec<String>,
    /// 支持的文件类型
    pub supported_extensions: Vec<String>,
    /// 远程内容抓取深度
    pub remote_depth: Option<u32>,
    /// 远程内容存储目录
    pub remote_dir: Option<PathBuf>,
}
