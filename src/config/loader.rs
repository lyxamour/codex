//! 配置加载器
//!
//! 负责从文件、环境变量加载和合并配置

use super::app::AppConfig;
use super::validator::{ConfigValidationError, ConfigValidator};
use dirs::home_dir;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// 配置加载器结果类型
pub type ConfigResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 配置加载器
pub struct ConfigLoader;

impl ConfigLoader {
    /// 创建新的配置加载器实例
    pub fn new() -> Self {
        Self
    }

    /// 加载配置
    pub fn load(&self, config_path: Option<&str>) -> ConfigResult<AppConfig> {
        // 获取基础配置
        let mut config = if let Some(path) = config_path {
            self.load_from_file(PathBuf::from(path))?
        } else {
            // 尝试从当前目录加载
            let current_config = PathBuf::from("./codex.yaml");
            if current_config.exists() {
                self.load_from_file(current_config)?
            } else {
                // 尝试从用户主目录加载
                if let Some(home) = home_dir() {
                    let home_config = home.join(".codex/config.yaml");
                    if home_config.exists() {
                        self.load_from_file(home_config)?
                    } else {
                        self.get_default_config()
                    }
                } else {
                    self.get_default_config()
                }
            }
        };

        // 从环境变量加载配置覆盖
        self.load_from_env(&mut config);

        // 验证配置
        let validator = ConfigValidator::new();
        match validator.validate(&config) {
            Ok(_) => Ok(config),
            Err(errors) => {
                // 打印验证错误
                println!("⚠️  配置验证警告:");
                for error in errors {
                    println!("   - {}", error);
                }
                // 即使验证失败，仍然返回配置，让应用程序可以使用默认值或修复配置
                Ok(config)
            }
        }
    }

