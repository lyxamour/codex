//! 应用程序核心
//! 
//! 管理应用程序的生命周期和核心功能

use crate::config::app::AppConfig;
use crate::tools::registry::ToolRegistry;
use crate::knowledge::indexer::CodeIndexer;
use crate::ai::AIClient;
use crate::error::AppResult;

/// 应用程序核心结构体
pub struct AppCore {
    /// 应用配置
    config: AppConfig,
    /// 工具注册表
    tool_registry: ToolRegistry,
    /// 代码索引器
    indexer: CodeIndexer,
    /// AI客户端
    ai_client: Option<AIClient>,
    /// 应用是否正在运行
    is_running: bool,
}

impl AppCore {
    /// 创建新的应用核心实例
    pub fn new(config: AppConfig) -> AppResult<Self> {
        // 初始化工具注册表
        let tool_registry = ToolRegistry::new()?;
        
        // 初始化代码索引器
        let indexer = CodeIndexer::new(config.knowledge.clone())?;
        
        ///// 初始化AI客户端
        let ai_client = None; // TODO: 主人~ 这里需要实现AI客户端初始化逻辑
        
        Ok(Self {
            config,
            tool_registry,
            indexer,
            ai_client,
            is_running: false,
        })
    }
    
    /// 启动应用程序
    pub fn start(&mut self) -> AppResult<()> {
        self.is_running = true;
        // TODO: 主人~ 这里需要实现应用程序启动逻辑
        // 提示：初始化日志、创建必要的目录、加载工具等
        Ok(())
    }
    
    /// 停止应用程序
    pub fn stop(&mut self) -> AppResult<()> {
        self.is_running = false;
        // TODO: 主人~ 这里需要实现应用程序停止逻辑
        // 提示：保存状态、清理资源、关闭连接等
        Ok(())
    }
    
    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    /// 获取工具注册表
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }
    
    /// 获取代码索引器
    pub fn indexer(&self) -> &CodeIndexer {
        &self.indexer
    }
    
    /// 获取AI客户端
    pub fn ai_client(&self) -> Option<&AIClient> {
        self.ai_client.as_ref()
    }
    
    /// 应用是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
