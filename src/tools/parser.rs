//! 工具定义解析器
//! 
//! 解析YAML格式的工具定义文件

use crate::error::AppResult;
use serde::Deserialize;


/// 工具解析器
pub struct ToolParser;

/// YAML工具定义结构
#[derive(Debug, Deserialize)]
struct YamlToolDefinition {
    /// 工具名称
    name: String,
    /// 工具描述
    description: String,
    /// 工具版本
    version: String,
    /// 工具类别
    category: String,
    /// 工具参数
    parameters: Vec<YamlToolParameter>,
    /// 执行配置
    execution: YamlExecutionConfig,
}

/// YAML工具参数结构
#[derive(Debug, Deserialize)]
struct YamlToolParameter {
    /// 参数名称
    name: String,
    /// 参数类型
    #[serde(rename = "type")]
    r#type: String,
    /// 是否必填
    required: bool,
    /// 参数描述
    description: String,
    /// 默认值
    default: Option<String>,
}

/// YAML执行配置结构
#[derive(Debug, Deserialize)]
struct YamlExecutionConfig {
    /// 执行类型
    r#type: String,
    /// 处理函数
    handler: String,
    /// 超时时间
    timeout: u32,
}

impl ToolParser {
    /// 创建新的工具解析器实例
    pub fn new() -> Self {
        Self
    }
    
    /// 从YAML字符串解析工具定义
    pub fn parse_yaml(&self, yaml_content: &str) -> AppResult<crate::tools::registry::ToolMetadata>
    {
        // 解析YAML内容
        let yaml_def: YamlToolDefinition = serde_yaml::from_str(yaml_content)?;
        
        // 转换为ToolMetadata
        let metadata = crate::tools::registry::ToolMetadata {
            name: yaml_def.name,
            description: yaml_def.description,
            version: yaml_def.version,
            category: yaml_def.category,
            parameters: yaml_def.parameters.into_iter().map(|p| {
                crate::tools::registry::ToolParameter {
                    name: p.name,
                    r#type: p.r#type,
                    required: p.required,
                    description: p.description,
                    default: p.default,
                }
            }).collect(),
        };
        
        Ok(metadata)
    }
    
    /// 从文件解析工具定义
    pub fn parse_file(&self, path: &std::path::Path) -> AppResult<crate::tools::registry::ToolMetadata>
    {
        let content = std::fs::read_to_string(path)?;
        self.parse_yaml(&content)
    }
    
    /// 从目录加载所有工具定义
    pub fn load_from_directory(&self, dir_path: &std::path::Path) -> AppResult<Vec<crate::tools::registry::ToolMetadata>>
    {
        // TODO: 主人~ 这里需要实现从目录加载所有工具定义的逻辑
        // 提示：遍历目录中的所有YAML文件，解析每个文件的工具定义
        let mut metadata_list = Vec::new();
        
        if dir_path.is_dir() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                    let metadata = self.parse_file(&path)?;
                    metadata_list.push(metadata);
                }
            }
        }
        
        Ok(metadata_list)
    }
}
