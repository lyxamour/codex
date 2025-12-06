use crate::ai::AIClient;
use std::error::Error;
use std::fs;
use std::path::Path;
use tree_sitter::{Parser, TreeCursor};
use walkdir::WalkDir;

/// Code language types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    C,
    Cpp,
    Go,
    Ruby,
    PHP,
    HTML,
    CSS,
    JSON,
    YAML,
    TOML,
    Markdown,
    Other,
}

impl From<&str> for CodeLanguage {
    fn from(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => CodeLanguage::Rust,
            "py" => CodeLanguage::Python,
            "js" => CodeLanguage::JavaScript,
            "ts" => CodeLanguage::TypeScript,
            "java" => CodeLanguage::Java,
            "c" => CodeLanguage::C,
            "cpp" => CodeLanguage::Cpp,
            "cc" => CodeLanguage::Cpp,
            "cxx" => CodeLanguage::Cpp,
            "go" => CodeLanguage::Go,
            "rb" => CodeLanguage::Ruby,
            "php" => CodeLanguage::PHP,
            "html" => CodeLanguage::HTML,
            "css" => CodeLanguage::CSS,
            "json" => CodeLanguage::JSON,
            "yaml" => CodeLanguage::YAML,
            "yml" => CodeLanguage::YAML,
            "toml" => CodeLanguage::TOML,
            "md" => CodeLanguage::Markdown,
            _ => CodeLanguage::Other,
        }
    }
}

impl From<CodeLanguage> for &'static str {
    fn from(lang: CodeLanguage) -> Self {
        match lang {
            CodeLanguage::Rust => "rust",
            CodeLanguage::Python => "python",
            CodeLanguage::JavaScript => "javascript",
            CodeLanguage::TypeScript => "typescript",
            CodeLanguage::Java => "java",
            CodeLanguage::C => "c",
            CodeLanguage::Cpp => "cpp",
            CodeLanguage::Go => "go",
            CodeLanguage::Ruby => "ruby",
            CodeLanguage::PHP => "php",
            CodeLanguage::HTML => "html",
            CodeLanguage::CSS => "css",
            CodeLanguage::JSON => "json",
            CodeLanguage::YAML => "yaml",
            CodeLanguage::TOML => "toml",
            CodeLanguage::Markdown => "markdown",
            CodeLanguage::Other => "text",
        }
    }
}

/// Code programming functionality
pub struct CodeProgrammer {
    ai_client: AIClient,
    parsers: std::collections::HashMap<CodeLanguage, Parser>,
}

impl Default for CodeProgrammer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeProgrammer {
    /// Create a new code programmer instance
    pub fn new() -> Self {
        let ai_client = AIClient::default();
        let mut parsers = std::collections::HashMap::new();
        
        // Initialize parsers for supported languages
        if let Ok(parser) = Self::create_parser(CodeLanguage::Rust) {
            parsers.insert(CodeLanguage::Rust, parser);
        }
        if let Ok(parser) = Self::create_parser(CodeLanguage::Python) {
            parsers.insert(CodeLanguage::Python, parser);
        }
        if let Ok(parser) = Self::create_parser(CodeLanguage::JavaScript) {
            parsers.insert(CodeLanguage::JavaScript, parser);
        }
        if let Ok(parser) = Self::create_parser(CodeLanguage::TypeScript) {
            parsers.insert(CodeLanguage::TypeScript, parser);
        }
        
        Self {
            ai_client,
            parsers,
        }
    }
    
    /// Create a parser for a specific language
    fn create_parser(language: CodeLanguage) -> Result<Parser, Box<dyn Error>> {
        let mut parser = Parser::new();
        let language = match language {
            CodeLanguage::Rust => tree_sitter_rust::language(),
            CodeLanguage::Python => tree_sitter_python::language(),
            CodeLanguage::JavaScript => tree_sitter_javascript::language(),
            CodeLanguage::TypeScript => tree_sitter_typescript::language_typescript(),
            _ => return Err("Unsupported language".into()),
        };
        
        parser.set_language(language)?;
        Ok(parser)
    }
    
    /// Generate code based on a prompt
    pub async fn generate_code(&mut self, prompt: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        self.ai_client.generate_code(prompt, Some(language_str)).await
    }
    
