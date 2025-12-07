//! 多语言解析器框架
//!
//! 定义LanguageParser trait和解析器注册表，支持多种编程语言的解析

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tree_sitter::Parser as TsParser;
use tree_sitter::{Language as TsLanguage, Node as TsNode};

use crate::error::AppResult;

/// 代码元素类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeElementType {
    /// 函数
    Function,
    /// 类
    Class,
    /// 结构体
    Struct,
    /// 枚举
    Enum,
    /// 接口
    Interface,
    /// 模块
    Module,
    /// 变量
    Variable,
    /// 常量
    Constant,
    /// 类型别名
    TypeAlias,
    /// 特征/接口
    Trait,
    /// 实现
    Implementation,
    /// 宏
    Macro,
    /// 其他
    Other,
}

/// 代码元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    /// 元素类型
    pub element_type: CodeElementType,
    /// 名称
    pub name: String,
    /// 定义位置
    pub definition: SourceLocation,
    /// 文档注释
    pub documentation: Option<String>,
    /// 父元素（序列化时忽略，避免循环引用）
    #[serde(skip)]
    pub parent: Option<Arc<CodeElement>>,
    /// 子元素（序列化时忽略，避免循环引用）
    #[serde(skip)]
    pub children: Vec<Arc<CodeElement>>,
    /// 语言
    pub language: String,
}

/// 源位置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
    /// 长度
    pub length: u32,
}

/// 语言解析器 trait，定义解析器的基本接口
pub trait LanguageParser {
    /// 获取支持的语言名称
    fn language_name(&self) -> String;

    /// 获取支持的文件扩展名
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// 获取tree-sitter语言
    fn tree_sitter_language(&self) -> TsLanguage;

    /// 解析代码文件，生成代码元素树
    fn parse_file(&self, file_path: &str, content: &str) -> AppResult<Vec<CodeElement>>;

    /// 解析代码片段，生成代码元素树
    fn parse_snippet(&self, content: &str) -> AppResult<Vec<CodeElement>>;

    /// 从tree-sitter节点构建代码元素
    fn build_code_elements(
        &self,
        root_node: TsNode,
        content: &str,
        file_path: &str,
    ) -> Vec<CodeElement>;
}

/// 解析器注册表，管理所有可用的解析器
pub struct ParserRegistry {
    /// 解析器映射，从语言名称到解析器实例
    parsers: HashMap<String, Arc<dyn LanguageParser + Send + Sync>>,
    /// 扩展名到解析器的映射
    extension_map: HashMap<String, Arc<dyn LanguageParser + Send + Sync>>,
}

impl ParserRegistry {
    /// 创建新的解析器注册表
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// 注册解析器
    pub fn register(&mut self, parser: Arc<dyn LanguageParser + Send + Sync>) {
        let lang_name = parser.language_name();
        self.parsers.insert(lang_name.clone(), parser.clone());

        // 为每个支持的扩展名创建映射
        for ext in parser.supported_extensions() {
            self.extension_map.insert(ext.to_string(), parser.clone());
        }
    }

    /// 根据语言名称获取解析器
    pub fn get_parser_by_language(
        &self,
        language: &str,
    ) -> Option<Arc<dyn LanguageParser + Send + Sync>> {
        self.parsers.get(language).cloned()
    }

    /// 根据文件扩展名获取解析器
    pub fn get_parser_by_extension(
        &self,
        extension: &str,
    ) -> Option<Arc<dyn LanguageParser + Send + Sync>> {
        self.extension_map.get(extension).cloned()
    }

    /// 根据文件名获取解析器
    pub fn get_parser_by_filename(
        &self,
        filename: &str,
    ) -> Option<Arc<dyn LanguageParser + Send + Sync>> {
        if let Some(ext) = filename.rsplit('.').next() {
            self.get_parser_by_extension(ext)
        } else {
            None
        }
    }

    /// 获取所有支持的语言
    pub fn supported_languages(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }

