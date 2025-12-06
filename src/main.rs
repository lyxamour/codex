use clap::{Parser, Subcommand};
use log::info;
use std::env;

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
    command: Commands,

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Set log level based on verbose flag
    if cli.verbose {
        env::set_var("RUST_LOG", "debug");
    }

    info!("Starting Codex v{}", env!("CARGO_PKG_VERSION"));

    // Handle commands
    match cli.command {
        Commands::Interactive { tab } => {
            // Start interactive UI mode
            cli::handle_interactive(tab.clone())?;
        }
        Commands::Code {
            prompt,
            language,
            output,
        } => {
            // Handle code generation
            cli::handle_code(&prompt, language.clone(), output.clone())?;
        }
        Commands::Knowledge { action } => {
            // Handle knowledge base commands
            cli::handle_knowledge(action.clone())?;
        }
        Commands::Scrape {
            urls,
            depth,
            add_to_kb,
        } => {
            // Handle web scraping
            cli::handle_scrape(&urls, depth, add_to_kb).await?;
        }
        Commands::Task { action } => {
            // Handle task management
            cli::handle_task(action.clone())?;
        }
        Commands::Solo { task, steps } => {
            // Handle solo mode
            cli::handle_solo(&task, steps).await?;
        }
        Commands::Docs {
            path,
            format,
            output,
        } => {
            // Handle documentation generation
            cli::handle_docs(&path, &format, &output)?;
        }
    }

    Ok(())
}
