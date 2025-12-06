//! 配置加载器
//! 
//! 负责从文件、环境变量加载和合并配置

use super::app::AppConfig;
use dirs::home_dir;
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
        // 尝试从指定路径加载配置
        if let Some(path) = config_path {
            return self.load_from_file(PathBuf::from(path));
        }

        // 尝试从当前目录加载
        let current_config = PathBuf::from("./codex.yaml");
        if current_config.exists() {
            return self.load_from_file(current_config);
        }

        // 尝试从用户主目录加载
        if let Some(home) = home_dir() {
            let home_config = home.join(".codex/config.yaml");
            if home_config.exists() {
                return self.load_from_file(home_config);
            }
        }

        // 返回默认配置
        Ok(self.get_default_config())
    }

    /// 从文件加载配置
    fn load_from_file(&self, path: PathBuf) -> ConfigResult<AppConfig> {
        // TODO: 主人~ 这里需要实现从YAML文件读取配置的逻辑
        // 提示：使用serde_yaml::from_reader()函数读取YAML内容
        let content = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 获取默认配置
    fn get_default_config(&self) -> AppConfig {
        // 设置默认配置值
        AppConfig {
            app: super::app::AppSettings {
                name: "codex".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                data_dir: home_dir().unwrap_or(PathBuf::from(".")).join(".codex/data"),
                log_level: "info".to_string(),
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
                    dir: home_dir().unwrap_or(PathBuf::from(".")).join(".codex/cache/ai"),
                    expiration: 3600,
                },
            },
            tools: super::app::ToolsConfig {
                tools_dir: home_dir().unwrap_or(PathBuf::from(".")).join(".codex/tools"),
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
                index_dir: home_dir().unwrap_or(PathBuf::from(".")).join(".codex/data/knowledge/index"),
                metadata_dir: home_dir().unwrap_or(PathBuf::from(".")).join(".codex/data/knowledge/metadata"),
                exclude_patterns: vec![".git", "target", "node_modules", "venv"].iter().map(|s| s.to_string()).collect(),
                supported_extensions: vec!["rs", "py", "js", "ts", "json", "yaml", "toml", "md"].iter().map(|s| s.to_string()).collect(),
            },
        }
    }
}