    /// 从文件加载配置
    fn load_from_file(&self, path: PathBuf) -> ConfigResult<AppConfig> {
        let content = fs::read_to_string(path)?;

        // 尝试直接解析配置
        match serde_yaml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                // 如果解析失败，检查是否是因为缺少字段
                if e.to_string().contains("missing field") {
                    // 返回默认配置，让系统使用默认值
                    Ok(self.get_default_config())
                } else {
                    // 其他错误直接返回
                    Err(e.into())
                }
            }
        }
    }

    /// 保存配置到文件
    pub fn save(&self, config: &AppConfig, path: Option<&str>) -> ConfigResult<()> {
        // 确定保存路径
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => {
                if let Some(home) = home_dir() {
                    let config_dir = home.join(".codex");
                    // 确保配置目录存在
                    fs::create_dir_all(&config_dir)?;
                    config_dir.join("config.yaml")
                } else {
                    PathBuf::from("./codex.yaml")
                }
            }
        };

        // 将配置序列化为YAML
        let yaml_content = serde_yaml::to_string(config)?;

        // 写入文件
        fs::write(config_path, yaml_content)?;
        Ok(())
    }

    /// 从环境变量加载配置覆盖
    fn load_from_env(&self, config: &mut AppConfig) {
        // 获取所有环境变量
        let env_vars: HashMap<String, String> = env::vars().collect();

        // 处理应用设置的环境变量
        self.process_env_vars_for_app(&env_vars, config);

        // 处理AI配置的环境变量
        self.process_env_vars_for_ai(&env_vars, config);

        // 处理工具配置的环境变量
        self.process_env_vars_for_tools(&env_vars, config);

        // 处理UI配置的环境变量
        self.process_env_vars_for_ui(&env_vars, config);

        // 处理知识库配置的环境变量
        self.process_env_vars_for_knowledge(&env_vars, config);
    }

    /// 处理应用设置的环境变量
    fn process_env_vars_for_app(&self, env_vars: &HashMap<String, String>, config: &mut AppConfig) {
        // 应用名称
        if let Some(value) = env_vars.get("CODEX_APP_NAME") {
            config.app.name = value.to_string();
        }

        // 应用版本
        if let Some(value) = env_vars.get("CODEX_APP_VERSION") {
            config.app.version = value.to_string();
        }

        // 数据目录
        if let Some(value) = env_vars.get("CODEX_APP_DATA_DIR") {
            config.app.data_dir = PathBuf::from(value);
        }

        // 日志级别
        if let Some(value) = env_vars.get("CODEX_APP_LOG_LEVEL") {
            config.app.log_level = value.to_string();
        }

        // 界面语言
        if let Some(value) = env_vars.get("CODEX_APP_LANGUAGE") {
            config.app.language = value.to_string();
        }
    }

    /// 处理AI配置的环境变量
    fn process_env_vars_for_ai(&self, env_vars: &HashMap<String, String>, config: &mut AppConfig) {
        // 默认AI平台
        if let Some(value) = env_vars.get("CODEX_AI_DEFAULT_PLATFORM") {
            config.ai.default_platform = value.to_string();
        }

        // 确保OpenAI配置存在
        if config.ai.openai.is_none() {
            config.ai.openai = Some(super::app::OpenAIConfig::default());
        }

        if let Some(openai_config) = &mut config.ai.openai {
            // OpenAI API密钥
            if let Some(value) = env_vars.get("CODEX_AI_OPENAI_API_KEY") {
                openai_config.api_key = value.to_string();
            }

            // OpenAI默认模型
            if let Some(value) = env_vars.get("CODEX_AI_OPENAI_DEFAULT_MODEL") {
                openai_config.default_model = value.to_string();
            }

            // OpenAI API基础URL
            if let Some(value) = env_vars.get("CODEX_AI_OPENAI_BASE_URL") {
                openai_config.base_url = value.to_string();
            }
        }

        // AI缓存配置
        if let Some(value) = env_vars.get("CODEX_AI_CACHE_ENABLED") {
            if let Ok(enabled) = value.parse::<bool>() {
                config.ai.cache.enabled = enabled;
            }
        }

        if let Some(value) = env_vars.get("CODEX_AI_CACHE_DIR") {
            config.ai.cache.dir = PathBuf::from(value);
        }

        if let Some(value) = env_vars.get("CODEX_AI_CACHE_EXPIRATION") {
            if let Ok(expiration) = value.parse::<u64>() {
                config.ai.cache.expiration = expiration;
            }
        }
    }

    /// 处理工具配置的环境变量
    fn process_env_vars_for_tools(
        &self,
        env_vars: &HashMap<String, String>,
        config: &mut AppConfig,
    ) {
        // 工具定义目录
        if let Some(value) = env_vars.get("CODEX_TOOLS_TOOLS_DIR") {
            config.tools.tools_dir = PathBuf::from(value);
        }

        // 默认超时时间
        if let Some(value) = env_vars.get("CODEX_TOOLS_DEFAULT_TIMEOUT") {
            if let Ok(timeout) = value.parse::<u32>() {
                config.tools.default_timeout = timeout;
            }
        }

        // 是否启用MCP工具
        if let Some(value) = env_vars.get("CODEX_TOOLS_MCP_ENABLED") {
            if let Ok(enabled) = value.parse::<bool>() {
                config.tools.mcp_enabled = enabled;
            }
        }
    }

    /// 处理UI配置的环境变量
    fn process_env_vars_for_ui(&self, env_vars: &HashMap<String, String>, config: &mut AppConfig) {
        // 是否启用彩色输出
        if let Some(value) = env_vars.get("CODEX_UI_COLORED") {
            if let Ok(colored) = value.parse::<bool>() {
                config.ui.colored = colored;
            }
        }

        // 默认主题
        if let Some(value) = env_vars.get("CODEX_UI_THEME") {
            config.ui.theme = value.to_string();
        }

        // 是否显示动画
        if let Some(value) = env_vars.get("CODEX_UI_ANIMATIONS") {
            if let Ok(animations) = value.parse::<bool>() {
                config.ui.animations = animations;
            }
        }

        // 字体大小
        if let Some(value) = env_vars.get("CODEX_UI_FONT_SIZE") {
            if let Ok(font_size) = value.parse::<u8>() {
                config.ui.font_size = font_size;
            }
        }
    }

    /// 处理知识库配置的环境变量
    fn process_env_vars_for_knowledge(
        &self,
        env_vars: &HashMap<String, String>,
        config: &mut AppConfig,
    ) {
        // 索引目录
        if let Some(value) = env_vars.get("CODEX_KNOWLEDGE_INDEX_DIR") {
            config.knowledge.index_dir = PathBuf::from(value);
        }

        // 元数据目录
        if let Some(value) = env_vars.get("CODEX_KNOWLEDGE_METADATA_DIR") {
            config.knowledge.metadata_dir = PathBuf::from(value);
        }

        // 排除的文件模式
        if let Some(value) = env_vars.get("CODEX_KNOWLEDGE_EXCLUDE_PATTERNS") {
            config.knowledge.exclude_patterns =
                value.split(',').map(|s| s.trim().to_string()).collect();
        }

        // 支持的文件类型
        if let Some(value) = env_vars.get("CODEX_KNOWLEDGE_SUPPORTED_EXTENSIONS") {
            config.knowledge.supported_extensions =
                value.split(',').map(|s| s.trim().to_string()).collect();
        }
    }

    /// 检查是否是首次启动（配置文件不存在）
    pub fn is_first_run(&self, config_path: Option<&str>) -> bool {
        // 检查指定路径
        if let Some(path) = config_path {
            return !PathBuf::from(path).exists();
        }

        // 检查当前目录
        let current_config = PathBuf::from("./codex.yaml");
        if current_config.exists() {
            return false;
        }

        // 检查用户主目录
        if let Some(home) = home_dir() {
            let home_config = home.join(".codex/config.yaml");
            return !home_config.exists();
        }

        // 默认是首次启动
        true
    }

    /// 获取默认配置
    pub fn get_default_config(&self) -> AppConfig {
        // 设置默认配置值
        AppConfig {
            app: super::app::AppSettings {
                name: "codex".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                data_dir: home_dir().unwrap_or(PathBuf::from("")).join(".codex/data"),
                log_level: "info".to_string(),
                language: "en".to_string(),
            },
            ai: super::app::AIConfig {
                default_platform: "openai".to_string(),
                openai: Some(super::app::OpenAIConfig {
                    api_key: "".to_string(),
                    default_model: "gpt-4o".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                }),
                cache: super::app::AICacheConfig {
                    enabled: true,
                    dir: home_dir()
                        .unwrap_or(PathBuf::from("."))
                        .join(".codex/cache/ai"),
                    expiration: 3600,
                },
            },
            tools: super::app::ToolsConfig {
                tools_dir: home_dir()
                    .unwrap_or(PathBuf::from("."))
                    .join(".codex/tools"),
                default_timeout: 30,
                mcp_enabled: false,
            },
            ui: super::app::UIConfig {
                colored: true,
                theme: "default".to_string(),
                animations: true,
                font_size: 14,
            },
            knowledge: super::app::KnowledgeConfig {
                index_dir: home_dir()
                    .unwrap_or(PathBuf::from("."))
                    .join(".codex/data/knowledge/index"),
                metadata_dir: home_dir()
                    .unwrap_or(PathBuf::from("."))
                    .join(".codex/data/knowledge/metadata"),
                exclude_patterns: vec![".git", "target", "node_modules", "venv"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                supported_extensions: vec!["rs", "py", "js", "ts", "json", "yaml", "toml", "md"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        }
    }
}
