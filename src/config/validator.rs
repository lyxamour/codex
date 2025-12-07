//! 配置验证器
//!
//! 负责验证配置的有效性和完整性

use super::app::*;
use std::error::Error;
use std::fmt;

/// 配置验证错误
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    /// 缺少必填字段
    MissingField {
        /// 字段路径
        path: String,
        /// 字段描述
        description: String,
    },
    /// 无效的值
    InvalidValue {
        /// 字段路径
        path: String,
        /// 无效值
        value: String,
        /// 预期值描述
        expected: String,
    },
    /// 无效的路径
    InvalidPath {
        /// 路径字段
        path: String,
        /// 无效路径
        value: String,
    },
    /// 无效的配置组合
    InvalidCombination {
        /// 相关字段
        fields: Vec<String>,
        /// 错误描述
        description: String,
    },
}

impl Error for ConfigValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    /// 创建新的配置验证器实例
    pub fn new() -> Self {
        Self
    }

    /// 验证应用配置
    pub fn validate(&self, config: &AppConfig) -> Result<(), Vec<ConfigValidationError>> {
        let mut errors = Vec::new();

        // 验证应用设置
        self.validate_app_settings(&config.app, &mut errors);

        // 验证AI配置
        self.validate_ai_config(&config.ai, &mut errors);

        // 验证工具配置
        self.validate_tools_config(&config.tools, &mut errors);

        // 验证UI配置
        self.validate_ui_config(&config.ui, &mut errors);

        // 验证知识库配置
        self.validate_knowledge_config(&config.knowledge, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 验证应用设置
    fn validate_app_settings(
        &self,
        settings: &AppSettings,
        errors: &mut Vec<ConfigValidationError>,
    ) {
        // 验证应用名称
        if settings.name.is_empty() {
            errors.push(ConfigValidationError::MissingField {
                path: "app.name".to_string(),
                description: "应用名称不能为空".to_string(),
            });
        }

        // 验证应用版本
        if settings.version.is_empty() {
            errors.push(ConfigValidationError::MissingField {
                path: "app.version".to_string(),
                description: "应用版本不能为空".to_string(),
            });
        }

        // 验证数据目录
        if settings.data_dir.to_str().unwrap_or("").is_empty() {
            errors.push(ConfigValidationError::MissingField {
                path: "app.data_dir".to_string(),
                description: "数据目录不能为空".to_string(),
            });
        }

        // 验证日志级别
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&settings.log_level.as_str()) {
            errors.push(ConfigValidationError::InvalidValue {
                path: "app.log_level".to_string(),
                value: settings.log_level.clone(),
                expected: format!("必须是以下之一: {:?}", valid_log_levels),
            });
        }

        // 验证语言设置
        let valid_languages = ["en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "ru"];
        if !valid_languages.contains(&settings.language.as_str()) {
            errors.push(ConfigValidationError::InvalidValue {
                path: "app.language".to_string(),
                value: settings.language.clone(),
                expected: format!("必须是以下之一: {:?}", valid_languages),
            });
        }
    }

    /// 验证AI配置
    fn validate_ai_config(&self, config: &AIConfig, errors: &mut Vec<ConfigValidationError>) {
        // 验证默认AI平台
        let valid_platforms = ["openai", "claude", "gemini", "local"];
        if !valid_platforms.contains(&config.default_platform.as_str()) {
            errors.push(ConfigValidationError::InvalidValue {
                path: "ai.default_platform".to_string(),
                value: config.default_platform.clone(),
                expected: format!("必须是以下之一: {:?}", valid_platforms),
            });
        }

        // 如果默认平台是openai，但没有配置openai，则报错
        if config.default_platform == "openai" && config.openai.is_none() {
            errors.push(ConfigValidationError::MissingField {
                path: "ai.openai".to_string(),
                description: "当默认AI平台为openai时，必须配置openai部分".to_string(),
            });
        }

        // 如果配置了openai，则验证openai配置
        if let Some(openai_config) = &config.openai {
            if openai_config.api_key.is_empty() {
                errors.push(ConfigValidationError::MissingField {
                    path: "ai.openai.api_key".to_string(),
                    description: "OpenAI API密钥不能为空".to_string(),
                });
            }

            // 验证默认模型
            if openai_config.default_model.is_empty() {
                errors.push(ConfigValidationError::MissingField {
                    path: "ai.openai.default_model".to_string(),
                    description: "OpenAI默认模型不能为空".to_string(),
                });
            }

            // 验证API基础URL
            if openai_config.base_url.is_empty() {
                errors.push(ConfigValidationError::MissingField {
                    path: "ai.openai.base_url".to_string(),
                    description: "OpenAI API基础URL不能为空".to_string(),
                });
            } else if !openai_config.base_url.starts_with("http://")
                && !openai_config.base_url.starts_with("https://")
            {
                errors.push(ConfigValidationError::InvalidValue {
                    path: "ai.openai.base_url".to_string(),
                    value: openai_config.base_url.clone(),
                    expected: "必须是有效的HTTP或HTTPS URL".to_string(),
                });
            }
        }

        // 验证AI缓存配置
        if config.cache.expiration == 0 {
            errors.push(ConfigValidationError::InvalidValue {
                path: "ai.cache.expiration".to_string(),
                value: config.cache.expiration.to_string(),
                expected: "缓存过期时间必须大于0".to_string(),
            });
        }
    }

    /// 验证工具配置
    fn validate_tools_config(&self, config: &ToolsConfig, errors: &mut Vec<ConfigValidationError>) {
        // 验证默认超时时间
        if config.default_timeout == 0 {
            errors.push(ConfigValidationError::InvalidValue {
                path: "tools.default_timeout".to_string(),
                value: config.default_timeout.to_string(),
                expected: "工具默认超时时间必须大于0".to_string(),
            });
        }
    }

    /// 验证UI配置
    fn validate_ui_config(&self, config: &UIConfig, errors: &mut Vec<ConfigValidationError>) {
        // 验证主题
        let valid_themes = ["default", "dark", "light", "monokai"];
        if !valid_themes.contains(&config.theme.as_str()) {
            errors.push(ConfigValidationError::InvalidValue {
                path: "ui.theme".to_string(),
                value: config.theme.clone(),
                expected: format!("必须是以下之一: {:?}", valid_themes),
            });
        }

        // 验证字体大小
        if config.font_size < 8 || config.font_size > 32 {
            errors.push(ConfigValidationError::InvalidValue {
                path: "ui.font_size".to_string(),
                value: config.font_size.to_string(),
                expected: "必须在8到32之间".to_string(),
            });
        }
    }

    /// 验证知识库配置
    fn validate_knowledge_config(
        &self,
        config: &KnowledgeConfig,
        errors: &mut Vec<ConfigValidationError>,
    ) {
        // 验证索引目录
        if config.index_dir.to_str().unwrap_or("").is_empty() {
            errors.push(ConfigValidationError::MissingField {
                path: "knowledge.index_dir".to_string(),
                description: "知识库索引目录不能为空".to_string(),
            });
        }

        // 验证元数据目录
        if config.metadata_dir.to_str().unwrap_or("").is_empty() {
            errors.push(ConfigValidationError::MissingField {
                path: "knowledge.metadata_dir".to_string(),
                description: "知识库元数据目录不能为空".to_string(),
            });
        }
    }
}

/// 配置验证结果
pub struct ConfigValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<ConfigValidationError>,
    /// 警告列表
    pub warnings: Vec<String>,
}

