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
    fn load_builtin_templates(&mut self) -> AppResult<()> {
        // 首先尝试从项目目录加载模板
        let project_templates_path = Path::new("templates/prompts");
        if project_templates_path.exists() {
            self.load_templates_from_dir(project_templates_path)?;
        }

        // 然后尝试从用户主目录加载模板
        if let Some(home_dir) = dirs::home_dir() {
            let user_templates_path = home_dir.join("codex").join("prompts");
            if user_templates_path.exists() {
                self.load_templates_from_dir(&user_templates_path)?;
            }
        }

        // 如果没有加载到任何模板，注册默认提示词模板作为后备
        if self.templates.is_empty() {
            self.register_default_templates()?;
        }

        Ok(())
    }

    /// 从目录加载提示词模板
    fn load_templates_from_dir(&mut self, dir_path: &Path) -> AppResult<()> {
        if !dir_path.is_dir() {
            return Ok(());
        }

        // 遍历目录中的所有YAML文件
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|ext| ext == "yaml" || ext == "yml")
                    .unwrap_or(false)
            {
                self.load_template_from_file(&path)?;
            }
        }

        Ok(())
    }

    /// 从文件加载单个提示词模板
    fn load_template_from_file(&mut self, file_path: &Path) -> AppResult<()> {
        let content = fs::read_to_string(file_path)?;
        let template: PromptTemplate = serde_yaml::from_str(&content)?;
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// 注册默认提示词模板作为后备
    fn register_default_templates(&mut self) -> AppResult<()> {
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
        self.templates
            .insert(explain_code_template.name.clone(), explain_code_template);

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
        self.templates
            .insert(generate_code_template.name.clone(), generate_code_template);

        // 注册analyze_code模板，用于代码分析
        let analyze_code_template = PromptTemplate {
            name: "analyze_code".to_string(),
            description: "分析代码的结构、功能和质量".to_string(),
            version: "1.0.0".to_string(),
            category: "analysis".to_string(),
            tags: vec!["code", "analysis", "quality"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要分析的代码".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
            ],
            template: "请详细分析以下{{language}}代码：\n\n```{{language}}\n{{code}}\n```\n\n请从以下几个方面进行分析：\n1. 代码结构和组织\n2. 主要功能和目的\n3. 代码质量和可维护性\n4. 复杂度分析\n5. 潜在问题和改进建议\n6. 代码意图和设计思路\n\n请以JSON格式输出结果，包含以下字段：\n- structure_summary: 代码结构摘要\n- function_description: 代码功能描述\n- quality_score: 质量评分(0-100)\n- complexity_score: 复杂度评分(0-100)\n- intent: 代码意图\n- dependencies: 依赖关系列表\n- suggestions: 改进建议列表\n- issues: 潜在问题列表\n- elements: 代码元素列表，每个元素包含element_type、name、description、doc、line_range、dependencies".to_string(),
            examples: Vec::new(),
        };
        self.templates
            .insert(analyze_code_template.name.clone(), analyze_code_template);

        // 注册understand_context模板，用于上下文理解
        let understand_context_template = PromptTemplate {
            name: "understand_context".to_string(),
            description: "理解代码在特定上下文中的含义".to_string(),
            version: "1.0.0".to_string(),
            category: "understanding".to_string(),
            tags: vec!["code", "context", "understanding"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要理解的代码片段".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "context".to_string(),
                    r#type: "string".to_string(),
                    description: "代码的上下文信息".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
            ],
            template: "请分析以下{{language}}代码在给定上下文中的含义和作用：\n\n上下文：\n{{context}}\n\n代码片段：\n```{{language}}\n{{code}}\n```\n\n请解释：\n1. 这段代码在上下文中的具体作用\n2. 它与周围代码的关系\n3. 它实现的核心功能\n4. 它的设计意图和思路\n5. 潜在的改进点".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(
            understand_context_template.name.clone(),
            understand_context_template,
        );

        // 注册identify_intent模板，用于代码意图识别
        let identify_intent_template = PromptTemplate {
            name: "identify_intent".to_string(),
            description: "识别代码的意图和目的".to_string(),
            version: "1.0.0".to_string(),
            category: "understanding".to_string(),
            tags: vec!["code", "intent", "understanding"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要识别意图的代码".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
            ],
            template: "请识别以下{{language}}代码的主要意图和目的：\n\n```{{language}}\n{{code}}\n```\n\n请简明扼要地描述这段代码的核心意图，重点关注它要解决的问题和实现的功能。".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(
            identify_intent_template.name.clone(),
            identify_intent_template,
        );

        // 注册evaluate_quality模板，用于代码质量评估
        let evaluate_quality_template = PromptTemplate {
            name: "evaluate_quality".to_string(),
            description: "评估代码的质量和可维护性".to_string(),
            version: "1.0.0".to_string(),
            category: "analysis".to_string(),
            tags: vec!["code", "quality", "evaluation"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要评估的代码".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
            ],
            template: "请评估以下{{language}}代码的质量和可维护性：\n\n```{{language}}\n{{code}}\n```\n\n请从以下几个方面进行评估，并给出0-100的评分：\n1. 代码清晰度和可读性\n2. 代码结构和组织\n3. 命名规范和一致性\n4. 注释质量和完整性\n5. 错误处理和边界情况\n6. 性能和效率\n7. 可测试性\n8. 可扩展性\n\n请直接返回评分数字，不要添加其他内容。".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(
            evaluate_quality_template.name.clone(),
            evaluate_quality_template,
        );

        // 注册analyze_complexity模板，用于代码复杂度分析
        let analyze_complexity_template = PromptTemplate {
            name: "analyze_complexity".to_string(),
            description: "分析代码的复杂度".to_string(),
            version: "1.0.0".to_string(),
            category: "analysis".to_string(),
            tags: vec!["code", "complexity", "analysis"].iter().map(|s| s.to_string()).collect(),
            variables: vec![
                PromptVariable {
                    name: "code".to_string(),
                    r#type: "code".to_string(),
                    description: "要分析复杂度的代码".to_string(),
                    required: true,
                },
                PromptVariable {
                    name: "language".to_string(),
                    r#type: "string".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                },
            ],
            template: "请分析以下{{language}}代码的复杂度，并给出0-100的评分：\n\n```{{language}}\n{{code}}\n```\n\n请从以下几个方面考虑复杂度：\n1. 代码行数\n2. 嵌套深度\n3. 条件分支数量\n4. 循环复杂度\n5. 函数调用深度\n6. 变量作用域复杂度\n7. 算法复杂度\n\n请直接返回评分数字，不要添加其他内容。".to_string(),
            examples: Vec::new(),
        };
        self.templates.insert(
            analyze_complexity_template.name.clone(),
            analyze_complexity_template,
        );

        Ok(())
    }

    /// 从YAML文件加载提示词模板
    pub fn load_from_file(&mut self, path: &Path) -> AppResult<()> {
        let content = fs::read_to_string(path)?;
        let template: PromptTemplate = serde_yaml::from_str(&content)?;
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// 从目录加载所有提示词模板
    pub fn load_from_directory(&mut self, dir_path: &Path) -> AppResult<()> {
        if dir_path.is_dir() {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| ext == "yaml" || ext == "yml")
                {
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
        self.templates
            .values()
            .filter(|template| template.category == category)
            .collect()
    }

    /// 按标签获取提示词模板
    pub fn list_templates_by_tag(&self, tag: &str) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|template| template.tags.contains(&tag.to_string()))
            .collect()
    }

    /// 渲染提示词模板
    pub fn render_template(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> AppResult<String> {
        let template = match self.get_template(name) {
            Some(template) => template,
            None => {
                return Err(crate::error::AppError::ai(&format!(
                    "提示词模板 {} 不存在",
                    name
                )))
            }
        };

        // 检查必填变量
        for var in &template.variables {
            if var.required && !variables.contains_key(&var.name) {
                return Err(crate::error::AppError::ai(&format!(
                    "缺少必填变量: {}",
                    var.name
                )));
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
