use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::sync::Arc;
/// Command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Command name
    pub name: String,
    /// Command output
    pub output: String,
    /// Whether the command succeeded
    pub success: bool,
    /// Exit code (if applicable)
    pub exit_code: Option<i32>,
    /// Execution time in seconds
    pub execution_time: f64,
    /// Any additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Command trait for all commands
#[async_trait]
pub trait CommandHandler: Sync + Send {
    /// Get the command name
    fn name(&self) -> &str;
    
    /// Get the command description
    fn description(&self) -> &str;
    
    /// Get the command usage
    fn usage(&self) -> &str;
    
    /// Execute the command with arguments
    async fn execute(&self, args: &[&str], context: &CommandContext) -> Result<CommandResult, Box<dyn Error>>;
    
    /// Get the command aliases
    fn aliases(&self) -> Vec<&str>;
    
    /// Check if the command is built-in
    fn is_builtin(&self) -> bool;
}

/// Command context for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    /// Current working directory
    pub cwd: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Command history
    pub history: Vec<String>,
    /// Additional context data
    pub data: HashMap<String, serde_json::Value>,
}

/// Built-in command implementation
pub struct BuiltinCommand {
    name: String,
    description: String,
    usage: String,
    aliases: Vec<String>,
    handler: Arc<dyn Fn(&[&str], &CommandContext) -> Result<String, Box<dyn Error>> + Sync + Send>,
}

impl BuiltinCommand {
    /// Create a new built-in command
    pub fn new(
        name: &str,
        description: &str,
        usage: &str,
        aliases: Vec<&str>,
        handler: Arc<dyn Fn(&[&str], &CommandContext) -> Result<String, Box<dyn Error>> + Sync + Send>
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
            aliases: aliases.into_iter().map(|s| s.to_string()).collect(),
            handler,
        }
    }
}

#[async_trait]
impl CommandHandler for BuiltinCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn usage(&self) -> &str {
        &self.usage
    }
    
    async fn execute(&self, args: &[&str], context: &CommandContext) -> Result<CommandResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        
        println!("Executing built-in command: {} with args: {:?}", self.name, args);
        
        match (self.handler)(args, context) {
            Ok(output) => {
                let execution_time = start_time.elapsed().as_secs_f64();
                Ok(CommandResult {
                    name: self.name.clone(),
                    output,
                    success: true,
                    exit_code: None,
                    execution_time,
                    metadata: HashMap::new(),
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_secs_f64();
                Ok(CommandResult {
                    name: self.name.clone(),
                    output: e.to_string(),
                    success: false,
                    exit_code: None,
                    execution_time,
                    metadata: HashMap::new(),
                })
            }
        }
    }
    
    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }
    
    fn is_builtin(&self) -> bool {
        true
    }
}

/// External command implementation
pub struct ExternalCommand {
    name: String,
    description: String,
    usage: String,
    aliases: Vec<String>,
    command_path: String,
}

impl ExternalCommand {
    /// Create a new external command
    pub fn new(
        name: &str,
        description: &str,
        command_path: &str
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            usage: format!("{name} [args]"),
            aliases: Vec::new(),
            command_path: command_path.to_string(),
        }
    }
}

#[async_trait]
impl CommandHandler for ExternalCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn usage(&self) -> &str {
        &self.usage
    }
    
    async fn execute(&self, args: &[&str], context: &CommandContext) -> Result<CommandResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        
        println!("Executing external command: {} with args: {:?}", self.name, args);
        
        // Build command with arguments
        let mut command = Command::new(&self.command_path);
        command.args(args);
        command.current_dir(&context.cwd);
        
        // Set environment variables
        for (key, value) in &context.env {
            command.env(key, value);
        }
        
        // Execute command
        let output = command.output()?;
        
        let execution_time = start_time.elapsed().as_secs_f64();
        let success = output.status.success();
        let exit_code = output.status.code();
        
        let output_str = if success {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        
        Ok(CommandResult {
            name: self.name.clone(),
            output: output_str,
            success,
            exit_code,
            execution_time,
            metadata: HashMap::new(),
        })
    }
    
    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }
    
    fn is_builtin(&self) -> bool {
        false
    }
}

