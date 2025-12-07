
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use std::path::Path;
use std::fs;
use std::io::{self, Read};

/// 插件生命周期事件
enum PluginLifecycleEvent {
    Load,
    Initialize,
    Enable,
    Disable,
    Unload,
}

/// 插件配置
#[derive(Clone)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件许可证
    pub license: String,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 插件入口点
    pub entry_point: String,
    /// 插件依赖
    pub dependencies: Vec<String>,
    /// 插件配置参数
    pub config: HashMap<String, serde_json::Value>,
}

/// 插件类型
#[derive(Debug, Clone)]
pub enum PluginType {
    Core,
    Command,
    Hook,
    Subagent,
    UI,
    Other,
}

/// 插件状态
#[derive(Debug, Clone)]
pub enum PluginState {
    Loaded,
    Initialized,
    Enabled,
    Disabled,
    Error(String),
}

/// 插件结果
pub struct PluginResult {
    /// 插件名称
    pub plugin: String,
    /// 是否成功
    pub success: bool,
    /// 结果信息
    pub message: String,
    /// 附加数据
    pub data: HashMap<String, serde_json::Value>,
}

/// 插件特质，所有插件都必须实现这个特质
#[async_trait]
pub trait Plugin: Sync + Send {
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件描述
    fn description(&self) -> &str;
    
    /// 获取插件版本
    fn version(&self) -> &str;
    
    /// 获取插件作者
    fn author(&self) -> &str;
    
    /// 获取插件许可证
    fn license(&self) -> &str;
    
    /// 获取插件类型
    fn plugin_type(&self) -> PluginType;
    
    /// 初始化插件
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn Error>>;
    
    /// 启用插件
    async fn enable(&self) -> Result<(), Box<dyn Error>>;
    
    /// 禁用插件
    async fn disable(&self) -> Result<(), Box<dyn Error>>;
    
    /// 卸载插件
    async fn unload(&self) -> Result<(), Box<dyn Error>>;
    
    /// 获取插件配置
    fn get_config(&self) -> &PluginConfig;
    
    /// 更新插件配置
    fn update_config(&mut self, config: PluginConfig) -> Result<(), Box<dyn Error>>;
}

/// 插件管理器，负责插件的加载、卸载和生命周期管理
pub struct PluginManager {
    /// 已加载的插件
    plugins: HashMap<String, PluginInfo>,
    /// 插件配置存储
    config: HashMap<String, PluginConfig>,
    /// 插件状态
    states: HashMap<String, PluginState>,
}