impl ConfigValidationResult {
    /// 创建新的验证结果
    pub fn new(errors: Vec<ConfigValidationError>, warnings: Vec<String>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// 打印验证结果
    pub fn print(&self) {
        if self.is_valid {
            println!("✅ 配置验证通过");
        } else {
            println!("❌ 配置验证失败，发现 {} 个错误:", self.errors.len());
            for error in &self.errors {
                println!("   - {}", error);
            }
        }

        if !self.warnings.is_empty() {
            println!("⚠️  发现 {} 个警告:", self.warnings.len());
            for warning in &self.warnings {
                println!("   - {}", warning);
            }
        }
    }
}

impl fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValidationError::MissingField { path, description } => {
                write!(f, "缺少必填字段 '{}': {}", path, description)
            }
            ConfigValidationError::InvalidValue {
                path,
                value,
                expected,
            } => {
                write!(
                    f,
                    "字段 '{}' 的值 '{}' 无效，预期: {}",
                    path, value, expected
                )
            }
            ConfigValidationError::InvalidPath { path, value } => {
                write!(f, "字段 '{}' 的路径 '{}' 无效", path, value)
            }
            ConfigValidationError::InvalidCombination {
                fields,
                description,
            } => {
                write!(f, "无效的配置组合 [{}]: {}", fields.join(", "), description)
            }
        }
    }
}
