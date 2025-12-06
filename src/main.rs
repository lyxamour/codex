use clap::{Parser, Subcommand};
use log::info;
use std::env;
use std::io::{self, Write};

// Import core modules
mod config;
mod core;
mod error;
mod knowledge;
mod tools;

// Import existing modules
mod ai;
mod cli;
mod code;
mod command;
mod context;
mod docs;
mod hook;
mod knowledge_base;
mod scraper;
mod solo;
mod subagent;
mod task;
mod ui;

// Import error type and config
use crate::config::loader::ConfigLoader;
use crate::error::{init_error_reporting, AppResult};

// Import knowledge and task actions from their respective modules
use knowledge_base::KnowledgeActions;
use task::TaskActions;

/// A CLI-based AI programming tool with local knowledge base, remote scraping, and multi-AI platform support
#[derive(Parser, Debug)]
#[command(name = "codex")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A powerful AI programming assistant for the command line", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(long, value_name = "FILE")]
    config: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start interactive mode with modern UI
    Interactive {
        /// Start with a specific tab
        #[arg(short, long)]
        tab: Option<String>,
    },

    /// Generate code based on a prompt
    Code {
        /// Code generation prompt
        prompt: String,

        /// Programming language
        #[arg(short, long)]
        language: Option<String>,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Knowledge base management
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeActions,
    },

    /// Scrape remote websites for knowledge base
    Scrape {
        /// URLs to scrape
        urls: Vec<String>,

        /// Maximum depth for recursive scraping
        #[arg(short, long, default_value_t = 2)]
        depth: u32,

        /// Add to knowledge base
        #[arg(short, long)]
        add_to_kb: bool,
    },

    /// Task management
    Task {
        #[command(subcommand)]
        action: TaskActions,
    },

    /// Solo mode for autonomous AI tasks
    Solo {
        /// Task description
        task: String,

        /// Maximum steps
        #[arg(short, long, default_value_t = 10)]
        steps: u32,
    },

    /// Generate documentation for code
    Docs {
        /// Path to code files or directory
        path: String,

        /// Output format (markdown, html, etc.)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,
    },
}

/// 引导用户进行首次配置
fn guide_first_run(config_loader: &ConfigLoader) -> Result<(), Box<dyn std::error::Error>> {
    // 使用交互式配置向导
    config::wizard::run_wizard()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Initialize error reporting system
    crate::error::init_error_reporting();

    // Parse command line arguments
    let cli = Cli::parse();

    // Set log level based on verbose flag
    if cli.verbose {
        env::set_var("RUST_LOG", "debug");
    }

    info!("Starting Codex v{}", env!("CARGO_PKG_VERSION"));

    // Check if it's first run and guide user through configuration
    let config_loader = ConfigLoader::new();
    if config_loader.is_first_run(cli.config.as_deref()) {
        guide_first_run(&config_loader)?;
    }

    // Load configuration
    let config = config_loader.load(cli.config.as_deref())?;
    let language = &config.app.language;

    // Handle commands
    match cli.command {
        Some(Commands::Interactive { tab }) => {
            // Start interactive UI mode
            cli::handle_interactive(tab.clone())?;
        }
        Some(Commands::Code {
            prompt,
            language,
            output,
        }) => {
            // Handle code generation
            cli::handle_code(&prompt, language.clone(), output.clone())?;
        }
        Some(Commands::Knowledge { action }) => {
            // Handle knowledge base commands
            cli::handle_knowledge(format!("{:?}", action))?;
        }
        Some(Commands::Scrape {
            urls,
            depth,
            add_to_kb,
        }) => {
            // Handle web scraping
            cli::handle_scrape(&urls, depth, add_to_kb).await?;
        }
        Some(Commands::Task { action }) => {
            // Handle task management
            cli::handle_task(format!("{:?}", action))?;
        }
        Some(Commands::Solo { task, steps }) => {
            // Handle solo mode with explicit task
            cli::handle_solo(&task, steps).await?;
        }
        Some(Commands::Docs {
            path,
            format,
            output,
        }) => {
            // Handle documentation generation
            cli::handle_docs(&path, &format, &output)?;
        }
        None => {
            // Default: enter solo mode for AI programming
            if language == "zh" {
                println!("欢迎使用 Codex AI 编程助手");
                println!("正在进入 solo 模式进行 AI 编程...");
                let default_task = "我需要 AI 编程帮助。请指导我完成这个过程。";
                println!("\n开始 solo 模式，任务: {}", default_task);
                println!("最大步骤: 10");
                cli::handle_solo(default_task, 10).await?;
            } else {
                println!("Welcome to Codex AI Programming Assistant");
                println!("Entering solo mode for AI programming...");
                let default_task =
                    "I need help with AI programming. Please guide me through the process.";
                println!("\nStarting solo mode for task: {}", default_task);
                println!("Maximum steps: 10");
                cli::handle_solo(default_task, 10).await?;
            }
        }
    }

    Ok(())
}
