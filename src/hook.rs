use crate::plugins::{PluginConfig, PluginManager};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

/// Hook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HookEvent {
    Startup,
    Shutdown,
    CommandExecuted,
    CodeGenerated,
    KnowledgeBaseUpdated,
    TaskCreated,
    TaskUpdated,
    TaskDeleted,
    SubagentExecuted,
    ContextCompressed,
    ScraperCompleted,
    Custom(String),
}

impl From<HookEvent> for String {
    fn from(event: HookEvent) -> Self {
        match event {
            HookEvent::Startup => "startup".to_string(),
            HookEvent::Shutdown => "shutdown".to_string(),
            HookEvent::CommandExecuted => "command_executed".to_string(),
            HookEvent::CodeGenerated => "code_generated".to_string(),
            HookEvent::KnowledgeBaseUpdated => "knowledge_base_updated".to_string(),
            HookEvent::TaskCreated => "task_created".to_string(),
            HookEvent::TaskUpdated => "task_updated".to_string(),
            HookEvent::TaskDeleted => "task_deleted".to_string(),
            HookEvent::SubagentExecuted => "subagent_executed".to_string(),
            HookEvent::ContextCompressed => "context_compressed".to_string(),
            HookEvent::ScraperCompleted => "scraper_completed".to_string(),
            HookEvent::Custom(s) => s,
        }
    }
}

/// Hook type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookType {
    Command,
    Script,
    Inline,
    Custom,
}

impl From<HookType> for &'static str {
    fn from(hook_type: HookType) -> Self {
        match hook_type {
            HookType::Command => "command",
            HookType::Script => "script",
            HookType::Inline => "inline",
            HookType::Custom => "custom",
        }
    }
}

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Hook name
    name: String,
    /// Hook description
    description: String,
    /// Event to trigger on
    event: HookEvent,
    /// Hook type
    hook_type: HookType,
    /// Hook content/command
    content: String,
    /// Hook priority (0-100, higher = earlier execution)
    priority: u8,
    /// Whether the hook is enabled
    enabled: bool,
    /// Additional configuration parameters
    config: HashMap<String, serde_json::Value>,
}

/// Hook result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook name
    hook: String,
    /// Event that triggered the hook
    event: HookEvent,
    /// Whether the hook executed successfully
    success: bool,
    /// Hook output
    output: String,
    /// Execution time in seconds
    execution_time: f64,
    /// Any error message if failed
    error: Option<String>,
}

/// Hook trait for all hooks
#[async_trait]
pub trait Hook: Sync + Send {
    /// Get the hook name
    fn name(&self) -> &str;

    /// Get the hook description
    fn description(&self) -> &str;

    /// Get the event that triggers this hook
    fn event(&self) -> HookEvent;

    /// Get the hook type
    fn hook_type(&self) -> HookType;

    /// Execute the hook with context
    async fn execute(&self, context: &HookContext) -> Result<HookResult, Box<dyn Error>>;

    /// Check if the hook is enabled
    fn is_enabled(&self) -> bool;

    /// Enable or disable the hook
    fn set_enabled(&mut self, enabled: bool);

    /// Get the hook priority
    fn priority(&self) -> u8;
}

/// Hook context for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Event that triggered the hook
    event: HookEvent,
    /// Additional context data
    data: HashMap<String, serde_json::Value>,
    /// Environment variables
    env: HashMap<String, String>,
    /// Hook execution count
    execution_count: u64,
}

/// Command hook implementation
pub struct CommandHook {
    config: HookConfig,
}

impl CommandHook {
    /// Create a new command hook
    pub fn new(config: HookConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Hook for CommandHook {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn event(&self) -> HookEvent {
        self.config.event.clone()
    }

    fn hook_type(&self) -> HookType {
        self.config.hook_type
    }

    async fn execute(&self, _context: &HookContext) -> Result<HookResult, Box<dyn Error>> {
        let start_time = Instant::now();

        println!(
            "Executing command hook: {} for event {:?}",
            self.name(),
            self.event()
        );

        // Execute the command
        let output = Command::new("bash")
            .arg("-c")
            .arg(&self.config.content)
            .output()?;

        let execution_time = start_time.elapsed().as_secs_f64();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: true,
                output: stdout,
                execution_time,
                error: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: false,
                output: stderr.clone(),
                execution_time,
                error: Some(stderr),
            })
        }
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }
}

/// Script hook implementation
pub struct ScriptHook {
    config: HookConfig,
}

