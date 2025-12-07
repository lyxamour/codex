//! 工具多语言支持
//!
//! 提供工具的多语言支持和本地化

use crate::error::AppResult;

/// 工具本地化管理器
pub struct ToolLocalization {
    /// 当前语言
    current_lang: String,
    /// 本地化资源映射
    resources: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl ToolLocalization {
    /// 创建新的工具本地化管理器实例
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            current_lang: "zh-CN".to_string(),
            resources: std::collections::HashMap::new(),
        })
    }

    /// 设置当前语言
    pub fn set_lang(&mut self, lang: &str) {
        self.current_lang = lang.to_string();
    }

    /// 获取本地化字符串
    pub fn get_string(&self, key: &str) -> String {
        if let Some(lang_resources) = self.resources.get(&self.current_lang) {
            if let Some(value) = lang_resources.get(key) {
                return value.clone();
            }
        }
        key.to_string()
    }
}