    /// 获取所有支持的文件扩展名
    pub fn supported_extensions(&self) -> Vec<String> {
        self.extension_map.keys().cloned().collect()
    }
}

/// 全局解析器注册表实例
pub static PARSER_REGISTRY: once_cell::sync::Lazy<Arc<RwLock<ParserRegistry>>> =
    once_cell::sync::Lazy::new(|| {
        let registry = ParserRegistry::new();
        Arc::new(RwLock::new(registry))
    });

/// 初始化解析器注册表，注册所有内置解析器
pub fn initialize_parsers() -> AppResult<()> {
    let mut registry = PARSER_REGISTRY.write().unwrap();

    // 注册Rust解析器
    let rust_parser = RustParser::new();
    registry.register(Arc::new(rust_parser));

    // 注册Python解析器
    let python_parser = PythonParser::new();
    registry.register(Arc::new(python_parser));

    // 注册JavaScript解析器
    let js_parser = JavaScriptParser::new();
    registry.register(Arc::new(js_parser));

    // 注册TypeScript解析器
    let ts_parser = TypeScriptParser::new();
    registry.register(Arc::new(ts_parser));

    Ok(())
}

/// Rust解析器实现
pub struct RustParser {
    /// tree-sitter解析器实例
    parser: TsParser,
}

impl RustParser {
    /// 创建新的Rust解析器
    pub fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Could not load Rust grammar");

        Self { parser }
    }
}

impl LanguageParser for RustParser {
    fn language_name(&self) -> String {
        "rust".to_string()
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["rs"]
    }

    fn tree_sitter_language(&self) -> TsLanguage {
        tree_sitter_rust::language()
    }

    fn parse_file(&self, file_path: &str, content: &str) -> AppResult<Vec<CodeElement>> {
        self.parse_snippet(content)
    }

    fn parse_snippet(&self, content: &str) -> AppResult<Vec<CodeElement>> {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Could not load Rust grammar");

        let tree = parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        let elements = self.build_code_elements(root_node, content, "");
        Ok(elements)
    }

    fn build_code_elements(
        &self,
        root_node: TsNode,
        content: &str,
        file_path: &str,
    ) -> Vec<CodeElement> {
        // TODO: 实现从Rust AST构建代码元素的逻辑
        // 遍历tree-sitter节点，识别函数、结构体、枚举等元素
        let mut elements = Vec::new();

        // 简单实现：只识别函数
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            if node.kind() == "function_item" {
                // 解析函数名
                let name_node = node.child_by_field_name("name").unwrap();
                let name = name_node.utf8_text(content.as_bytes()).unwrap().to_string();

                // 解析位置信息
                let start_pos = node.start_position();
                let end_pos = node.end_position();
                // 计算长度，使用安全的方式避免溢出
                let line_diff = (end_pos.row - start_pos.row) as u32;
                let column_diff = (end_pos.column - start_pos.column) as u32;
                // 长度估算：行数差 * 100（每行平均字符数） + 列数差
                let length = line_diff * 100 + column_diff;
                let location = SourceLocation {
                    file_path: file_path.to_string(),
                    line: start_pos.row as u32 + 1,
                    column: start_pos.column as u32 + 1,
                    length,
                };

                let element = CodeElement {
                    element_type: CodeElementType::Function,
                    name,
                    definition: location,
                    documentation: None,
                    parent: None,
                    children: Vec::new(),
                    language: self.language_name(),
                };

                elements.push(element);
            }
            // 递归处理子节点
            elements.extend(self.build_code_elements(node, content, file_path));
        }

        elements
    }
}

/// Python解析器实现
pub struct PythonParser {
    /// tree-sitter解析器实例
    parser: TsParser,
}

impl PythonParser {
    /// 创建新的Python解析器
    pub fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_python::language())
            .expect("Could not load Python grammar");

        Self { parser }
    }
}

impl LanguageParser for PythonParser {
    fn language_name(&self) -> String {
        "python".to_string()
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["py", "pyw"]
    }

    fn tree_sitter_language(&self) -> TsLanguage {
        tree_sitter_python::language()
    }

