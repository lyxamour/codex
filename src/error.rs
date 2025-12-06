//! 错误处理模块
//!
//! 定义应用程序的错误类型和处理机制

use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 文件系统错误
    #[error("文件系统错误: {0}")]
    FileSystem(#[from] std::io::Error),

    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_yaml::Error),

    /// AI平台错误
    #[error("AI平台错误: {0}")]
    AI(String),

    /// 工具执行错误
    #[error("工具执行错误: {0}")]
    Tool(String),

    /// 知识库错误
    #[error("知识库错误: {0}")]
    Knowledge(String),

    /// 网页抓取错误
    #[error("网页抓取错误: {0}")]
    Scraper(String),

    /// 命令执行错误
    #[error("命令执行错误: {0}")]
    Command(String),

    /// UI错误
    #[error("UI错误: {0}")]
    UI(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),

    /// 目录遍历错误
    #[error("目录遍历错误: {0}")]
    WalkDir(#[from] walkdir::Error),

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