/// Command registry for managing commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler>>,
    aliases: HashMap<String, String>,
    builtin_commands: Vec<String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            builtin_commands: Vec::new(),
        };
        
        registry.register_builtin_commands();
        
        registry
    }
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Register a command handler
    pub fn register(&mut self, command: impl CommandHandler + 'static) {
        let name = command.name().to_string();
        let is_builtin = command.is_builtin();
        let aliases: Vec<String> = command.aliases().iter().map(|alias| alias.to_string()).collect();
        
        self.commands.insert(name.clone(), Box::new(command));
        
        if is_builtin {
            self.builtin_commands.push(name.clone());
        }
        
        // Register aliases
        for alias in aliases {
            self.aliases.insert(alias, name.clone());
        }
    }
    
    /// Register a built-in command
    pub fn register_builtin(
        &mut self,
        name: &str,
        description: &str,
        usage: &str,
        aliases: Vec<&str>,
        handler: Arc<dyn Fn(&[&str], &CommandContext) -> Result<String, Box<dyn Error>> + Sync + Send>
    ) {
        let command = BuiltinCommand::new(name, description, usage, aliases, handler);
        self.register(command);
    }
    
    /// Register an external command
    pub fn register_external(
        &mut self,
        name: &str,
        description: &str,
        command_path: &str
    ) {
        let command = ExternalCommand::new(name, description, command_path);
        self.register(command);
    }
    
    /// Register built-in commands
    fn register_builtin_commands(&mut self) {
        // Help command
        self.register_builtin(
            "help",
            "Show help for commands",
            "help [command]",
            vec!["h"],
            Arc::new(|args, _context| {
                if args.is_empty() {
                    Ok("Available commands: \n\nType 'help <command>' for detailed help.\n\nBuilt-in commands:\n- help, h - Show help for commands\n- exit, quit - Exit the application\n- clear, cls - Clear the screen\n- history - Show command history\n- config - Manage configuration\n- alias - Manage command aliases\n- hook - Manage hooks\n- subagent - Manage subagents\n- kb - Knowledge base commands\n- task - Task management commands\n- solo - Solo mode commands\n- scrape - Web scraping commands\n- code - Code generation commands".to_string())
                } else {
                    Ok(format!("Help for command '{}'\n\nNot implemented yet.", args[0]))
                }
            })
        );
        
        // Exit command
        self.register_builtin(
            "exit",
            "Exit the application",
            "exit",
            vec!["quit", "q"],
            Arc::new(|_args, _context| {
                Ok("Exiting application...".to_string())
            })
        );
        
        // Clear screen command
        self.register_builtin(
            "clear",
            "Clear the screen",
            "clear",
            vec!["cls"],
            Arc::new(|_args, _context| {
                Ok("Screen cleared".to_string())
            })
        );
        
        // History command
        self.register_builtin(
            "history",
            "Show command history",
            "history [--clear]",
            vec!["hist"],
            Arc::new(|args, _context| {
                if args.contains(&"--clear") {
                    Ok("Command history cleared".to_string())
                } else {
                    Ok("Command history not implemented yet".to_string())
                }
            })
        );
        
        // Config command
        self.register_builtin(
            "config",
            "Manage configuration",
            "config [get|set|list|reset] [key] [value]",
            vec!["cfg"],
            Arc::new(|args, _context| {
                Ok(format!("Config command with args: {:?}", args))
            })
        );
        
        // Alias command
        self.register_builtin(
            "alias",
            "Manage command aliases",
            "alias [add|remove|list] [name] [command]",
            vec!["al"],
            Arc::new(|args, _context| {
                Ok(format!("Alias command with args: {:?}", args))
            })
        );
        
        // Hook command
        self.register_builtin(
            "hook",
            "Manage hooks",
            "hook [list|enable|disable|add|remove] [name]",
            vec!["hk"],
            Arc::new(|args, _context| {
                Ok(format!("Hook command with args: {:?}", args))
            })
        );
        
        // Subagent command
        self.register_builtin(
            "subagent",
            "Manage subagents",
            "subagent [list|execute|add|remove] [name]",
            vec!["sa"],
            Arc::new(|args, _context| {
                Ok(format!("Subagent command with args: {:?}", args))
            })
        );
    }
    
    /// Get a command by name
    pub fn get_command(&self, name: &str) -> Option<&Box<dyn CommandHandler>> {
        // Check if it's an alias first
        if let Some(actual_name) = self.aliases.get(name) {
            self.commands.get(actual_name)
        } else {
            self.commands.get(name)
        }
    }
    
    /// Execute a command by name
    pub async fn execute(
        &self,
        name: &str,
        args: &[&str],
        context: &CommandContext
    ) -> Result<CommandResult, Box<dyn Error>> {
        let command = self.get_command(name)
            .ok_or(format!("Command '{}' not found", name))?;
        
        command.execute(args, context).await
    }
    
    /// List all registered commands
    pub fn list_commands(&self) -> Vec<CommandInfo> {
        self.commands.values()
            .map(|command| CommandInfo {
                name: command.name().to_string(),
                description: command.description().to_string(),
                usage: command.usage().to_string(),
                aliases: command.aliases().iter().map(|s| s.to_string()).collect(),
                is_builtin: command.is_builtin(),
            })
            .collect()
    }
    
    /// List built-in commands
    pub fn list_builtin_commands(&self) -> Vec<CommandInfo> {
        self.commands.values()
            .filter(|command| command.is_builtin())
            .map(|command| CommandInfo {
                name: command.name().to_string(),
                description: command.description().to_string(),
                usage: command.usage().to_string(),
                aliases: command.aliases().iter().map(|s| s.to_string()).collect(),
                is_builtin: true,
            })
            .collect()
    }
    
    /// Add an alias for a command
    pub fn add_alias(&mut self, alias: &str, command_name: &str) -> Result<(), Box<dyn Error>> {
        // Check if the command exists
        if !self.commands.contains_key(command_name) {
            return Err(format!("Command '{}' not found", command_name).into());
        }
        
        self.aliases.insert(alias.to_string(), command_name.to_string());
        Ok(())
    }
    
    /// Remove an alias
    pub fn remove_alias(&mut self, alias: &str) -> Result<(), Box<dyn Error>> {
        if self.aliases.remove(alias).is_none() {
            return Err(format!("Alias '{}' not found", alias).into());
        }
        Ok(())
    }
    
    /// Get all aliases
    pub fn get_aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }
    
    /// Load commands from a configuration file
    pub fn load_from_file(&mut self, _path: &str) -> Result<(), Box<dyn Error>> {
        // Read and parse command configurations from file
        // This is a placeholder for future implementation
        Ok(())
    }
    
    /// Save commands to a configuration file
    pub fn save_to_file(&self, _path: &str) -> Result<(), Box<dyn Error>> {
        // Save command configurations to file
        // This is a placeholder for future implementation
        Ok(())
    }
}