    fn parse_file(&self, file_path: &str, content: &str) -> AppResult<Vec<CodeElement>> {
        self.parse_snippet(content)
    }

    fn parse_snippet(&self, content: &str) -> AppResult<Vec<CodeElement>> {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_python::language())
            .expect("Could not load Python grammar");

        let tree = parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        let elements = self.build_code_elements(root_node, content, "");
        Ok(elements)
    }

    fn build_code_elements(
        &self,
        root_node: TsNode,
        content: &str,
        file_path: &str,
    ) -> Vec<CodeElement> {
        // TODO: 实现从Python AST构建代码元素的逻辑
        let mut elements = Vec::new();

        // 简单实现：只识别函数定义
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            if node.kind() == "function_definition" {
                // 解析函数名
                let name_node = node.child_by_field_name("name").unwrap();
                let name = name_node.utf8_text(content.as_bytes()).unwrap().to_string();

                // 解析位置信息
                let start_pos = node.start_position();
                let end_pos = node.end_position();
                // 使用安全的方式计算长度，避免溢出
                let line_diff = (end_pos.row.saturating_sub(start_pos.row)) as u32;
                let column_diff = (end_pos.column.saturating_sub(start_pos.column)) as u32;
                let length = line_diff.saturating_mul(100).saturating_add(column_diff);
                let location = SourceLocation {
                    file_path: file_path.to_string(),
                    line: start_pos.row as u32 + 1,
                    column: start_pos.column as u32 + 1,
                    length,
                };

                let element = CodeElement {
                    element_type: CodeElementType::Function,
                    name,
                    definition: location,
                    documentation: None,
                    parent: None,
                    children: Vec::new(),
                    language: self.language_name(),
                };

                elements.push(element);
            }
            // 递归处理子节点
            elements.extend(self.build_code_elements(node, content, file_path));
        }

        elements
    }
}

/// JavaScript解析器实现
pub struct JavaScriptParser {
    /// tree-sitter解析器实例
    parser: TsParser,
}

impl JavaScriptParser {
    /// 创建新的JavaScript解析器
    pub fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_javascript::language())
            .expect("Could not load JavaScript grammar");

        Self { parser }
    }
}

impl LanguageParser for JavaScriptParser {
    fn language_name(&self) -> String {
        "javascript".to_string()
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["js"]
    }

    fn tree_sitter_language(&self) -> TsLanguage {
        tree_sitter_javascript::language()
    }

    fn parse_file(&self, file_path: &str, content: &str) -> AppResult<Vec<CodeElement>> {
        self.parse_snippet(content)
    }

    fn parse_snippet(&self, content: &str) -> AppResult<Vec<CodeElement>> {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_javascript::language())
            .expect("Could not load JavaScript grammar");

