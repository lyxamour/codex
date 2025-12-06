use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// Subagent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SubagentConfig {
    /// Subagent name
    name: String,
    /// Subagent description
    description: String,
    /// AI model to use
    model: String,
    /// System prompt template
    system_prompt: String,
    /// Temperature for generation
    temperature: f32,
    /// Maximum tokens for response
    max_tokens: usize,
    /// Subagent type/category
    agent_type: SubagentType,
    /// Default tools available to the subagent
    default_tools: Vec<String>,
    /// Configuration parameters
    config: HashMap<String, serde_json::Value>,
}

/// Subagent type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubagentType {
    CodeGenerator,
    CodeReviewer,
    Tester,
    Debugger,
    Architect,
    Documenter,
    Translator,
    Summarizer,
    Planner,
    Other,
}

impl From<SubagentType> for &'static str {
    fn from(agent_type: SubagentType) -> Self {
        match agent_type {
            SubagentType::CodeGenerator => "code-generator",
            SubagentType::CodeReviewer => "code-reviewer",
            SubagentType::Tester => "tester",
            SubagentType::Debugger => "debugger",
            SubagentType::Architect => "architect",
            SubagentType::Documenter => "documenter",
            SubagentType::Translator => "translator",
            SubagentType::Summarizer => "summarizer",
            SubagentType::Planner => "planner",
            SubagentType::Other => "other",
        }
    }
}

/// Result from a subagent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SubagentResult {
    /// Result content
    content: String,
    /// Subagent used
    subagent: String,
    /// Model used
    model: String,
    /// Tokens used
    tokens_used: Option<usize>,
    /// Execution time in seconds
    execution_time: f64,
    /// Any additional metadata
    metadata: HashMap<String, serde_json::Value>,
}

/// Subagent trait for all subagents
#[async_trait]
#[allow(dead_code)]
pub trait Subagent: Sync + Send {
    /// Get the subagent name
    fn name(&self) -> &str;

    /// Get the subagent description
    fn description(&self) -> &str;

    /// Get the subagent type
    fn agent_type(&self) -> SubagentType;

    /// Execute the subagent with a prompt and context
    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>>;

    /// Get the subagent configuration
    fn config(&self) -> &SubagentConfig;

    /// Update the subagent configuration
    fn update_config(&mut self, config: SubagentConfig);
}

/// Subagent registry for managing subagents
pub struct SubagentRegistry {
    subagents: HashMap<String, Box<dyn Subagent>>,
    default_subagents: HashMap<SubagentType, String>,
}

impl Default for SubagentRegistry {
    fn default() -> Self {
        let mut registry = Self {
            subagents: HashMap::new(),
            default_subagents: HashMap::new(),
        };

        // Register default subagents
        registry.register_default_subagents();

        registry
    }
}

impl SubagentRegistry {
    /// Create a new subagent registry
    pub fn new() -> Self {
        Default::default()
    }

    /// Register default subagents
    fn register_default_subagents(&mut self) {
        // Register code generator subagent
        let code_gen_config = SubagentConfig {
            name: "code-generator".to_string(),
            description: "Subagent for generating code".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are a professional code generator. Generate clean, efficient, and well-documented code based on the user's request.\n\nAlways follow these guidelines:\n1. Write production-quality code\n2. Include appropriate comments\n3. Follow best practices for the specified language\n4. Provide only the code, no explanations unless requested\n5. Ensure the code is functional and correct".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            agent_type: SubagentType::CodeGenerator,
            default_tools: vec!["file_edit".to_string(), "code_lookup".to_string()],
            config: HashMap::new(),
        };

        let code_gen_subagent = CodeGeneratorSubagent::new(code_gen_config);
        self.register(code_gen_subagent);
        self.set_default_subagent(SubagentType::CodeGenerator, "code-generator".to_string());

        // Register code reviewer subagent
        let code_reviewer_config = SubagentConfig {
            name: "code-reviewer".to_string(),
            description: "Subagent for reviewing code".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are a professional code reviewer. Analyze the provided code and provide detailed feedback on the following aspects:\n\n1. Code quality and readability\n2. Performance issues\n3. Security vulnerabilities\n4. Best practices compliance\n5. Bug risks\n6. Suggested improvements\n\nBe thorough but constructive in your feedback.".to_string(),
            temperature: 0.5,
            max_tokens: 4096,
            agent_type: SubagentType::CodeReviewer,
            default_tools: vec!["code_analysis".to_string(), "security_scan".to_string()],
            config: HashMap::new(),
        };

        let code_reviewer_subagent = CodeReviewerSubagent::new(code_reviewer_config);
        self.register(code_reviewer_subagent);
        self.set_default_subagent(SubagentType::CodeReviewer, "code-reviewer".to_string());

        // Register tester subagent
        let tester_config = SubagentConfig {
            name: "tester".to_string(),
            description: "Subagent for generating tests".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are a professional test engineer. Generate comprehensive tests for the provided code.\n\nAlways follow these guidelines:\n1. Test edge cases thoroughly\n2. Include unit tests, integration tests, and end-to-end tests where appropriate\n3. Follow the testing framework conventions for the specified language\n4. Ensure tests cover all critical functionality\n5. Write clear, descriptive test names".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            agent_type: SubagentType::Tester,
            default_tools: vec!["test_runner".to_string(), "coverage_analyzer".to_string()],
            config: HashMap::new(),
        };

        let tester_subagent = TesterSubagent::new(tester_config);
        self.register(tester_subagent);
        self.set_default_subagent(SubagentType::Tester, "tester".to_string());
    }