/// Command information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub is_builtin: bool,
}

/// Command context builder for easy context creation
pub struct CommandContextBuilder {
    cwd: String,
    env: HashMap<String, String>,
    history: Vec<String>,
    data: HashMap<String, serde_json::Value>,
}

impl Default for CommandContextBuilder {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            env: std::env::vars().collect(),
            history: Vec::new(),
            data: HashMap::new(),
        }
    }
}

impl CommandContextBuilder {
    /// Create a new command context builder
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Set the current working directory
    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.cwd = cwd.to_string();
        self
    }
    
    /// Set an environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set multiple environment variables
    pub fn with_envs(mut self, envs: &HashMap<String, String>) -> Self {
        self.env.extend(envs.clone());
        self
    }
    
    /// Set command history
    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }
    
    /// Add a data item
    pub fn with_data(mut self, key: &str, value: serde_json::Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }
    
    /// Build the command context
    pub fn build(self) -> CommandContext {
        CommandContext {
            cwd: self.cwd,
            env: self.env,
            history: self.history,
            data: self.data,
        }
    }
}

/// Command executor for executing commands with context
pub struct CommandExecutor {
    registry: Arc<CommandRegistry>,
    context: CommandContext,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            registry,
            context: CommandContextBuilder::new().build(),
        }
    }
    
    /// Create a new command executor with custom context
    pub fn with_context(registry: Arc<CommandRegistry>, context: CommandContext) -> Self {
        Self {
            registry,
            context,
        }
    }
    
    /// Execute a command with arguments
    pub async fn execute(
        &self,
        command_line: &str
    ) -> Result<CommandResult, Box<dyn Error>> {
        // Parse command line into command and arguments
        let parts: Vec<String> = shell_words::split(command_line)?;
        if parts.is_empty() {
            return Err("Empty command".into());
        }
        
        let command_name = &parts[0];
        let args = &parts[1..];
        
        // Convert Vec<String> to Vec<&str>
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.registry.execute(command_name, &args_ref, &self.context).await
    }
    
    /// Get the current context
    pub fn context(&self) -> &CommandContext {
        &self.context
    }
    
    /// Update the context
    pub fn update_context(&mut self, context: CommandContext) {
        self.context = context;
    }
    
    /// Get the command registry
    pub fn registry(&self) -> &Arc<CommandRegistry> {
        &self.registry
    }
}

/// Command parser for parsing command lines
pub struct CommandParser {
    // Parser configuration
    allow_aliases: bool,
    allow_env_expansion: bool,
    allow_globbing: bool,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            allow_aliases: true,
            allow_env_expansion: true,
            allow_globbing: false,
        }
    }
}

impl CommandParser {
    /// Create a new command parser
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Parse a command line string into command and arguments
    pub fn parse(&self, command_line: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
        // Use shell-words to split the command line
        let parts = shell_words::split(command_line)?;
        
        if parts.is_empty() {
            return Err("Empty command".into());
        }
        
        let command_name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Ok((command_name, args))
    }
    
    /// Enable or disable aliases
    pub fn set_allow_aliases(&mut self, allow: bool) {
        self.allow_aliases = allow;
    }
    
    /// Enable or disable environment expansion
    pub fn set_allow_env_expansion(&mut self, allow: bool) {
        self.allow_env_expansion = allow;
    }
    
    /// Enable or disable globbing
    pub fn set_allow_globbing(&mut self, allow: bool) {
        self.allow_globbing = allow;
    }
}
