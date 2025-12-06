//! AI 集成模块
//! 
//! 提供与多种AI平台的集成和交互功能

pub mod adapter;
pub mod multilingual;
pub mod prompt;

// 导出AI客户端结构体
pub use adapter::AIClient;