    /// Register a new subagent
    pub fn register(&mut self, subagent: impl Subagent + 'static) {
        let name = subagent.name().to_string();
        self.subagents.insert(name, Box::new(subagent));
    }

    /// Get a subagent by name
    pub fn get(&self, name: &str) -> Option<&Box<dyn Subagent>> {
        self.subagents.get(name)
    }

    /// Get a mutable subagent by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Subagent>> {
        self.subagents.get_mut(name)
    }

    /// Get the default subagent for a specific type
    pub fn get_default(&self, agent_type: SubagentType) -> Option<&Box<dyn Subagent>> {
        if let Some(name) = self.default_subagents.get(&agent_type) {
            self.subagents.get(name)
        } else {
            None
        }
    }

    /// Set the default subagent for a specific type
    pub fn set_default_subagent(&mut self, agent_type: SubagentType, name: String) {
        self.default_subagents.insert(agent_type, name);
    }

    /// List all registered subagents
    pub fn list(&self) -> Vec<SubagentInfo> {
        self.subagents
            .values()
            .map(|subagent| SubagentInfo {
                name: subagent.name().to_string(),
                description: subagent.description().to_string(),
                agent_type: subagent.agent_type(),
                model: subagent.config().model.clone(),
            })
            .collect()
    }

    /// Execute a subagent by name
    pub async fn execute(
        &self,
        name: &str,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let subagent = self
            .get(name)
            .ok_or(format!("Subagent '{}' not found", name))?;

        subagent.execute(prompt, context).await
    }

    /// Execute the default subagent for a specific type
    pub async fn execute_default(
        &self,
        agent_type: SubagentType,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let subagent = self.get_default(agent_type).ok_or(format!(
            "No default subagent found for type {:?}",
            agent_type
        ))?;

        subagent.execute(prompt, context).await
    }
}

/// Subagent information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    pub name: String,
    pub description: String,
    pub agent_type: SubagentType,
    pub model: String,
}

/// Code generator subagent implementation
pub struct CodeGeneratorSubagent {
    config: SubagentConfig,
    ai_client: crate::ai::AIClient,
}

impl CodeGeneratorSubagent {
    /// Create a new code generator subagent
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            ai_client: crate::ai::AIClient::default(),
        }
    }
}

#[async_trait]
impl Subagent for CodeGeneratorSubagent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn agent_type(&self) -> SubagentType {
        self.config.agent_type
    }

    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();

        // Format prompt with system prompt
        let full_prompt = if let Some(context) = context {
            format!(
                "{}\n\nContext:\n{}\n\nUser Request:\n{}",
                self.config.system_prompt, context, prompt
            )
        } else {
            format!("{}\n\nUser Request:\n{}", self.config.system_prompt, prompt)
        };

        // Generate response using AI client
        let response = self
            .ai_client
            .generate_response(&full_prompt, Some(&self.config.model))
            .await?;

        let execution_time = start_time.elapsed().as_secs_f64();

        Ok(SubagentResult {
            content: response.content().to_string(),
            subagent: self.name().to_string(),
            model: response.model().to_string(),
            tokens_used: response.tokens_used(),
            execution_time,
            metadata: HashMap::new(),
        })
    }

    fn config(&self) -> &SubagentConfig {
        &self.config
    }

    fn update_config(&mut self, config: SubagentConfig) {
        self.config = config;
    }
}

/// Code reviewer subagent implementation
pub struct CodeReviewerSubagent {
    config: SubagentConfig,
    ai_client: crate::ai::AIClient,
}

impl CodeReviewerSubagent {
    /// Create a new code reviewer subagent
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            ai_client: crate::ai::AIClient::default(),
        }
    }
}

