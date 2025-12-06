//! 知识库基础
//! 
//! 定义知识库的基本结构和操作

use crate::error::AppResult;
use std::path::PathBuf;

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

/// 代码元素类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeElementType {
    /// 函数
    Function,
    /// 类
    Class,
    /// 模块
    Module,
    /// 变量
    Variable,
    /// 枚举
    Enum,
    /// 结构体
    Struct,
    /// 接口
    Interface,
    /// 注释
    Comment,
    /// 其他
    Other,
}

/// 代码元素
#[derive(Debug, Clone)]
pub struct CodeElement {
    /// 元素类型
    pub element_type: CodeElementType,
    /// 元素名称
    pub name: String,
    /// 元素描述
    pub description: String,
    /// 所在文件
    pub file_path: PathBuf,
    /// 起始行号
    pub start_line: u32,
    /// 结束行号
    pub end_line: u32,
    /// 父元素
    pub parent: Option<String>,
    /// 子元素
    pub children: Vec<CodeElement>,
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
    fn search(&mut self, query: &str) -> AppResult<Vec<CodeElement>>;
    
    /// 获取文件列表
    fn list_files(&self) -> AppResult<Vec<PathBuf>>;
    
    /// 清除知识库
    fn clear(&mut self) -> AppResult<()>;
}