    /// Complete code snippet
    pub async fn complete_code(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Complete the following {language_str} code:\n\n{code}\n\nContinue from where it left off. Provide only the completed code, no explanations."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Explain code
    pub async fn explain_code(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Explain the following {language_str} code:\n\n{code}\n\nProvide a clear, concise explanation of what the code does, how it works, and any important concepts or patterns used."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Optimize code
    pub async fn optimize_code(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Optimize the following {language_str} code for performance and readability:\n\n{code}\n\nProvide the optimized code along with explanations of the optimizations made."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Review code for issues
    pub async fn review_code(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Review the following {language_str} code for issues:\n\n{code}\n\nCheck for bugs, security vulnerabilities, performance issues, code smells, and violations of best practices. Provide a detailed report of any issues found and suggestions for improvement."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Generate tests for code
    pub async fn generate_tests(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Generate comprehensive tests for the following {language_str} code:\n\n{code}\n\nInclude unit tests, integration tests, and edge cases. Provide the test code along with explanations of what each test covers."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Refactor code
    pub async fn refactor_code(&mut self, code: &str, language: CodeLanguage, refactor_type: &str) -> Result<String, Box<dyn Error>> {
        let language_str: &str = language.into();
        let prompt = format!(
            "Refactor the following {language_str} code to {refactor_type}:\n\n{code}\n\nProvide the refactored code along with explanations of the changes made and how they address the refactoring goal."
        );
        
        let response = self.ai_client.generate_response(&prompt, None).await?;
        Ok(response.content().to_string())
    }
    
    /// Analyze codebase structure
    pub fn analyze_codebase(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        if !path.is_dir() {
            return Err("Path must be a directory".into());
        }
        
        let mut code_files = Vec::new();
        let mut language_counts = std::collections::HashMap::new();
        
        // Walk directory and collect code files
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext = ext.to_str().unwrap();
                    let language = CodeLanguage::from(ext);
                    
                    if language != CodeLanguage::Other {
                        code_files.push(path.to_path_buf());
                        *language_counts.entry(language).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Generate analysis report
        let mut report = format!("Codebase Analysis Report\n======================\n\n");
        report.push_str(&format!("Directory: {}\n", path.display()));
        report.push_str(&format!("Total Code Files: {}\n\n", code_files.len()));
        
        report.push_str("Language Distribution:\n");
        for (language, count) in language_counts.iter() {
            let language_str: &str = (*language).into();
            report.push_str(&format!("- {}: {}\n", language_str, count));
        }
        
        Ok(report)
    }
    
    /// Parse code and return syntax tree information
    pub fn parse_code(&mut self, code: &str, language: CodeLanguage) -> Result<String, Box<dyn Error>> {
        if let Some(parser) = self.parsers.get_mut(&language) {
            let tree = parser.parse(code, None)
                .ok_or("Failed to parse code")?;
            
            let root_node = tree.root_node();
            let start_pos = root_node.start_position();
            let end_pos = root_node.end_position();
            let mut report = format!("Syntax Tree Analysis\n==================\n\n");
            report.push_str(&format!("Root Node: {} [{}:{} - {}:{}]\n", 
                root_node.kind(),
                start_pos.row,
                start_pos.column,
                end_pos.row,
                end_pos.column
            ));
            
            // Count nodes by type
            let mut node_counts = std::collections::HashMap::new();
            let mut cursor = root_node.walk();
            Self::count_nodes(&mut cursor, &mut node_counts);
            
            report.push_str("\nNode Type Distribution:\n");
            for (node_type, count) in node_counts.iter() {
                report.push_str(&format!("- {}: {}\n", node_type, count));
            }
            
            Ok(report)
        } else {
            Err("No parser available for this language".into())
        }
    }
    
    /// Helper function to count nodes in a syntax tree
    fn count_nodes(cursor: &mut TreeCursor, counts: &mut std::collections::HashMap<String, usize>) {
        let node_type = cursor.node().kind().to_string();
        *counts.entry(node_type).or_insert(0) += 1;
        
        if cursor.goto_first_child() {
            loop {
                Self::count_nodes(cursor, counts);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
    
    /// Get code language from file path
    pub fn get_language_from_path(path: &Path) -> CodeLanguage {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return CodeLanguage::from(ext_str);
            }
        }
        CodeLanguage::Other
    }
    
    /// Read code from file
    pub fn read_code_from_file(path: &Path) -> Result<String, Box<dyn Error>> {
        fs::read_to_string(path)
            .map_err(|e| e.into())
    }
    
    /// Write code to file
    pub fn write_code_to_file(path: &Path, code: &str) -> Result<(), Box<dyn Error>> {
        fs::write(path, code)
            .map_err(|e| e.into())
    }
}

/// Code refactoring types
pub enum RefactorType {
    ExtractFunction,
    InlineFunction,
    ExtractVariable,
    RenameVariable,
    ExtractClass,
    ExtractInterface,
    ConvertToGeneric,
    ReplaceMagicNumbers,
    SimplifyConditionals,
    RemoveDuplication,
    Other(String),
}

impl From<&str> for RefactorType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extract_function" | "extract function" => RefactorType::ExtractFunction,
            "inline_function" | "inline function" => RefactorType::InlineFunction,
            "extract_variable" | "extract variable" => RefactorType::ExtractVariable,
            "rename_variable" | "rename variable" => RefactorType::RenameVariable,
            "extract_class" | "extract class" => RefactorType::ExtractClass,
            "extract_interface" | "extract interface" => RefactorType::ExtractInterface,
            "convert_to_generic" | "convert to generic" => RefactorType::ConvertToGeneric,
            "replace_magic_numbers" | "replace magic numbers" => RefactorType::ReplaceMagicNumbers,
            "simplify_conditionals" | "simplify conditionals" => RefactorType::SimplifyConditionals,
            "remove_duplication" | "remove duplication" => RefactorType::RemoveDuplication,
            _ => RefactorType::Other(s.to_string()),
        }
    }
}
