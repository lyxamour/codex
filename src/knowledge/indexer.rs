//! 代码索引器
//! 
//! 索引本地代码文件，构建代码知识库

use crate::config::app::KnowledgeConfig;
use crate::error::AppResult;
use crate::knowledge::base::CodeFile;

/// 代码索引器
pub struct CodeIndexer {
    /// 知识库配置
    config: KnowledgeConfig,
    /// 索引器是否已初始化
    initialized: bool,
}

impl CodeIndexer {
    /// 创建新的代码索引器实例
    pub fn new(config: KnowledgeConfig) -> AppResult<Self> {
        Ok(Self {
            config,
            initialized: false,
        })
    }
    
    /// 初始化索引器
    pub fn init(&mut self) -> AppResult<()>
    {
        use std::fs;
        
        // 创建索引目录
        fs::create_dir_all(&self.config.index_dir)?;
        fs::create_dir_all(&self.config.metadata_dir)?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// 索引单个文件
    pub fn index_file(&mut self, file: CodeFile) -> AppResult<()>
    {
        // 确保索引器已初始化
        if !self.initialized {
            self.init()?;
        }
        
        // TODO: 主人~ 这里需要实现单个文件索引逻辑
        // 提示：使用tantivy索引引擎添加文件内容到索引
        // 当前暂时只记录日志，不进行实际索引
        println!("索引文件: {:?}", file.path);
        Ok(())
    }
    
    /// 索引目录
    pub fn index_directory(&mut self, path: &std::path::Path) -> AppResult<()>
    {
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
            .filter(|e| e.file_type().is_file()) {
            
            let file_path = entry.path();
            
            // 检查文件扩展名是否在支持列表中
            if let Some(ext) = file_path.extension() {
                let ext_str = ext.to_str().unwrap_or("");
                if self.config.supported_extensions.contains(&ext_str.to_lowercase()) {
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
                        let modified_at = metadata.modified().map_err(|e| crate::error::AppError::Other(e.to_string()))?.elapsed().map_err(|e| crate::error::AppError::Other(e.to_string()))?.as_secs();
                        
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
        
        Ok(())
    }
    
    /// 搜索索引
    pub fn search(&self, query: &str) -> AppResult<Vec<crate::knowledge::base::CodeElement>>
    {
        // TODO: 主人~ 这里需要实现索引搜索逻辑
        // 提示：使用tantivy搜索索引，返回匹配的代码元素
        Ok(Vec::new())
    }
}