impl ScriptHook {
    /// Create a new script hook
    pub fn new(config: HookConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Hook for ScriptHook {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn event(&self) -> HookEvent {
        self.config.event.clone()
    }

    fn hook_type(&self) -> HookType {
        self.config.hook_type
    }

    async fn execute(&self, _context: &HookContext) -> Result<HookResult, Box<dyn Error>> {
        let start_time = Instant::now();

        println!(
            "Executing script hook: {} for event {:?}",
            self.name(),
            self.event()
        );

        // Check if script file exists
        if !std::path::Path::new(&self.config.content).exists() {
            return Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: false,
                output: "Script file not found".to_string(),
                execution_time: start_time.elapsed().as_secs_f64(),
                error: Some("Script file not found".to_string()),
            });
        }

        // Execute the script
        let output = Command::new(&self.config.content).output()?;

        let execution_time = start_time.elapsed().as_secs_f64();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: true,
                output: stdout,
                execution_time,
                error: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: false,
                output: stderr.clone(),
                execution_time,
                error: Some(stderr),
            })
        }
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }
}

/// Inline hook implementation (Rust closures)
pub struct InlineHook {
    config: HookConfig,
    callback: Arc<dyn Fn(&HookContext) -> Result<String, Box<dyn Error>> + Sync + Send>,
}

impl InlineHook {
    /// Create a new inline hook
    pub fn new(
        config: HookConfig,
        callback: Arc<dyn Fn(&HookContext) -> Result<String, Box<dyn Error>> + Sync + Send>,
    ) -> Self {
        Self { config, callback }
    }
}

#[async_trait]
impl Hook for InlineHook {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn event(&self) -> HookEvent {
        self.config.event.clone()
    }

    fn hook_type(&self) -> HookType {
        self.config.hook_type
    }

    async fn execute(&self, context: &HookContext) -> Result<HookResult, Box<dyn Error>> {
        let start_time = Instant::now();

        println!(
            "Executing inline hook: {} for event {:?}",
            self.name(),
            self.event()
        );

        // Execute the inline callback
        match (self.callback)(context) {
            Ok(output) => Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: true,
                output,
                execution_time: start_time.elapsed().as_secs_f64(),
                error: None,
            }),
            Err(e) => Ok(HookResult {
                hook: self.name().to_string(),
                event: self.event(),
                success: false,
                output: e.to_string(),
                execution_time: start_time.elapsed().as_secs_f64(),
                error: Some(e.to_string()),
            }),
        }
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }
}

