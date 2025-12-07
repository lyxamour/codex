//! 错误处理模块
//!
//! 定义应用程序的错误类型和处理机制

use std::any::Any;
use std::fmt::Debug;
use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 配置错误
    #[error("配置错误: {description}")]
    Config {
        /// 错误描述
        description: String,
        /// 可选的源错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 文件系统错误
    #[error("文件系统错误: {path:?} - {description}")]
    FileSystem {
        /// 文件路径
        path: Option<std::path::PathBuf>,
        /// 错误描述
        description: String,
        /// 底层IO错误
        #[source]
        source: std::io::Error,
    },

    /// 序列化/反序列化错误
    #[error("序列化错误: {path:?} - {description}")]
    Serialization {
        /// 文件路径
        path: Option<std::path::PathBuf>,
        /// 错误描述
        description: String,
        /// 底层序列化错误
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// AI平台错误
    #[error("AI平台错误: {platform} - {description}")]
    AI {
        /// AI平台名称
        platform: String,
        /// 错误描述
        description: String,
        /// HTTP状态码（如果适用）
        status_code: Option<u16>,
        /// 底层网络错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 工具执行错误
    #[error("工具执行错误: {tool_name} - {description}")]
    Tool {
        /// 工具名称
        tool_name: String,
        /// 错误描述
        description: String,
        /// 执行命令（如果适用）
        command: Option<String>,
        /// 退出码（如果适用）
        exit_code: Option<i32>,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 知识库错误
    #[error("知识库错误: {operation} - {description}")]
    Knowledge {
        /// 操作名称
        operation: String,
        /// 错误描述
        description: String,
        /// 索引路径
        index_path: Option<std::path::PathBuf>,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 网页抓取错误
    #[error("网页抓取错误: {url} - {description}")]
    Scraper {
        /// 抓取URL
        url: String,
        /// 错误描述
        description: String,
        /// HTTP状态码
        status_code: Option<u16>,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 命令执行错误
    #[error("命令执行错误: {command} - {description}")]
    Command {
        /// 执行的命令
        command: String,
        /// 错误描述
        description: String,
        /// 退出码
        exit_code: Option<i32>,
        /// 工作目录
        cwd: Option<std::path::PathBuf>,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// UI错误
    #[error("UI错误: {component} - {description}")]
    UI {
        /// UI组件名称
        component: String,
        /// 错误描述
        description: String,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 网络错误
    #[error("网络错误: {description}")]
    Network {
        /// 请求URL
        url: Option<String>,
        /// 错误描述
        description: String,
        /// HTTP状态码
        status_code: Option<u16>,
        /// 底层网络错误
        #[source]
        source: reqwest::Error,
    },

    /// 目录遍历错误
    #[error("目录遍历错误: {path:?} - {description}")]
    WalkDir {
        /// 遍历路径
        path: std::path::PathBuf,
        /// 错误描述
        description: String,
        /// 底层遍历错误
        #[source]
        source: walkdir::Error,
    },

    /// 子代理错误
    #[error("子代理错误: {agent_name} - {description}")]
    Subagent {
        /// 子代理名称
        agent_name: String,
        /// 错误描述
        description: String,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 钩子错误
    #[error("钩子错误: {hook_name} - {description}")]
    Hook {
        /// 钩子名称
        hook_name: String,
        /// 错误描述
        description: String,
        /// 事件名称
        event: Option<String>,
        /// 底层错误
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 应用程序结果类型
pub type AppResult<T> = Result<T, AppError>;

/// 从字符串创建配置错误
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Other(s.to_string())
    }
}

/// 从字符串创建配置错误
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Other(s)
    }
}

/// 为AppError添加便捷创建方法
impl AppError {
    /// 创建配置错误
    pub fn config(description: &str) -> Self {
        AppError::Config {
            description: description.to_string(),
            source: None,
        }
    }

    /// 创建AI平台错误
    pub fn ai(description: &str) -> Self {
        AppError::AI {
            platform: "unknown".to_string(),
            description: description.to_string(),
            status_code: None,
            source: None,
        }
    }

    /// 创建工具执行错误
    pub fn tool(description: &str) -> Self {
        AppError::Tool {
            tool_name: "unknown".to_string(),
            description: description.to_string(),
            command: None,
            exit_code: None,
            source: None,
        }
    }

    /// 创建命令执行错误
    pub fn command(description: &str) -> Self {
        AppError::Command {
            command: "unknown".to_string(),
            description: description.to_string(),
            exit_code: None,
            cwd: None,
            source: None,
        }
    }

    /// 创建知识库错误
    pub fn knowledge(description: &str) -> Self {
        AppError::Knowledge {
            operation: "unknown".to_string(),
            description: description.to_string(),
            index_path: None,
            source: None,
        }
    }

    /// 创建网页抓取错误
    pub fn scraper(description: &str) -> Self {
        AppError::Scraper {
            url: "unknown".to_string(),
            description: description.to_string(),
            status_code: None,
            source: None,
        }
    }

    /// 创建UI错误
    pub fn ui(description: &str) -> Self {
        AppError::UI {
            component: "unknown".to_string(),
            description: description.to_string(),
            source: None,
        }
    }
}

/// 从std::io::Error转换为AppError
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystem {
            path: None,
            description: err.to_string(),
            source: err,
        }
    }
}

/// 从serde_yaml::Error转换为AppError
impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        AppError::Serialization {
            path: None,
            description: err.to_string(),
            source: Box::new(err),
        }
    }
}

/// 从serde_json::Error转换为AppError
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization {
            path: None,
            description: err.to_string(),
            source: Box::new(err),
        }
    }
}