        let tree = parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        let elements = self.build_code_elements(root_node, content, "");
        Ok(elements)
    }

    fn build_code_elements(
        &self,
        root_node: TsNode,
        content: &str,
        file_path: &str,
    ) -> Vec<CodeElement> {
        // TODO: 实现从JavaScript AST构建代码元素的逻辑
        let mut elements = Vec::new();

        // 打印根节点信息，用于调试
        println!(
            "根节点: {}, 子节点数量: {}",
            root_node.kind(),
            root_node.child_count()
        );

        // 简单实现：识别函数定义和赋值给变量的箭头函数
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            println!(
                "处理节点: {}, 子节点数量: {}",
                node.kind(),
                node.child_count()
            );
            match node.kind() {
                "program" => {
                    // 处理program节点，递归处理其子节点
                    elements.extend(self.build_code_elements(node, content, file_path));
                }
                "function_declaration" => {
                    // 解析命名函数
                    println!("发现函数声明节点");
                    let name = if let Some(name_node) = node.child_by_field_name("name") {
                        name_node.utf8_text(content.as_bytes()).unwrap().to_string()
                    } else {
                        "anonymous".to_string()
                    };

                    // 解析位置信息
                    let start_pos = node.start_position();
                    let end_pos = node.end_position();
                    // 使用安全的方式计算长度，避免溢出
                    let line_diff = (end_pos.row.saturating_sub(start_pos.row)) as u32;
                    let column_diff = (end_pos.column.saturating_sub(start_pos.column)) as u32;
                    let length = line_diff.saturating_mul(100).saturating_add(column_diff);
                    let location = SourceLocation {
                        file_path: file_path.to_string(),
                        line: start_pos.row as u32 + 1,
                        column: start_pos.column as u32 + 1,
                        length,
                    };

                    let element = CodeElement {
                        element_type: CodeElementType::Function,
                        name,
                        definition: location,
                        documentation: None,
                        parent: None,
                        children: Vec::new(),
                        language: self.language_name(),
                    };

                    elements.push(element);
                }
                "lexical_declaration" => {
                    // 处理变量声明，可能包含箭头函数赋值
                    println!("发现词法声明");
                    let mut var_cursor = node.walk();
                    for var_node in node.children(&mut var_cursor) {
                        println!("词法声明子节点: {}", var_node.kind());
                        if var_node.kind() == "variable_declarator" {
                            // 解析变量名
                            if let Some(name_node) = var_node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(content.as_bytes()).unwrap().to_string();
                                println!("变量名: {}", name);

                                // 解析赋值表达式的右侧箭头函数
                                if let Some(value_node) = var_node.child_by_field_name("value") {
                                    println!("变量值节点类型: {}", value_node.kind());
                                    if value_node.kind() == "arrow_function" {
                                        println!("发现箭头函数赋值: {}", name);
                                        // 解析位置信息
                                        let start_pos = node.start_position();
                                        let end_pos = node.end_position();
                                        // 使用安全的方式计算长度，避免溢出
                                        let line_diff =
                                            (end_pos.row.saturating_sub(start_pos.row)) as u32;
                                        let column_diff =
                                            (end_pos.column.saturating_sub(start_pos.column))
                                                as u32;
                                        let length = line_diff
                                            .saturating_mul(100)
                                            .saturating_add(column_diff);
                                        let location = SourceLocation {
                                            file_path: file_path.to_string(),
                                            line: start_pos.row as u32 + 1,
                                            column: start_pos.column as u32 + 1,
                                            length,
                                        };

                                        let element = CodeElement {
                                            element_type: CodeElementType::Function,
                                            name,
                                            definition: location,
                                            documentation: None,
                                            parent: None,
                                            children: Vec::new(),
                                            language: self.language_name(),
                                        };

                                        elements.push(element);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // 递归处理其他节点
                    elements.extend(self.build_code_elements(node, content, file_path));
                }
            }
        }

        elements
    }
}

/// TypeScript解析器实现
pub struct TypeScriptParser {
    /// tree-sitter解析器实例
    parser: TsParser,
}

impl TypeScriptParser {
    /// 创建新的TypeScript解析器
    pub fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_typescript::language_typescript())
            .expect("Could not load TypeScript grammar");

        Self { parser }
    }
}

impl LanguageParser for TypeScriptParser {
    fn language_name(&self) -> String {
        "typescript".to_string()
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx"]
    }

    fn tree_sitter_language(&self) -> TsLanguage {
        tree_sitter_typescript::language_typescript()
    }

    fn parse_file(&self, file_path: &str, content: &str) -> AppResult<Vec<CodeElement>> {
        self.parse_snippet(content)
    }

    fn parse_snippet(&self, content: &str) -> AppResult<Vec<CodeElement>> {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_typescript::language_typescript())
            .expect("Could not load TypeScript grammar");

