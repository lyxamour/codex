//! 知识库基础
//!
//! 定义知识库的基本结构和操作

use crate::error::AppResult;
use crate::parsers::{CodeElement, CodeElementType};
use std::path::PathBuf;
use std::sync::Arc;

/// 代码文件信息
#[derive(Debug, Clone)]
pub struct CodeFile {
    /// 文件路径
    pub path: PathBuf,
    /// 文件内容
    pub content: String,
    /// 文件语言
    pub language: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间
    pub modified_at: u64,
}

/// 知识库基础接口
pub trait KnowledgeBase {
    /// 初始化知识库
    fn init(&mut self) -> AppResult<()>;

    /// 添加文件到知识库
    fn add_file(&mut self, file: CodeFile) -> AppResult<()>;

    /// 从知识库中移除文件
    fn remove_file(&mut self, path: &PathBuf) -> AppResult<()>;

    /// 搜索代码元素
    fn search(&mut self, query: &str) -> AppResult<Vec<Arc<CodeElement>>>;

    /// 获取文件列表
    fn list_files(&self) -> AppResult<Vec<PathBuf>>;

    /// 清除知识库
    fn clear(&mut self) -> AppResult<()>;
}