/// Hook manager for managing hooks
#[derive(Default)]
pub struct HookManager {
    hooks: HashMap<HookEvent, Vec<Box<dyn Hook>>>,
    execution_count: u64,
    plugin_manager: Option<Arc<RwLock<PluginManager>>>,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        Default::default()
    }

    /// 设置插件管理器
    pub fn set_plugin_manager(&mut self, plugin_manager: Arc<RwLock<PluginManager>>) {
        self.plugin_manager = Some(plugin_manager);
    }

    /// 从插件加载钩子
    pub async fn load_from_plugins(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(plugin_manager) = &self.plugin_manager {
            let manager = plugin_manager.read().unwrap();
            // 遍历所有插件，检查是否有钩子插件
            for plugin_info in manager.list_plugins() {
                println!("Checking plugin {} for hooks...", plugin_info.name);

                // 检查插件是否实现了HookProvider接口
                // TODO: 主人~ 这里需要实现插件接口检查逻辑
                // if let Some(hook_provider) = plugin_info.plugin.as_any().downcast_ref::<dyn HookProvider>() {
                //     // 获取插件提供的钩子
                //     let hooks = hook_provider.provide_hooks();
                //
                //     // 注册钩子
                //     for hook in hooks {
                //         self.register(Box::new(hook));
                //         println!("Registered hook '{}' from plugin '{}'", hook.name(), plugin_info.name);
                //     }
                // }
            }
        }
        Ok(())
    }

    /// Register a new hook
    pub fn register(&mut self, hook: impl Hook + 'static) {
        let event = hook.event();

        if !self.hooks.contains_key(&event) {
            self.hooks.insert(event.clone(), Vec::new());
        }

        self.hooks.get_mut(&event).unwrap().push(Box::new(hook));

        // Sort hooks by priority (descending)
        self.hooks
            .get_mut(&event)
            .unwrap()
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Register a command hook
    pub fn register_command_hook(
        &mut self,
        name: &str,
        description: &str,
        event: HookEvent,
        command: &str,
        priority: u8,
    ) {
        let config = HookConfig {
            name: name.to_string(),
            description: description.to_string(),
            event,
            hook_type: HookType::Command,
            content: command.to_string(),
            priority,
            enabled: true,
            config: HashMap::new(),
        };

        let hook = CommandHook::new(config);
        self.register(hook);
    }

    /// Register a script hook
    pub fn register_script_hook(
        &mut self,
        name: &str,
        description: &str,
        event: HookEvent,
        script_path: &str,
        priority: u8,
    ) {
        let config = HookConfig {
            name: name.to_string(),
            description: description.to_string(),
            event,
            hook_type: HookType::Script,
            content: script_path.to_string(),
            priority,
            enabled: true,
            config: HashMap::new(),
        };

        let hook = ScriptHook::new(config);
        self.register(hook);
    }

    /// Register an inline hook
    pub fn register_inline_hook<F>(
        &mut self,
        name: &str,
        description: &str,
        event: HookEvent,
        callback: F,
        priority: u8,
    ) where
        F: Fn(&HookContext) -> Result<String, Box<dyn Error>> + Sync + Send + 'static,
    {
        let config = HookConfig {
            name: name.to_string(),
            description: description.to_string(),
            event,
            hook_type: HookType::Inline,
            content: "inline_closure".to_string(),
            priority,
            enabled: true,
            config: HashMap::new(),
        };

        let hook = InlineHook::new(config, Arc::new(callback));
        self.register(hook);
    }

    /// Trigger hooks for an event
    pub async fn trigger(
        &mut self,
        event: HookEvent,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<HookResult>, Box<dyn Error>> {
        self.execution_count += 1;

        let mut results = Vec::new();

        if let Some(hooks) = self.hooks.get(&event) {
            println!("Triggering {} hooks for event {:?}", hooks.len(), event);

            // Create hook context
            let context = HookContext {
                event,
                data,
                env: Self::get_environment(),
                execution_count: self.execution_count,
            };

            // Execute all enabled hooks for this event
            for hook in hooks {
                if hook.is_enabled() {
                    let result = hook.execute(&context).await?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Get all hooks for a specific event
    pub fn get_hooks(&self, event: HookEvent) -> Option<&Vec<Box<dyn Hook>>> {
        self.hooks.get(&event)
    }

    /// Get all registered hooks
    pub fn get_all_hooks(&self) -> Vec<&Box<dyn Hook>> {
        self.hooks.values().flat_map(|hooks| hooks.iter()).collect()
    }

    /// Enable a hook by name
    pub fn enable_hook(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        for hooks in self.hooks.values_mut() {
            for hook in hooks {
                if hook.name() == name {
                    hook.set_enabled(true);
                    return Ok(());
                }
            }
        }

        Err(format!("Hook '{}' not found", name).into())
    }

    /// Disable a hook by name
    pub fn disable_hook(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        for hooks in self.hooks.values_mut() {
            for hook in hooks {
                if hook.name() == name {
                    hook.set_enabled(false);
                    return Ok(());
                }
            }
        }

        Err(format!("Hook '{}' not found", name).into())
    }

    /// Remove a hook by name
    pub fn remove_hook(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let mut found = false;

        for hooks in self.hooks.values_mut() {
            hooks.retain(|hook| hook.name() != name);
            if hooks.iter().any(|hook| hook.name() == name) {
                found = true;
            }
        }

        if found {
            Ok(())
        } else {
            Err(format!("Hook '{}' not found", name).into())
        }
    }

    /// Get environment variables as a hash map
    fn get_environment() -> HashMap<String, String> {
        std::env::vars().collect()
    }

    /// Load hooks from a configuration file
    pub fn load_from_file(&mut self, _path: &str) -> Result<(), Box<dyn Error>> {
        // Read and parse hook configurations from file
        // This is a placeholder for future implementation
        Ok(())
    }

    /// Save hooks to a configuration file
    pub fn save_to_file(&self, _path: &str) -> Result<(), Box<dyn Error>> {
        // Save hook configurations to file
        // This is a placeholder for future implementation
        Ok(())
    }

    /// Create default hooks
    pub fn create_default_hooks(&mut self) {
        // Register some default hooks

        // Startup hook to print welcome message
        self.register_command_hook(
            "welcome_message",
            "Print welcome message on startup",
            HookEvent::Startup,
            "echo 'Welcome to Codex! Type 'help' for commands.'",
            100,
        );

        // Command executed hook to log commands
        self.register_command_hook(
            "log_commands",
            "Log all executed commands",
            HookEvent::CommandExecuted,
            "echo \"$(date) - Command executed: $CODEX_COMMAND \" >> ~/.codex_command_log.txt",
            50,
        );

        // Code generated hook to format code (if available)
        self.register_command_hook(
            "format_generated_code",
            "Format generated code with rustfmt if available",
            HookEvent::CodeGenerated,
            "which rustfmt > /dev/null && rustfmt $CODEX_GENERATED_FILE 2>/dev/null || true",
            75,
        );
    }
}

/// Hook registry for global access to hooks
pub struct HookRegistry {
    hook_manager: Arc<std::sync::RwLock<HookManager>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self {
            hook_manager: Arc::new(std::sync::RwLock::new(HookManager::new())),
        }
    }
}

impl HookRegistry {
    /// Get the hook manager
    pub fn get_manager(&self) -> Arc<std::sync::RwLock<HookManager>> {
        self.hook_manager.clone()
    }

    /// Trigger hooks for an event
    pub async fn trigger(
        &self,
        event: HookEvent,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<HookResult>, Box<dyn Error>> {
        let mut manager = self.hook_manager.write().unwrap();
        manager.trigger(event, data).await
    }
}
