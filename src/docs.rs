use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Documentation generation format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocsFormat {
    Markdown,
    HTML,
    JSON,
    YAML,
    Custom(String),
}

impl From<DocsFormat> for String {
    fn from(format: DocsFormat) -> Self {
        match format {
            DocsFormat::Markdown => "markdown".to_string(),
            DocsFormat::HTML => "html".to_string(),
            DocsFormat::JSON => "json".to_string(),
            DocsFormat::YAML => "yaml".to_string(),
            DocsFormat::Custom(s) => s,
        }
    }
}

/// Documentation generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    /// Output format
    format: DocsFormat,
    /// Output directory
    output_dir: String,
    /// Whether to include private items
    include_private: bool,
    /// Whether to include tests
    include_tests: bool,
    /// Whether to include examples
    include_examples: bool,
    /// Whether to include dependencies
    include_dependencies: bool,
    /// Custom templates directory
    templates_dir: Option<String>,
    /// Additional configuration parameters
    config: HashMap<String, serde_json::Value>,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            format: DocsFormat::Markdown,
            output_dir: "./docs".to_string(),
            include_private: false,
            include_tests: false,
            include_examples: true,
            include_dependencies: false,
            templates_dir: None,
            config: HashMap::new(),
        }
    }
}

/// Documentation generator
pub struct DocsGenerator {
    config: DocsConfig,
    // Additional state for the generator
    generated_files: Vec<PathBuf>,
}

impl Default for DocsGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create docs generator")
    }
}

