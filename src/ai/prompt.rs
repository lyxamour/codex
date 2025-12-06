//! 提示词系统模块
//! 
//! 提供提示词管理、加载、解析和变量替换功能

use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 提示词变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    /// 变量名称
    pub name: String,
    /// 变量类型
    pub r#type: String,
    /// 变量描述
    pub description: String,
    /// 是否必填
    pub required: bool,
}

/// 提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// 提示词名称
    pub name: String,
    /// 提示词描述
    pub description: String,
    /// 提示词版本
    pub version: String,
    /// 提示词类别
    pub category: String,
    /// 提示词标签
    pub tags: Vec<String>,
    /// 提示词变量
    pub variables: Vec<PromptVariable>,
    /// 提示词模板内容
    pub template: String,
    /// 示例
    pub examples: Vec<PromptExample>,
}

/// 提示词示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    /// 示例输入
    pub input: HashMap<String, String>,
    /// 示例输出
    pub output: String,
}

/// 提示词管理器
pub struct PromptManager {
    /// 提示词模板映射
    templates: HashMap<String, PromptTemplate>,
    /// 当前语言
    current_lang: String,
}

impl PromptManager {
    /// 创建新的提示词管理器实例
    pub fn new() -> AppResult<Self> {
        let mut manager = Self {
            templates: HashMap::new(),
            current_lang: "zh-CN".to_string(),
        };
        
        // 加载内置提示词模板
        manager.load_builtin_templates()?;
        
        Ok(manager)
    }
    
    /// 加载内置提示词模板
    fn load_builtin_templates(&mut self) -> AppResult<()>
    {
        // TODO: 主人~ 这里需要实现加载内置提示词模板的逻辑
        // 提示：加载templates/prompts目录下的YAML文件
        
        // 注册默认提示词模板
        let explain_code_template = PromptTemplate {
            name: "explain_code".to_string(),
            description: "解释代码的功能和实现原理".to_string(),
            version: "1.0.0".to_string(),
            category: "explanation".to_string(),
            tags: vec!["code", "explanation", "learning"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要解释的代码".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: false,
                },
            ],
            template: "请详细解释以下{{language}}代码的功能和实现原理：\n\n```{{language}}\n{{code}}\n```\n\n请从以下几个方面进行解释：\n1. 代码的整体功能和目的\n2. 关键算法和数据结构\n3. 代码的设计模式和架构\n4. 可能的改进建议\n5. 潜在的问题和注意事项\n\n请使用清晰易懂的语言，适合初学者理解。".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(explain_code_template.name.clone(), explain_code_template);
        
        // 注册generate_code模板
        let generate_code_template = PromptTemplate {
            name: "generate_code".to_string(),
            description: "根据需求生成代码".to_string(),
            version: "1.0.0".to_string(),
            category: "generation".to_string(),
            tags: vec!["code", "generation", "implementation"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "requirement".to_string(),
                    r#type: "string".to_string(),
                    description: "功能需求描述".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "framework".to_string(),
                    r#type: "string".to_string(),
                    description: "使用的框架".to_string(),
                    required: false,
                },
            ],
            template: "请根据以下需求，使用{{language}}语言{{#framework}}和{{framework}}框架{{/framework}}生成代码：\n\n需求描述：\n{{requirement}}\n\n请遵循以下要求：\n1. 代码应该清晰、简洁、易于理解\n2. 包含必要的注释和文档\n3. 遵循最佳实践和编码规范\n4. 考虑错误处理和边界情况\n5. 提供使用示例\n\n请生成完整的、可运行的代码。".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(generate_code_template.name.clone(), generate_code_template);
        
        Ok(())
    }
    
    /// 从YAML文件加载提示词模板
    pub fn load_from_file(&mut self, path: &Path) -> AppResult<()>
    {
        let content = fs::read_to_string(path)?;
        let template: PromptTemplate = serde_yaml::from_str(&content)?;
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }
    
    /// 从目录加载所有提示词模板
    pub fn load_from_directory(&mut self, dir_path: &Path) -> AppResult<()>
    {
        if dir_path.is_dir() {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                    self.load_from_file(&path)?;
                }
            }
        }
        Ok(())
    }
    
    /// 获取提示词模板
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }
    
    /// 获取所有提示词模板
    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
    
    /// 按类别获取提示词模板
    pub fn list_templates_by_category(&self, category: &str) -> Vec<&PromptTemplate> {
        self.templates.values()
            .filter(|template| template.category == category)
            .collect()
    }
    
    /// 按标签获取提示词模板
    pub fn list_templates_by_tag(&self, tag: &str) -> Vec<&PromptTemplate> {
        self.templates.values()
            .filter(|template| template.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// 渲染提示词模板
    pub fn render_template(&self, name: &str, variables: &HashMap<String, String>) -> AppResult<String>
    {
        let template = match self.get_template(name) {
            Some(template) => template,
            None => return Err(crate::error::AppError::AI(format!("提示词模板 {} 不存在", name))),
        };
        
        // 检查必填变量
        for var in &template.variables {
            if var.required && !variables.contains_key(&var.name) {
                return Err(crate::error::AppError::AI(format!("缺少必填变量: {}", var.name)));
            }
        }
        
        // 替换变量
        let mut rendered = template.template.clone();
        
        for (name, value) in variables {
            let placeholder = format!("{{{{{}}}}}", name);
            rendered = rendered.replace(&placeholder, value);
        }
        
        // 处理条件变量（如{{#framework}}和{{/framework}}）
        // TODO: 主人~ 这里需要实现更复杂的模板渲染逻辑，支持条件变量
        
        Ok(rendered)
    }
}
