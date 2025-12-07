//! 代码索引器
//!
//! 索引本地代码文件，构建代码知识库

use crate::config::app::KnowledgeConfig;
use crate::error::AppResult;
use crate::knowledge::base::CodeFile;
use crate::parsers::{initialize_parsers, CodeElement, CodeElementType, PARSER_REGISTRY};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// 索引缓存项，存储单个文件的索引信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexCacheItem {
    /// 文件路径
    pub path: PathBuf,
    /// 文件修改时间
    pub modified_at: SystemTime,
    /// 文件大小
    pub size: u64,
    /// 代码元素数量
    pub element_count: usize,
    /// 代码元素
    pub elements: Vec<CodeElement>,
}

/// 索引缓存，存储所有已索引文件的信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexCache {
    /// 缓存版本
    pub version: u32,
    /// 缓存创建时间
    pub created_at: SystemTime,
    /// 缓存更新时间
    pub updated_at: SystemTime,
    /// 索引文件总数
    pub file_count: usize,
    /// 索引代码元素总数
    pub total_elements: usize,
    /// 文件缓存映射
    pub files: HashMap<String, IndexCacheItem>,
}

impl Default for IndexCache {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            version: 1,
            created_at: now,
            updated_at: now,
            file_count: 0,
            total_elements: 0,
            files: HashMap::new(),
        }
    }
}

/// 代码索引器
pub struct CodeIndexer {
    /// 知识库配置
    config: KnowledgeConfig,
    /// 索引器是否已初始化
    initialized: bool,
    /// 是否已初始化解析器
    parsers_initialized: bool,
    /// 索引缓存
    cache: IndexCache,
    /// 缓存是否已加载
    cache_loaded: bool,
}

impl CodeIndexer {
    /// 创建新的代码索引器实例
    pub fn new(config: KnowledgeConfig) -> AppResult<Self> {
        Ok(Self {
            config,
            initialized: false,
            parsers_initialized: false,
            cache: IndexCache::default(),
            cache_loaded: false,
        })
    }

    /// 获取缓存文件路径
    fn get_cache_path(&self) -> PathBuf {
        self.config.metadata_dir.join("index_cache.json")
    }

    /// 保存索引缓存到文件
    pub fn save_cache(&self) -> AppResult<()> {
        let cache_path = self.get_cache_path();
        let cache_json = serde_json::to_string_pretty(&self.cache)?;
        let mut file = File::create(cache_path)?;
        file.write_all(cache_json.as_bytes())?;
        Ok(())
    }

    /// 获取索引的文件数量
    pub fn file_count(&self) -> usize {
        self.cache.file_count
    }

    /// 获取索引的代码元素数量
    pub fn total_elements(&self) -> usize {
        self.cache.total_elements
    }

    /// 从文件加载索引缓存
    pub fn load_cache(&mut self) -> AppResult<()> {
        let cache_path = self.get_cache_path();
        if cache_path.exists() {
            let mut file = File::open(cache_path)?;
            let mut cache_json = String::new();
            file.read_to_string(&mut cache_json)?;
            self.cache = serde_json::from_str(&cache_json)?;
            self.cache_loaded = true;
            println!(
                "加载缓存成功，包含 {} 个文件，{} 个代码元素",
                self.cache.file_count, self.cache.total_elements
            );
        } else {
            // 缓存文件不存在，使用默认缓存
            self.cache = IndexCache::default();
            self.cache_loaded = true;
            println!("未找到缓存文件，使用新缓存");
        }
        Ok(())
    }

    /// 更新索引缓存
    pub fn update_cache(&mut self, file: &CodeFile, elements: &[CodeElement]) -> AppResult<()> {
        let file_key = file.path.to_str().unwrap_or("").to_string();
        let now = SystemTime::now();

        // 创建缓存项
        let cache_item = IndexCacheItem {
            path: file.path.clone(),
            modified_at: SystemTime::now(),
            size: file.size,
            element_count: elements.len(),
            elements: elements.to_vec(),
        };

        // 移除旧缓存项（如果存在）
        if let Some(old_item) = self.cache.files.remove(&file_key) {
            self.cache.total_elements = self
                .cache
                .total_elements
                .saturating_sub(old_item.element_count);
        }

        // 添加新缓存项
        self.cache.files.insert(file_key.clone(), cache_item);

        // 更新缓存统计信息
        self.cache.file_count = self.cache.files.len();
        self.cache.total_elements += elements.len();
        self.cache.updated_at = now;

        Ok(())
    }

