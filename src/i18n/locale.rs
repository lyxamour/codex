//! 语言和地区定义
//! 
//! 定义应用支持的语言和地区类型

use serde::{Deserialize, Serialize};

/// 支持的自然语言列表
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaturalLanguage {
    /// 简体中文
    ChineseSimplified,
    /// 繁体中文
    ChineseTraditional,
    /// 英语
    English,
    /// 日语
    Japanese,
    /// 韩语
    Korean,
    /// 法语
    French,
    /// 德语
    German,
    /// 西班牙语
    Spanish,
    /// 俄语
    Russian,
    /// 阿拉伯语
    Arabic,
    /// 葡萄牙语
    Portuguese,
}

impl NaturalLanguage {
    /// 获取语言的ISO 639-1代码
    pub fn iso_code(&self) -> &str {
        match self {
            NaturalLanguage::ChineseSimplified => "zh",
            NaturalLanguage::ChineseTraditional => "zh-TW",
            NaturalLanguage::English => "en",
            NaturalLanguage::Japanese => "ja",
            NaturalLanguage::Korean => "ko",
            NaturalLanguage::French => "fr",
            NaturalLanguage::German => "de",
            NaturalLanguage::Spanish => "es",
            NaturalLanguage::Russian => "ru",
            NaturalLanguage::Arabic => "ar",
            NaturalLanguage::Portuguese => "pt",
        }
    }
    
    /// 获取语言的显示名称
    pub fn display_name(&self) -> &str {
        match self {
            NaturalLanguage::ChineseSimplified => "简体中文",
            NaturalLanguage::ChineseTraditional => "繁体中文",
            NaturalLanguage::English => "English",
            NaturalLanguage::Japanese => "日本語",
            NaturalLanguage::Korean => "한국어",
            NaturalLanguage::French => "Français",
            NaturalLanguage::German => "Deutsch",
            NaturalLanguage::Spanish => "Español",
            NaturalLanguage::Russian => "Русский",
            NaturalLanguage::Arabic => "العربية",
            NaturalLanguage::Portuguese => "Português",
        }
    }
    
    /// 从ISO代码创建语言实例
    pub fn from_iso_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "zh" | "zh-cn" | "zh-hans" => Some(NaturalLanguage::ChineseSimplified),
            "zh-tw" | "zh-hant" => Some(NaturalLanguage::ChineseTraditional),
            "en" => Some(NaturalLanguage::English),
            "ja" => Some(NaturalLanguage::Japanese),
            "ko" => Some(NaturalLanguage::Korean),
            "fr" => Some(NaturalLanguage::French),
            "de" => Some(NaturalLanguage::German),
            "es" => Some(NaturalLanguage::Spanish),
            "ru" => Some(NaturalLanguage::Russian),
            "ar" => Some(NaturalLanguage::Arabic),
            "pt" => Some(NaturalLanguage::Portuguese),
            _ => None,
        }
    }
}

/// 支持的编程语言列表
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// Rust语言
    Rust,
    /// Python语言
    Python,
    /// JavaScript语言
    JavaScript,
    /// TypeScript语言
    TypeScript,
    /// Java语言
    Java,
    /// Go语言
    Go,
    /// C++语言
    Cpp,
    /// C语言
    C,
    /// Ruby语言
    Ruby,
    /// Swift语言
    Swift,
    /// Kotlin语言
    Kotlin,
    /// PHP语言
    Php,
    /// HTML
    Html,
    /// CSS
    Css,
    /// Markdown
    Markdown,
    /// JSON
    Json,
    /// YAML
    Yaml,
    /// TOML
    Toml,
    /// XML
    Xml,
    /// Shell脚本
    Shell,
}

impl Language {
    /// 获取语言的名称
    pub fn name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Java => "Java",
            Language::Go => "Go",
            Language::Cpp => "C++",
            Language::C => "C",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Php => "PHP",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Markdown => "Markdown",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Xml => "XML",
            Language::Shell => "Shell",
        }
    }
    
    /// 获取语言的常见文件扩展名
    pub fn extensions(&self) -> Vec<&str> {
        match self {
            Language::Rust => vec!["rs"],
            Language::Python => vec!["py"],
            Language::JavaScript => vec!["js"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::Java => vec!["java"],
            Language::Go => vec!["go"],
            Language::Cpp => vec!["cpp", "cxx", "cc"],
            Language::C => vec!["c"],
            Language::Ruby => vec!["rb"],
            Language::Swift => vec!["swift"],
            Language::Kotlin => vec!["kt"],
            Language::Php => vec!["php"],
            Language::Html => vec!["html", "htm"],
            Language::Css => vec!["css"],
            Language::Markdown => vec!["md", "markdown"],
            Language::Json => vec!["json"],
            Language::Yaml => vec!["yaml", "yml"],
            Language::Toml => vec!["toml"],
            Language::Xml => vec!["xml"],
            Language::Shell => vec!["sh", "bash", "zsh"],
        }
    }
    
    /// 从文件名或扩展名推断语言
    pub fn from_filename(filename: &str) -> Option<Self> {
        // 获取文件扩展名
        let extension = filename
            .rsplit('.')
            .next()?
            .to_lowercase();
        
        Self::from_extension(&extension)
    }
    
    /// 从扩展名推断语言
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "js" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "java" => Some(Language::Java),
            "go" => Some(Language::Go),
            "cpp" | "cxx" | "cc" => Some(Language::Cpp),
            "c" => Some(Language::C),
            "rb" => Some(Language::Ruby),
            "swift" => Some(Language::Swift),
            "kt" => Some(Language::Kotlin),
            "php" => Some(Language::Php),
            "html" | "htm" => Some(Language::Html),
            "css" => Some(Language::Css),
            "md" | "markdown" => Some(Language::Markdown),
            "json" => Some(Language::Json),
            "yaml" | "yml" => Some(Language::Yaml),
            "toml" => Some(Language::Toml),
            "xml" => Some(Language::Xml),
            "sh" | "bash" | "zsh" => Some(Language::Shell),
            _ => None,
        }
    }
}