        let tree = parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        let elements = self.build_code_elements(root_node, content, "");
        Ok(elements)
    }

    fn build_code_elements(
        &self,
        root_node: TsNode,
        content: &str,
        file_path: &str,
    ) -> Vec<CodeElement> {
        // TODO: 实现从TypeScript AST构建代码元素的逻辑
        let mut elements = Vec::new();

        // 简单实现：识别函数和接口定义
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            match node.kind() {
                "function_declaration" => {
                    // 解析命名函数
                    let name = if let Some(name_node) = node.child_by_field_name("name") {
                        name_node.utf8_text(content.as_bytes()).unwrap().to_string()
                    } else {
                        "anonymous".to_string()
                    };

                    // 解析位置信息
                    let start_pos = node.start_position();
                    let end_pos = node.end_position();
                    // 使用安全的方式计算长度，避免溢出
                    let line_diff = (end_pos.row.saturating_sub(start_pos.row)) as u32;
                    let column_diff = (end_pos.column.saturating_sub(start_pos.column)) as u32;
                    let length = line_diff.saturating_mul(100).saturating_add(column_diff);
                    let location = SourceLocation {
                        file_path: file_path.to_string(),
                        line: start_pos.row as u32 + 1,
                        column: start_pos.column as u32 + 1,
                        length,
                    };

                    let element = CodeElement {
                        element_type: CodeElementType::Function,
                        name,
                        definition: location,
                        documentation: None,
                        parent: None,
                        children: Vec::new(),
                        language: self.language_name(),
                    };

                    elements.push(element);
                }
                "lexical_declaration" => {
                    // 处理变量声明，可能包含箭头函数赋值
                    let mut var_cursor = node.walk();
                    for var_node in node.children(&mut var_cursor) {
                        if var_node.kind() == "variable_declarator" {
                            // 解析变量名
                            if let Some(name_node) = var_node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(content.as_bytes()).unwrap().to_string();

                                // 解析赋值表达式的右侧箭头函数
                                if let Some(value_node) = var_node.child_by_field_name("value") {
                                    if value_node.kind() == "arrow_function" {
                                        // 解析位置信息
                                        let start_pos = node.start_position();
                                        let end_pos = node.end_position();
                                        // 使用安全的方式计算长度，避免溢出
                                        let line_diff =
                                            (end_pos.row.saturating_sub(start_pos.row)) as u32;
                                        let column_diff =
                                            (end_pos.column.saturating_sub(start_pos.column))
                                                as u32;
                                        let length = line_diff
                                            .saturating_mul(100)
                                            .saturating_add(column_diff);
                                        let location = SourceLocation {
                                            file_path: file_path.to_string(),
                                            line: start_pos.row as u32 + 1,
                                            column: start_pos.column as u32 + 1,
                                            length,
                                        };

                                        let element = CodeElement {
                                            element_type: CodeElementType::Function,
                                            name,
                                            definition: location,
                                            documentation: None,
                                            parent: None,
                                            children: Vec::new(),
                                            language: self.language_name(),
                                        };

                                        elements.push(element);
                                    }
                                }
                            }
                        }
                    }
                }
                "interface_declaration" => {
                    // 解析接口名
                    let name = if let Some(name_node) = node.child_by_field_name("name") {
                        name_node.utf8_text(content.as_bytes()).unwrap().to_string()
                    } else {
                        "anonymous_interface".to_string()
                    };

                    // 解析位置信息
                    let start_pos = node.start_position();
                    let end_pos = node.end_position();
                    // 使用安全的方式计算长度，避免溢出
                    let line_diff = (end_pos.row.saturating_sub(start_pos.row)) as u32;
                    let column_diff = (end_pos.column.saturating_sub(start_pos.column)) as u32;
                    let length = line_diff.saturating_mul(100).saturating_add(column_diff);
                    let location = SourceLocation {
                        file_path: file_path.to_string(),
                        line: start_pos.row as u32 + 1,
                        column: start_pos.column as u32 + 1,
                        length,
                    };

                    let element = CodeElement {
                        element_type: CodeElementType::Interface,
                        name,
                        definition: location,
                        documentation: None,
                        parent: None,
                        children: Vec::new(),
                        language: self.language_name(),
                    };

                    elements.push(element);
                }
                _ => {}
            }
            // 递归处理子节点
            elements.extend(self.build_code_elements(node, content, file_path));
        }

        elements
    }
}
