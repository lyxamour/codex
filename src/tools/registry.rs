//! 工具注册表
//! 
//! 管理所有可用工具的注册和查询

use crate::error::AppResult;
use std::collections::HashMap;

/// 工具元数据
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具版本
    pub version: String,
    /// 工具类别
    pub category: String,
    /// 工具参数
    pub parameters: Vec<ToolParameter>,
}

/// 工具参数
#[derive(Debug, Clone)]
pub struct ToolParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub r#type: String,
    /// 是否必填
    pub required: bool,
    /// 参数描述
    pub description: String,
    /// 默认值
    pub default: Option<String>,
}

/// 工具注册表
pub struct ToolRegistry {
    /// 工具注册表映射
    tools: HashMap<String, ToolMetadata>,
}

impl ToolRegistry {
    /// 创建新的工具注册表实例
    pub fn new() -> AppResult<Self> {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // 自动注册内置工具
        registry.register_builtin_tools()?;
        
        Ok(registry)
    }
    
    /// 注册工具
    pub fn register_tool(&mut self, metadata: ToolMetadata) -> AppResult<()>
    {
        self.tools.insert(metadata.name.clone(), metadata);
        Ok(())
    }
    
    /// 自动注册内置工具
    fn register_builtin_tools(&mut self) -> AppResult<()>
    {
        // TODO: 主人~ 这里需要实现自动注册内置工具的逻辑
        // 提示：注册read_file、write_file、shell等内置工具
        
        // 注册文件操作工具
        let file_read_metadata = ToolMetadata {
            name: "read_file".to_string(),
            description: "读取文件内容".to_string(),
            version: "1.0.0".to_string(),
            category: "builtin".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    description: "文件路径".to_string(),
                    default: None,
                },
                ToolParameter {
                    name: "encoding".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                    description: "文件编码".to_string(),
                    default: Some("utf-8".to_string()),
                },
            ],
        };
        self.register_tool(file_read_metadata)?;
        
        // 注册write_file工具
        let file_write_metadata = ToolMetadata {
            name: "write_file".to_string(),
            description: "写入文件内容".to_string(),
            version: "1.0.0".to_string(),
            category: "builtin".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    description: "文件路径".to_string(),
                    default: None,
                },
                ToolParameter {
                    name: "content".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    description: "文件内容".to_string(),
                    default: None,
                },
                ToolParameter {
                    name: "append".to_string(),
                    r#type: "boolean".to_string(),
                    required: false,
                    description: "是否追加模式".to_string(),
                    default: Some("false".to_string()),
                },
            ],
        };
        self.register_tool(file_write_metadata)?;
        
        Ok(())
    }
    
    /// 从YAML文件加载工具定义
    pub fn load_from_yaml(&mut self, yaml_path: &str) -> AppResult<()>
    {
        // TODO: 主人~ 这里需要实现从YAML文件加载工具定义的逻辑
        // 提示：使用ToolParser解析YAML文件，然后注册工具
        let parser = super::parser::ToolParser::new();
        let content = std::fs::read_to_string(yaml_path)?;
        let metadata = parser.parse_yaml(&content)?;
        self.register_tool(metadata)
    }
    
    /// 获取工具元数据
    pub fn get_tool(&self, name: &str) -> Option<&ToolMetadata>
    {
        self.tools.get(name)
    }
    
    /// 获取所有工具列表
    pub fn list_tools(&self) -> Vec<&ToolMetadata>
    {
        self.tools.values().collect()
    }
    
    /// 检查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool
    {
        self.tools.contains_key(name)
    }
    
    /// 按类别获取工具列表
    pub fn list_tools_by_category(&self, category: &str) -> Vec<&ToolMetadata>
    {
        self.tools.values()
            .filter(|tool| tool.category == category)
            .collect()
    }
}