/// 从walkdir::Error转换为AppError
impl From<walkdir::Error> for AppError {
    fn from(err: walkdir::Error) -> Self {
        let path = err
            .path()
            .unwrap_or(&std::path::PathBuf::new())
            .to_path_buf();

        AppError::WalkDir {
            path,
            description: err.to_string(),
            source: err,
        }
    }
}

/// 从reqwest::Error转换为AppError
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        let url = err.url().map(|u| u.to_string());
        let status_code = err.status().map(|s| s.as_u16());

        AppError::Network {
            url,
            description: err.to_string(),
            status_code,
            source: err,
        }
    }
}

/// 初始化eyre错误报告系统
pub fn init_error_reporting() {
    if let Err(e) = color_eyre::install() {
        eprintln!("警告: 无法安装彩色错误报告: {}", e);
    }
}

/// 扩展Result类型，添加错误上下文
pub trait ResultExt<T, E>: Sized {
    /// 添加配置错误上下文
    fn with_config_context(self, description: &str) -> Result<T, AppError>;

    /// 添加文件系统错误上下文
    fn with_file_context(self, path: &std::path::Path) -> Result<T, AppError>;

    /// 添加AI平台错误上下文
    fn with_ai_context(self, platform: &str) -> Result<T, AppError>;
}

/// 实现ResultExt trait
impl<T, E: Into<AppError>> ResultExt<T, E> for Result<T, E> {
    fn with_config_context(self, description: &str) -> Result<T, AppError> {
        self.map_err(|e| {
            let source: AppError = e.into();
            AppError::Config {
                description: format!("{}: {}", description, source),
                source: Some(Box::new(source)),
            }
        })
    }

    fn with_file_context(self, path: &std::path::Path) -> Result<T, AppError> {
        self.map_err(|e| {
            let source: AppError = e.into();
            match source {
                AppError::FileSystem { source, .. } => AppError::FileSystem {
                    path: Some(path.to_path_buf()),
                    description: source.to_string(),
                    source,
                },
                AppError::Serialization {
                    description,
                    source,
                    ..
                } => AppError::Serialization {
                    path: Some(path.to_path_buf()),
                    description,
                    source,
                },
                _ => AppError::Other(format!("文件 '{}': {}", path.display(), source)),
            }
        })
    }

    fn with_ai_context(self, platform: &str) -> Result<T, AppError> {
        self.map_err(|e| {
            let source: AppError = e.into();
            match source {
                AppError::Network {
                    source,
                    url,
                    status_code,
                    ..
                } => AppError::AI {
                    platform: platform.to_string(),
                    description: source.to_string(),
                    status_code,
                    source: Some(Box::new(source)),
                },
                _ => AppError::AI {
                    platform: platform.to_string(),
                    description: source.to_string(),
                    status_code: None,
                    source: Some(Box::new(source)),
                },
            }
        })
    }
}