    /// 初始化索引器
    pub fn init(&mut self) -> AppResult<()> {
        use std::fs;

        // 创建索引目录
        fs::create_dir_all(&self.config.index_dir)?;
        fs::create_dir_all(&self.config.metadata_dir)?;

        // 初始化解析器
        self.init_parsers()?;

        // 加载索引缓存
        self.load_cache()?;

        self.initialized = true;
        Ok(())
    }

    /// 初始化解析器
    pub fn init_parsers(&mut self) -> AppResult<()> {
        if !self.parsers_initialized {
            initialize_parsers()?;
            self.parsers_initialized = true;
        }
        Ok(())
    }

    /// 索引单个文件，实现增量索引
    pub fn index_file(&mut self, file: CodeFile) -> AppResult<()> {
        // 确保索引器已初始化
        if !self.initialized {
            self.init()?;
        }

        let file_key = file.path.to_str().unwrap_or("").to_string();
        let file_metadata = std::fs::metadata(&file.path)?;
        let file_modified = file_metadata.modified()?;

        // 检查文件是否已经在缓存中，且未修改
        let needs_reindex = if let Some(cache_item) = self.cache.files.get(&file_key) {
            // 比较修改时间和大小
            cache_item.modified_at != file_modified || cache_item.size != file.size
        } else {
            // 文件不在缓存中，需要索引
            true
        };

        if !needs_reindex {
            // 文件未修改，跳过索引
            println!("文件未修改，跳过索引: {:?}", file.path);
            return Ok(());
        }

        println!("开始索引文件: {:?}", file.path);

        // 使用解析器解析代码元素
        let registry = PARSER_REGISTRY.read().unwrap();
        if let Some(parser) = registry.get_parser_by_filename(file.path.to_str().unwrap_or("")) {
            // 解析文件内容，生成代码元素
            let code_elements = parser.parse_file(file.path.to_str().unwrap(), &file.content)?;

            // TODO: 主人~ 这里需要实现将代码元素添加到tantivy索引的逻辑
            // 提示：遍历code_elements，为每个元素创建索引文档
            println!(
                "索引文件: {:?}，找到 {} 个代码元素",
                file.path,
                code_elements.len()
            );

            // 简单打印找到的元素
            for element in &code_elements {
                println!("  - {:?}: {}", element.element_type, element.name);
            }

            // 更新缓存
            self.update_cache(&file, &code_elements)?;
        } else {
            println!("警告：未找到适合文件 {:?} 的解析器", file.path);
        }

        Ok(())
    }

    /// 索引目录
    pub fn index_directory(&mut self, path: &std::path::Path) -> AppResult<()> {
        use std::fs;
        use std::io::Read;
        use walkdir::WalkDir;

        // 确保索引器已初始化
        if !self.initialized {
            self.init()?;
        }

        // 遍历目录
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // 检查文件扩展名是否在支持列表中
            if let Some(ext) = file_path.extension() {
                let ext_str = ext.to_str().unwrap_or("");
                if self
                    .config
                    .supported_extensions
                    .contains(&ext_str.to_lowercase())
                {
                    // 检查是否在排除列表中
                    let path_str = file_path.to_str().unwrap_or("");
                    let mut should_exclude = false;
                    for pattern in &self.config.exclude_patterns {
                        if path_str.contains(pattern) {
                            should_exclude = true;
                            break;
                        }
                    }

                    if !should_exclude {
                        // 读取文件内容
                        let mut file = fs::File::open(file_path)?;
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;

                        // 获取文件元数据
                        let metadata = fs::metadata(file_path)?;
                        let modified_at = metadata
                            .modified()
                            .map_err(|e| crate::error::AppError::Other(e.to_string()))?
                            .elapsed()
                            .map_err(|e| crate::error::AppError::Other(e.to_string()))?
                            .as_secs();

                        // 创建CodeFile对象
                        let code_file = CodeFile {
                            path: file_path.to_path_buf(),
                            content,
                            language: ext_str.to_lowercase(),
                            size: metadata.len(),
                            modified_at,
                        };

                        // 索引文件
                        self.index_file(code_file)?;
                    }
                }
            }
        }

        // 索引完成后保存缓存
        self.save_cache()?;
        println!("索引完成，已保存缓存");

        Ok(())
    }

    /// 搜索索引
    pub fn search(&self, query: &str) -> AppResult<Vec<crate::parsers::CodeElement>> {
        // TODO: 主人~ 这里需要实现索引搜索逻辑
        // 提示：使用tantivy搜索索引，返回匹配的代码元素
        Ok(Vec::new())
    }
}
