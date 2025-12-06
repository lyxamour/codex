//! Codex AI 编程工具核心库
//! 
//! 提供本地代码索引、AI集成、工具系统和交互式UI等核心功能

// 公共模块导出
pub mod ai;
pub mod cli;
pub mod code;
pub mod command;
pub mod config;
pub mod core;
pub mod error;
pub mod hook;
pub mod knowledge;
pub mod tools;
pub mod ui;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
