//! 资源加载器
//!
//! 加载和解析语言资源文件

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::locale::NaturalLanguage;

/// 资源加载器
pub struct ResourceLoader {
    // 资源加载器配置
}

impl Default for ResourceLoader {
    fn default() -> Self {
        Self {}
    }
}

impl ResourceLoader {
    /// 创建新的资源加载器
    pub fn new() -> Self {
        Self::default()
    }

    /// 从文件加载语言资源
    pub fn load_from_file(
        &self,
        path: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.parse_resource(&content)
    }

    /// 从目录加载所有语言资源
    pub fn load_from_dir(
        &self,
        dir: &str,
    ) -> Result<HashMap<NaturalLanguage, HashMap<String, String>>, Box<dyn std::error::Error>> {
        let mut result = HashMap::new();
        let path = Path::new(dir);

        // 检查目录是否存在
        if !path.exists() {
            return Ok(result);
        }

        // 遍历目录下的所有文件
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            // 只处理YAML文件
            if file_path.is_file() {
                if let Some(extension) = file_path.extension() {
                    if extension == "yaml" || extension == "yml" {
                        // 从文件名推断语言
                        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                        if let Some(language) = NaturalLanguage::from_iso_code(file_name) {
                            let resource = self.load_from_file(file_path.to_str().unwrap())?;
                            result.insert(language, resource);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// 解析资源内容
    pub fn parse_resource(
        &self,
        content: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // 目前只支持简单的键值对格式
        // TODO: 支持更复杂的YAML结构
        let mut result = HashMap::new();

        for line in content.lines() {
            // 跳过注释和空行
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 解析键值对
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                // 移除引号
                let value = value
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(&value)
                    .to_string();
                let value = value
                    .strip_prefix('\'')
                    .and_then(|s| s.strip_suffix('\''))
                    .unwrap_or(&value)
                    .to_string();

                result.insert(key, value);
            }
        }

        Ok(result)
    }
}