#[async_trait]
impl Subagent for CodeReviewerSubagent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn agent_type(&self) -> SubagentType {
        self.config.agent_type
    }

    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();

        // Format prompt with system prompt
        let full_prompt = if let Some(context) = context {
            format!(
                "{}\n\nContext:\n{}\n\nCode to review:\n{}",
                self.config.system_prompt, context, prompt
            )
        } else {
            format!(
                "{}\n\nCode to review:\n{}",
                self.config.system_prompt, prompt
            )
        };

        // Generate response using AI client
        let response = self
            .ai_client
            .generate_response(&full_prompt, Some(&self.config.model))
            .await?;

        let execution_time = start_time.elapsed().as_secs_f64();

        Ok(SubagentResult {
            content: response.content().to_string(),
            subagent: self.name().to_string(),
            model: response.model().to_string(),
            tokens_used: response.tokens_used(),
            execution_time,
            metadata: HashMap::new(),
        })
    }

    fn config(&self) -> &SubagentConfig {
        &self.config
    }

    fn update_config(&mut self, config: SubagentConfig) {
        self.config = config;
    }
}

/// Tester subagent implementation
pub struct TesterSubagent {
    config: SubagentConfig,
    ai_client: crate::ai::AIClient,
}

impl TesterSubagent {
    /// Create a new tester subagent
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            ai_client: crate::ai::AIClient::default(),
        }
    }
}

#[async_trait]
impl Subagent for TesterSubagent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn agent_type(&self) -> SubagentType {
        self.config.agent_type
    }

    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();

        // Format prompt with system prompt
        let full_prompt = if let Some(context) = context {
            format!(
                "{}\n\nContext:\n{}\n\nCode to test:\n{}",
                self.config.system_prompt, context, prompt
            )
        } else {
            format!("{}\n\nCode to test:\n{}", self.config.system_prompt, prompt)
        };

        // Generate response using AI client
        let response = self
            .ai_client
            .generate_response(&full_prompt, Some(&self.config.model))
            .await?;

        let execution_time = start_time.elapsed().as_secs_f64();

        Ok(SubagentResult {
            content: response.content().to_string(),
            subagent: self.name().to_string(),
            model: response.model().to_string(),
            tokens_used: response.tokens_used(),
            execution_time,
            metadata: HashMap::new(),
        })
    }

    fn config(&self) -> &SubagentConfig {
        &self.config
    }

    fn update_config(&mut self, config: SubagentConfig) {
        self.config = config;
    }
}

/// Debugger subagent implementation
pub struct DebuggerSubagent {
    config: SubagentConfig,
    ai_client: crate::ai::AIClient,
}

impl DebuggerSubagent {
    /// Create a new debugger subagent
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            ai_client: crate::ai::AIClient::default(),
        }
    }
}

#[async_trait]
impl Subagent for DebuggerSubagent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn agent_type(&self) -> SubagentType {
        self.config.agent_type
    }

    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();

        // Format prompt with system prompt
        let full_prompt = if let Some(context) = context {
            format!(
                "{}\n\nContext:\n{}\n\nDebug Request:\n{}",
                self.config.system_prompt, context, prompt
            )
        } else {
            format!(
                "{}\n\nDebug Request:\n{}",
                self.config.system_prompt, prompt
            )
        };

        // Generate response using AI client
        let response = self
            .ai_client
            .generate_response(&full_prompt, Some(&self.config.model))
            .await?;

        let execution_time = start_time.elapsed().as_secs_f64();

        Ok(SubagentResult {
            content: response.content().to_string(),
            subagent: self.name().to_string(),
            model: response.model().to_string(),
            tokens_used: response.tokens_used(),
            execution_time,
            metadata: HashMap::new(),
        })
    }

    fn config(&self) -> &SubagentConfig {
        &self.config
    }

    fn update_config(&mut self, config: SubagentConfig) {
        self.config = config;
    }
}

/// Documenter subagent implementation
pub struct DocumenterSubagent {
    config: SubagentConfig,
    ai_client: crate::ai::AIClient,
}

impl DocumenterSubagent {
    /// Create a new documenter subagent
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            ai_client: crate::ai::AIClient::default(),
        }
    }
}

#[async_trait]
impl Subagent for DocumenterSubagent {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn agent_type(&self) -> SubagentType {
        self.config.agent_type
    }

    async fn execute(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<SubagentResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();

        // Format prompt with system prompt
        let full_prompt = if let Some(context) = context {
            format!(
                "{}\n\nContext:\n{}\n\nContent to document:\n{}",
                self.config.system_prompt, context, prompt
            )
        } else {
            format!(
                "{}\n\nContent to document:\n{}",
                self.config.system_prompt, prompt
            )
        };

        // Generate response using AI client
        let response = self
            .ai_client
            .generate_response(&full_prompt, Some(&self.config.model))
            .await?;

        let execution_time = start_time.elapsed().as_secs_f64();

        Ok(SubagentResult {
            content: response.content().to_string(),
            subagent: self.name().to_string(),
            model: response.model().to_string(),
            tokens_used: response.tokens_used(),
            execution_time,
            metadata: HashMap::new(),
        })
    }

    fn config(&self) -> &SubagentConfig {
        &self.config
    }

    fn update_config(&mut self, config: SubagentConfig) {
        self.config = config;
    }
}