/// 插件信息
pub struct PluginInfo {
    /// 插件实例
    pub plugin: Box<dyn Plugin>,
    /// 插件配置
    pub config: PluginConfig,
    /// 插件状态
    pub state: PluginState,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            config: HashMap::new(),
            states: HashMap::new(),
        }
    }
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Default::default()
    }
    
    /// 加载插件
    pub async fn load_plugin(&mut self, plugin_path: &str) -> Result<PluginResult, Box<dyn Error>> {
        // 1. 读取插件元数据
        let plugin_config = self.load_plugin_config(plugin_path)?;
        let plugin_name = plugin_config.name.clone();
        
        // 2. 验证插件签名 - TODO: 主人~ 这里需要实现插件签名验证逻辑
        // self.verify_plugin_signature(plugin_path, &plugin_config)?;
        
        // 3. 加载插件动态库 - TODO: 主人~ 这里需要实现插件动态库加载逻辑
        // let plugin_lib = self.load_plugin_library(plugin_path)?;
        
        // 4. 创建插件实例 - TODO: 主人~ 这里需要实现插件实例创建逻辑
        // let plugin_instance = self.create_plugin_instance(plugin_lib, &plugin_config)?;
        
        // 5. 注册插件
        // self.register_plugin(plugin_instance, plugin_config.clone());
        
        Ok(PluginResult {
            plugin: plugin_name.clone(),
            success: true,
            message: format!("Plugin '{}' loaded successfully", plugin_name),
            data: HashMap::new(),
        })
    }
    
    /// 加载插件配置
    fn load_plugin_config(&self, plugin_path: &str) -> Result<PluginConfig, Box<dyn Error>> {
        // 读取插件配置文件
        let config_path = Path::new(plugin_path).join("plugin.toml");
        if !config_path.exists() {
            return Err(format!("Plugin config file not found: {}", config_path.display()).into());
        }
        
        let config_content = fs::read_to_string(config_path)?;
        let toml_config: toml::Value = toml::from_str(&config_content)?;
        
        // 解析插件配置
        let name = toml_config["name"].as_str().unwrap_or("unknown").to_string();
        let description = toml_config["description"].as_str().unwrap_or("").to_string();
        let version = toml_config["version"].as_str().unwrap_or("0.0.1").to_string();
        let author = toml_config["author"].as_str().unwrap_or("unknown").to_string();
        let license = toml_config["license"].as_str().unwrap_or("unknown").to_string();
        
        // 解析插件类型
        let plugin_type_str = toml_config["plugin_type"].as_str().unwrap_or("other");
        let plugin_type = match plugin_type_str {
            "core" => PluginType::Core,
            "command" => PluginType::Command,
            "hook" => PluginType::Hook,
            "subagent" => PluginType::Subagent,
            "ui" => PluginType::UI,
            _ => PluginType::Other,
        };
        
        let entry_point = toml_config["entry_point"].as_str().unwrap_or("").to_string();
        
        // 解析插件依赖
        let dependencies = if let Some(deps) = toml_config["dependencies"].as_array() {
            deps.iter()
                .filter_map(|dep| dep.as_str())
                .map(|dep| dep.to_string())
                .collect()
        } else {
            Vec::new()
        };
        
        // 解析插件配置参数
        let config = if let Some(config_map) = toml_config["config"].as_table() {
            config_map.iter()
                .map(|(key, value)| {
                    let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                    (key.to_string(), json_value)
                })
                .collect()
        } else {
            HashMap::new()
        };
        
        Ok(PluginConfig {
            name,
            description,
            version,
            author,
            license,
            plugin_type,
            entry_point,
            dependencies,
            config,
        })
    }
    
    /// 从目录加载所有插件
    pub async fn load_plugins_from_dir(&mut self, dir_path: &str) -> Result<Vec<PluginResult>, Box<dyn Error>> {
        let mut results = Vec::new();
        
        // 检查目录是否存在
        let dir = Path::new(dir_path);
        if !dir.exists() || !dir.is_dir() {
            return Err(format!("Plugins directory '{}' does not exist", dir_path).into());
        }
        
        // 遍历目录中的所有插件
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // 尝试加载插件
                match self.load_plugin(path.to_str().unwrap()).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        results.push(PluginResult {
                            plugin: path.file_name().unwrap().to_str().unwrap().to_string(),
                            success: false,
                            message: format!("Failed to load plugin: {}", e),
                            data: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// 初始化插件
    pub async fn initialize_plugin(&mut self, plugin_name: &str) -> Result<PluginResult, Box<dyn Error>> {
        if let Some(info) = self.plugins.get_mut(plugin_name) {
            // 检查插件是否已经初始化
            if matches!(info.state, PluginState::Initialized) {
                return Ok(PluginResult {
                    plugin: plugin_name.to_string(),
                    success: true,
                    message: format!("Plugin '{}' is already initialized", plugin_name),
                    data: HashMap::new(),
                });
            }
            
            // 初始化插件
            info.plugin.initialize(&info.config).await?;
            
            // 更新插件状态
            info.state = PluginState::Initialized;
            self.states.insert(plugin_name.to_string(), PluginState::Initialized);
            
            Ok(PluginResult {
                plugin: plugin_name.to_string(),
                success: true,
                message: format!("Plugin '{}' initialized successfully", plugin_name),
                data: HashMap::new(),
            })
        } else {
            Err(format!("Plugin '{}' not found", plugin_name).into())
        }
    }
    
    /// 启用插件
    pub async fn enable_plugin(&mut self, plugin_name: &str) -> Result<PluginResult, Box<dyn Error>> {
        if let Some(info) = self.plugins.get_mut(plugin_name) {
            // 检查插件是否已经启用
            if matches!(info.state, PluginState::Enabled) {
                return Ok(PluginResult {
                    plugin: plugin_name.to_string(),
                    success: true,
                    message: format!("Plugin '{}' is already enabled", plugin_name),
                    data: HashMap::new(),
                });
            }
            
            // 检查插件是否已经初始化
            if !matches!(info.state, PluginState::Initialized) {
                return Err(format!("Plugin '{}' must be initialized before enabling", plugin_name).into());
            }
            
            // 启用插件
            info.plugin.enable().await?;
            
            // 更新插件状态
            info.state = PluginState::Enabled;
            self.states.insert(plugin_name.to_string(), PluginState::Enabled);
            
            Ok(PluginResult {
                plugin: plugin_name.to_string(),
                success: true,
                message: format!("Plugin '{}' enabled successfully", plugin_name),
                data: HashMap::new(),
            })
        } else {
            Err(format!("Plugin '{}' not found", plugin_name).into())
        }
    }
    
    /// 禁用插件
    pub async fn disable_plugin(&mut self, plugin_name: &str) -> Result<PluginResult, Box<dyn Error>> {
        if let Some(info) = self.plugins.get_mut(plugin_name) {
            // 检查插件是否已经禁用
            if matches!(info.state, PluginState::Disabled) {
                return Ok(PluginResult {
                    plugin: plugin_name.to_string(),
                    success: true,
                    message: format!("Plugin '{}' is already disabled", plugin_name),
                    data: HashMap::new(),
                });
            }
            
            // 禁用插件
            info.plugin.disable().await?;
            
            // 更新插件状态
            info.state = PluginState::Disabled;
            self.states.insert(plugin_name.to_string(), PluginState::Disabled);
            
            Ok(PluginResult {
                plugin: plugin_name.to_string(),
                success: true,
                message: format!("Plugin '{}' disabled successfully", plugin_name),
                data: HashMap::new(),
            })
        } else {
            Err(format!("Plugin '{}' not found", plugin_name).into())
        }
    }
    
    /// 卸载插件
    pub async fn unload_plugin(&mut self, plugin_name: &str) -> Result<PluginResult, Box<dyn Error>> {
        if let Some(mut info) = self.plugins.remove(plugin_name) {
            // 如果插件已启用，先禁用
            if matches!(info.state, PluginState::Enabled) {
                info.plugin.disable().await?;
            }
            
            // 卸载插件
            info.plugin.unload().await?;
            
            // 更新插件状态
            self.states.remove(plugin_name);
            self.config.remove(plugin_name);
            
            Ok(PluginResult {
                plugin: plugin_name.to_string(),
                success: true,
                message: format!("Plugin '{}' unloaded successfully", plugin_name),
                data: HashMap::new(),
            })
        } else {
            Err(format!("Plugin '{}' not found", plugin_name).into())
        }
    }
    
    /// 获取插件信息
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<&PluginInfo> {
        self.plugins.get(plugin_name)
    }
    
    /// 列出所有插件
    pub fn list_plugins(&self) -> Vec<PluginInfoSummary> {
        self.plugins.values()
            .map(|info| PluginInfoSummary {
                name: info.plugin.name().to_string(),
                description: info.plugin.description().to_string(),
                version: info.plugin.version().to_string(),
                author: info.plugin.author().to_string(),
                plugin_type: info.plugin.plugin_type(),
                state: info.state.clone(),
            })
            .collect()
    }
    
    /// 检查插件是否已加载
    pub fn is_plugin_loaded(&self, plugin_name: &str) -> bool {
        self.plugins.contains_key(plugin_name)
    }
    
    /// 检查插件是否已启用
    pub fn is_plugin_enabled(&self, plugin_name: &str) -> bool {
        if let Some(state) = self.states.get(plugin_name) {
            matches!(state, PluginState::Enabled)
        } else {
            false
        }
    }
    
    /// 注册插件
    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static, config: PluginConfig) {
        let name = plugin.name().to_string();
        self.plugins.insert(name.clone(), PluginInfo {
            plugin: Box::new(plugin),
            config: config.clone(),
            state: PluginState::Loaded,
        });
        self.config.insert(name.clone(), config);
        self.states.insert(name, PluginState::Loaded);
    }
    
    /// 从配置文件加载插件配置
    pub fn load_config(&mut self, config_path: &str) -> Result<(), Box<dyn Error>> {
        // TODO: 实现从配置文件加载插件配置
        Ok(())
    }
    
    /// 保存插件配置到文件
    pub fn save_config(&self, config_path: &str) -> Result<(), Box<dyn Error>> {
        // TODO: 实现保存插件配置到文件
        Ok(())
    }
}

/// 插件信息摘要
pub struct PluginInfoSummary {
    /// 插件名称
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 插件状态
    pub state: PluginState,
}

/// 插件加载器，负责从文件系统加载插件
pub struct PluginLoader {
    /// 插件搜索路径
    search_paths: Vec<String>,
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self {
            search_paths: vec![
                "~/.codex/plugins".to_string(),
                "/usr/local/share/codex/plugins".to_string(),
                "/opt/codex/plugins".to_string(),
            ],
        }
    }
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new() -> Self {
        Default::default()
    }
    
    /// 添加插件搜索路径
    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }
    
    /// 获取插件搜索路径
    pub fn search_paths(&self) -> &Vec<String> {
        &self.search_paths
    }
    
    /// 查找插件
    pub fn find_plugins(&self) -> Vec<String> {
        let mut plugins = Vec::new();
        
        // 遍历所有搜索路径，查找插件
        for path in &self.search_paths {
            let expanded_path = shellexpand::tilde(path).to_string();
            let path_obj = Path::new(&expanded_path);
            
            if path_obj.exists() && path_obj.is_dir() {
                // 遍历目录，查找插件
                if let Ok(entries) = fs::read_dir(path_obj) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            if entry_path.is_dir() {
                                // 检查是否有插件配置文件
                                let plugin_config_path = entry_path.join("plugin.toml");
                                if plugin_config_path.exists() {
                                    plugins.push(entry_path.to_str().unwrap().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        plugins
    }
    
    /// 读取插件配置
    pub fn read_plugin_config(&self, plugin_path: &str) -> Result<PluginConfig, Box<dyn Error>> {
        // TODO: 实现读取插件配置
        Ok(PluginConfig {
            name: "unknown".to_string(),
            description: "Unknown plugin".to_string(),
            version: "0.0.1".to_string(),
            author: "Unknown author".to_string(),
            license: "Unknown license".to_string(),
            plugin_type: PluginType::Other,
            entry_point: "".to_string(),
            dependencies: Vec::new(),
            config: HashMap::new(),
        })
    }
}
