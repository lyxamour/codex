//! 多语言资源管理
//!
//! 管理应用的多语言资源和翻译

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use super::loader::ResourceLoader;
use super::locale::NaturalLanguage;

/// 多语言资源管理器
pub struct I18nManager {
    /// 加载的资源
    resources: HashMap<NaturalLanguage, HashMap<String, String>>,
    /// 当前语言
    current_language: NaturalLanguage,
}

impl Default for I18nManager {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
            current_language: NaturalLanguage::English,
        }
    }
}

impl I18nManager {
    /// 创建新的多语言资源管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置当前语言
    pub fn set_language(&mut self, language: NaturalLanguage) {
        self.current_language = language;
    }

    /// 获取当前语言
    pub fn current_language(&self) -> NaturalLanguage {
        self.current_language
    }

    /// 加载语言资源
    pub fn load_resource(&mut self, language: NaturalLanguage, resource: HashMap<String, String>) {
        self.resources.insert(language, resource);
    }

    /// 从文件加载语言资源
    pub fn load_from_file(
        &mut self,
        language: NaturalLanguage,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let loader = ResourceLoader::new();
        let resource = loader.load_from_file(path)?;
        self.load_resource(language, resource);
        Ok(())
    }

    /// 从目录加载所有语言资源
    pub fn load_from_dir(&mut self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let loader = ResourceLoader::new();
        let resources = loader.load_from_dir(dir)?;

        for (language, resource) in resources {
            self.load_resource(language, resource);
        }

        Ok(())
    }

    /// 翻译文本
    pub fn translate(&self, key: &str) -> String {
        // 首先尝试使用当前语言
        if let Some(resource) = self.resources.get(&self.current_language) {
            if let Some(text) = resource.get(key) {
                return text.clone();
            }
        }

        // 如果当前语言没有找到，尝试使用英语作为后备
        if let Some(resource) = self.resources.get(&NaturalLanguage::English) {
            if let Some(text) = resource.get(key) {
                return text.clone();
            }
        }

        // 如果都没有找到，返回原键
        key.to_string()
    }

    /// 翻译文本并格式化
    pub fn translate_format(&self, key: &str, args: &[&str]) -> String {
        let text = self.translate(key);
        self.format_text(&text, args)
    }

    /// 格式化文本
    fn format_text(&self, text: &str, args: &[&str]) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        let mut arg_index = 0;

        while let Some(c) = chars.next() {
            if c == '{' {
                if let Some(next) = chars.peek() {
                    if *next == '{' {
                        // 转义的 {{，添加一个 { 到结果
                        result.push('{');
                        chars.next(); // 跳过第二个 {
                    } else {
                        // 格式化参数 {0}, {1} 等
                        let mut num_str = String::new();
                        while let Some(c) = chars.next() {
                            if c == '}' {
                                break;
                            }
                            num_str.push(c);
                        }

                        if let Ok(index) = num_str.parse::<usize>() {
                            if index < args.len() {
                                result.push_str(args[index]);
                            } else {
                                // 如果索引超出范围，保留原格式
                                result.push_str(&format!("{{{}}}", index));
                            }
                        } else {
                            // 如果不是数字，保留原格式
                            result.push_str(&format!("{{{}}}", num_str));
                        }
                    }
                } else {
                    // 单个 {，直接添加
                    result.push(c);
                }
            } else if c == '}' {
                if let Some(next) = chars.peek() {
                    if *next == '}' {
                        // 转义的 }}，添加一个 } 到结果
                        result.push('}');
                        chars.next(); // 跳过第二个 }
                    } else {
                        // 单个 }，直接添加
                        result.push(c);
                    }
                } else {
                    // 单个 }，直接添加
                    result.push(c);
                }
            } else {
                // 普通字符，直接添加
                result.push(c);
            }
        }

        result
    }

    /// 获取所有支持的语言
    pub fn supported_languages(&self) -> Vec<NaturalLanguage> {
        self.resources.keys().cloned().collect()
    }
}

/// 全局多语言资源管理器实例
pub static I18N_MANAGER: Lazy<RwLock<I18nManager>> = Lazy::new(|| {
    let mut manager = I18nManager::new();

    // 加载默认英语资源
    let default_english = HashMap::from([
        (
            "welcome".to_string(),
            "Welcome to Codex AI Programming Assistant!".to_string(),
        ),
        (
            "enter_solo_mode".to_string(),
            "Entering solo mode for AI programming...".to_string(),
        ),
        (
            "starting_task".to_string(),
            "Starting solo mode for task: {0}".to_string(),
        ),
        ("max_steps".to_string(), "Maximum steps: {0}".to_string()),
        (
            "config_welcome".to_string(),
            "This is your first time running Codex. Let's configure it together!".to_string(),
        ),
        (
            "language_selection".to_string(),
            "Select Interface Language".to_string(),
        ),
        ("ai_settings".to_string(), "AI Settings".to_string()),
        ("ui_settings".to_string(), "UI Settings".to_string()),
        (
            "config_summary".to_string(),
            "Configuration Summary".to_string(),
        ),
        (
            "config_complete".to_string(),
            "Configuration Complete!".to_string(),
        ),
    ]);

    manager.load_resource(NaturalLanguage::English, default_english);

    // 加载默认中文资源
    let default_chinese = HashMap::from([
        (
            "welcome".to_string(),
            "欢迎使用 Codex AI 编程助手!".to_string(),
        ),
        (
            "enter_solo_mode".to_string(),
            "正在进入 solo 模式进行 AI 编程...".to_string(),
        ),
        (
            "starting_task".to_string(),
            "开始 solo 模式，任务: {0}".to_string(),
        ),
        ("max_steps".to_string(), "最大步骤: {0}".to_string()),
        (
            "config_welcome".to_string(),
            "这是您第一次运行 Codex。让我们一起配置它!".to_string(),
        ),
        ("language_selection".to_string(), "选择界面语言".to_string()),
        ("ai_settings".to_string(), "AI 设置".to_string()),
        ("ui_settings".to_string(), "UI 设置".to_string()),
        ("config_summary".to_string(), "配置摘要".to_string()),
        ("config_complete".to_string(), "配置完成!".to_string()),
    ]);

    manager.load_resource(NaturalLanguage::ChineseSimplified, default_chinese);

    RwLock::new(manager)
});

/// 翻译宏，简化翻译调用
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::manager::I18N_MANAGER.read().unwrap().translate($key)
    };
    ($key:expr, $($arg:expr),*) => {
        $crate::i18n::manager::I18N_MANAGER.read().unwrap().translate_format($key, &[$($arg),*])
    };
}