impl DocsGenerator {
    /// Create a new documentation generator
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: DocsConfig::default(),
            generated_files: Vec::new(),
        })
    }
    
    /// Create a new documentation generator with custom configuration
    pub fn with_config(config: DocsConfig) -> Self {
        Self {
            config,
            generated_files: Vec::new(),
        }
    }
    
    /// Generate documentation for a path
    pub fn generate(&mut self, path: &str, format: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
        // Update configuration
        self.config.output_dir = output_dir.to_string();
        self.config.format = match format.to_lowercase().as_str() {
            "markdown" | "md" => DocsFormat::Markdown,
            "html" => DocsFormat::HTML,
            "json" => DocsFormat::JSON,
            "yaml" | "yml" => DocsFormat::YAML,
            _ => return Err(format!("Unsupported format: {}", format).into()),
        };
        
        let path = Path::new(path);
        
        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;
        
        if path.is_dir() {
            // Generate docs for directory
            self.generate_dir_docs(path)?;
        } else if path.is_file() {
            // Generate docs for single file
            self.generate_file_docs(path)?;
        } else {
            return Err(format!("Path '{}' does not exist", path.display()).into());
        }
        
        println!("Documentation generated successfully!");
        println!("Output directory: {}", self.config.output_dir);
        println!("Generated {} files", self.generated_files.len());
        
        Ok(())
    }
    
    /// Generate documentation for a directory
    fn generate_dir_docs(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        println!("Generating documentation for directory: {}", path.display());
        
        // Walk directory and generate docs for each file
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                if self.should_process_file(file_path) {
                    self.generate_file_docs(file_path)?;
                }
            }
        }
        
        // Generate index file
        self.generate_index_file()?;
        
        Ok(())
    }
    
    /// Generate documentation for a single file
    fn generate_file_docs(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        println!("Generating documentation for file: {}", path.display());
        
        // Read file content
        let content = fs::read_to_string(path)?;
        
        // Determine file type
        let file_type = self.get_file_type(path);
        
        // Generate docs based on file type
        match file_type {
            FileType::Rust => {
                self.generate_rust_docs(path, &content)?;
            }
            FileType::Python => {
                self.generate_python_docs(path, &content)?;
            }
            FileType::JavaScript => {
                self.generate_javascript_docs(path, &content)?;
            }
            FileType::TypeScript => {
                self.generate_typescript_docs(path, &content)?;
            }
            FileType::Markdown => {
                self.generate_markdown_docs(path, &content)?;
            }
            FileType::Other => {
                self.generate_other_docs(path, &content)?;
            }
        }
        
        Ok(())
    }
    
    /// Check if a file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap_or("");
            match ext {
                // Supported file types
                "rs" | "py" | "js" | "ts" | "md" | "html" | "css" | "json" | "yaml" | "yml" | "toml" => {
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Get the file type from path
    fn get_file_type(&self, path: &Path) -> FileType {
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap_or("");
            match ext {
                "rs" => FileType::Rust,
                "py" => FileType::Python,
                "js" => FileType::JavaScript,
                "ts" => FileType::TypeScript,
                "md" => FileType::Markdown,
                _ => FileType::Other,
            }
        } else {
            FileType::Other
        }
    }
    
    /// Generate documentation for Rust files
    fn generate_rust_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        // For Rust files, we can use the tree-sitter parser to extract structure
        // This is a simplified implementation
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        let mut file = File::create(&output_path)?;
        
        // Write basic documentation
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** Rust")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content Preview")?;
        writeln!(&mut file, "```rust")?;
        writeln!(&mut file, "{}", content.lines().take(50).collect::<Vec<_>>().join("\n"))?;
        if content.lines().count() > 50 {
            writeln!(&mut file, "...")?;
        }
        writeln!(&mut file, "```")?;
        
        // Extract modules, structs, functions, etc. using tree-sitter (placeholder)
        self.extract_rust_structure(&mut file, content)?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Extract Rust structure using tree-sitter
    fn extract_rust_structure(&self, file: &mut File, _content: &str) -> Result<(), Box<dyn Error>> {
        // This is a placeholder implementation
        writeln!(file, "\n## Structure")?;
        writeln!(file, "\n### Modules")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        writeln!(file, "\n### Structs")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        writeln!(file, "\n### Functions")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        Ok(())
    }
    
    /// Generate documentation for Python files
    fn generate_python_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        let mut file = File::create(&output_path)?;
        
        // Write basic documentation
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** Python")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content Preview")?;
        writeln!(&mut file, "```python")?;
        writeln!(&mut file, "{}", content.lines().take(50).collect::<Vec<_>>().join("\n"))?;
        if content.lines().count() > 50 {
            writeln!(&mut file, "...")?;
        }
        writeln!(&mut file, "```")?;
        
        // Extract modules, classes, functions, etc. using tree-sitter (placeholder)
        self.extract_python_structure(&mut file, content)?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Extract Python structure
    fn extract_python_structure(&self, file: &mut File, _content: &str) -> Result<(), Box<dyn Error>> {
        // This is a placeholder implementation
        writeln!(file, "\n## Structure")?;
        writeln!(file, "\n### Classes")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        writeln!(file, "\n### Functions")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        Ok(())
    }
    
    /// Generate documentation for JavaScript files
    fn generate_javascript_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        let mut file = File::create(&output_path)?;
        
        // Write basic documentation
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** JavaScript")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content Preview")?;
        writeln!(&mut file, "```javascript")?;
        writeln!(&mut file, "{}", content.lines().take(50).collect::<Vec<_>>().join("\n"))?;
        if content.lines().count() > 50 {
            writeln!(&mut file, "...")?;
        }
        writeln!(&mut file, "```")?;
        
        // Extract structure (placeholder)
        self.extract_javascript_structure(&mut file, content)?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Extract JavaScript structure
    fn extract_javascript_structure(&self, file: &mut File, _content: &str) -> Result<(), Box<dyn Error>> {
        // This is a placeholder implementation
        writeln!(file, "\n## Structure")?;
        writeln!(file, "\n### Classes")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        writeln!(file, "\n### Functions")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        Ok(())
    }
    
    /// Generate documentation for TypeScript files
    fn generate_typescript_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        let mut file = File::create(&output_path)?;
        
        // Write basic documentation
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** TypeScript")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content Preview")?;
        writeln!(&mut file, "```typescript")?;
        writeln!(&mut file, "{}", content.lines().take(50).collect::<Vec<_>>().join("\n"))?;
        if content.lines().count() > 50 {
            writeln!(&mut file, "...")?;
        }
        writeln!(&mut file, "```")?;
        
        // Extract structure (placeholder)
        self.extract_typescript_structure(&mut file, content)?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Extract TypeScript structure
    fn extract_typescript_structure(&self, file: &mut File, _content: &str) -> Result<(), Box<dyn Error>> {
        // This is a placeholder implementation
        writeln!(file, "\n## Structure")?;
        writeln!(file, "\n### Interfaces")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        writeln!(file, "\n### Classes")?;
        writeln!(file, "- None detected (placeholder)")?;
        
        Ok(())
    }
    
    /// Generate documentation for Markdown files
    fn generate_markdown_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        // For Markdown files, we can just copy them to the output directory
        // with some additional metadata
        let mut file = File::create(&output_path)?;
        
        // Write metadata header
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** Markdown")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content")?;
        writeln!(&mut file, "{}", content)?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Generate documentation for other file types
    fn generate_other_docs(&mut self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = Path::new(&self.config.output_dir)
            .join(format!("{}.md", file_name));
        
        let mut file = File::create(&output_path)?;
        
        // Write basic documentation
        writeln!(&mut file, "# Documentation for {}", file_name)?;
        writeln!(&mut file, "\n**Path:** {}", path.display())?;
        writeln!(&mut file, "**Type:** Other")?;
        writeln!(&mut file, "**Lines:** {}", content.lines().count())?;
        writeln!(&mut file, "\n## Content Preview")?;
        writeln!(&mut file, "```")?;
        writeln!(&mut file, "{}", content.lines().take(50).collect::<Vec<_>>().join("\n"))?;
        if content.lines().count() > 50 {
            writeln!(&mut file, "...")?;
        }
        writeln!(&mut file, "```")?;
        
        // Add to generated files
        self.generated_files.push(output_path);
        
        Ok(())
    }
    
    /// Generate index file
    fn generate_index_file(&mut self) -> Result<(), Box<dyn Error>> {
        let index_path = Path::new(&self.config.output_dir)
            .join("index.md");
        
        let mut file = File::create(&index_path)?;
        
        writeln!(&mut file, "# Documentation Index")?;
        writeln!(&mut file, "\nGenerated by Codex Documentation Generator")?;
        writeln!(&mut file, "\n## Files")?;
        
        for generated_file in &self.generated_files {
            let file_name = generated_file.file_name().unwrap().to_str().unwrap();
            writeln!(&mut file, "- [{file_name}]({file_name})")?;
        }
        
        // Add to generated files
        self.generated_files.push(index_path);
        
        Ok(())
    }
    
    /// Get the generated files
    pub fn generated_files(&self) -> &Vec<PathBuf> {
        &self.generated_files
    }
    
    /// Clear the generated files list
    pub fn clear_generated_files(&mut self) {
        self.generated_files.clear();
    }
    
    /// Set the configuration
    pub fn set_config(&mut self, config: DocsConfig) {
        self.config = config;
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DocsConfig {
        &self.config
    }
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Markdown,
    Other,
}

/// Documentation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsSummary {
    /// Total files processed
    pub total_files: usize,
    /// Files by type
    pub files_by_type: HashMap<String, usize>,
    /// Generated files
    pub generated_files: usize,
    /// Output directory
    pub output_dir: String,
    /// Output format
    pub format: String,
    /// Generation time in seconds
    pub generation_time: f64,
}
