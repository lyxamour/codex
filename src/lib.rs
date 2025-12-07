//! Codex AI 编程工具核心库
//!
//! 提供本地代码索引、AI集成、工具系统和交互式UI等核心功能

// 公共模块导出
pub mod ai;
pub mod ai_response_quality;
pub mod cli;
pub mod code;
pub mod code_understanding;
pub mod command;
pub mod config;
pub mod context;
pub mod core;
pub mod docs;
pub mod error;
pub mod frameworks;
pub mod hook;
pub mod i18n;
pub mod knowledge;
pub mod parsers;
pub mod plugins;
pub mod scraper;
pub mod solo;
pub mod subagent;
pub mod task;
pub mod tools;
pub mod ui;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
