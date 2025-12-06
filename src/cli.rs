use crate::knowledge_base::KnowledgeActions;
use crate::task::TaskActions;
use crate::{ai, docs, knowledge_base, scraper, solo, task, ui};
use std::error::Error;

/// Handle interactive mode command
pub fn handle_interactive(tab: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("Starting interactive mode...");
    println!("Tab: {:?}", tab);

    // Initialize UI
    ui::run(tab)?;

    Ok(())
}

/// Handle code generation command
pub fn handle_code(
    prompt: &str,
    language: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("Generating code for prompt: {}", prompt);
    println!("Language: {:?}", language);
    println!("Output: {:?}", output);

    // Initialize tokio runtime to execute async code
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move {
        // Initialize AI client
        let mut ai_client = ai::AIClient::new()?;

        // Generate code
        let generated_code = ai_client.generate_code(prompt, language.as_deref()).await?;

        // Output result
        if let Some(output_path) = output {
            std::fs::write(&output_path, generated_code)?;
            println!("Code generated and saved to {}", output_path);
        } else {
            println!("{}", generated_code);
        }

        Ok(())
    })
}

/// Handle knowledge base commands
pub fn handle_knowledge(action: KnowledgeActions) -> Result<(), Box<dyn Error>> {
    let mut kb = knowledge_base::KnowledgeBase::new()?;

    match action {
        KnowledgeActions::Add { paths, recursive } => {
            kb.add_files(paths, recursive)?;
            println!("Added files to knowledge base");
        }
        KnowledgeActions::Search { query, limit } => {
            let results = kb.search(&query, limit)?;
            println!("Found {} results for query: {}", results.len(), query);
            for result in results {
                println!("- {}", result);
            }
        }
        KnowledgeActions::List { details } => {
            let items = kb.list(details)?;
            println!("Knowledge base contents:");
            for item in items {
                println!("- {}", item);
            }
        }
        KnowledgeActions::Clear { confirm } => {
            if confirm {
                kb.clear()?;
                println!("Knowledge base cleared");
            } else {
                println!("Clear operation cancelled. Use --confirm to proceed.");
            }
        }
    }

    Ok(())
}

/// Handle web scraping command
pub async fn handle_scrape(
    urls: &[String],
    depth: u32,
    add_to_kb: bool,
) -> Result<(), Box<dyn Error>> {
    println!("Scraping URLs: {:?}", urls);
    println!("Depth: {}", depth);
    println!("Add to KB: {}", add_to_kb);

    let mut scraper = scraper::WebScraper::new(depth)?;

    // Scrape URLs
    let scraped_content = scraper.scrape(urls).await?;

    // Add to knowledge base if requested
    if add_to_kb {
        let mut kb = knowledge_base::KnowledgeBase::new()?;
        for (url, content) in scraped_content {
            kb.add_scraped_content(&url, &content)?;
        }
        println!("Scraped content added to knowledge base");
    } else {
        // Print scraped content
        for (url, content) in scraped_content {
            println!("\n=== {} ===", url);
            println!("{}", content);
        }
    }

    Ok(())
}

/// Handle task management commands
pub fn handle_task(action: TaskActions) -> Result<(), Box<dyn Error>> {
    let mut task_manager = task::TaskManager::new()?;

    match action {
        TaskActions::Create {
            description,
            priority,
        } => {
            let task = task_manager.create(description, priority)?;
            println!("Created task: {}", task.id);
        }
        TaskActions::List { status } => {
            let tasks = task_manager.list(status.as_deref())?;
            println!("Tasks:");
            for task in tasks {
                println!("- {}: {} [{:?}]", task.id, task.description, task.status);
            }
        }
        TaskActions::Update {
            id,
            description,
            status,
            priority,
        } => {
            let updated_task = task_manager.update(&id, description, status, priority)?;
            println!("Updated task: {}", updated_task.id);
        }
        TaskActions::Delete { id } => {
            task_manager.delete(&id)?;
            println!("Deleted task: {}", id);
        }
    }

    Ok(())
}

/// Handle solo mode command
pub async fn handle_solo(task: &str, steps: u32) -> Result<(), Box<dyn Error>> {
    println!("Starting solo mode for task: {}", task);
    println!("Maximum steps: {}", steps);

    let mut solo_agent = solo::SoloAgent::new()?;
    let result = solo_agent.execute(task, steps).await?;

    println!("\nSolo mode completed:");
    println!("Result: {}", result);

    Ok(())
}

/// Handle documentation generation command
pub fn handle_docs(path: &str, format: &str, output: &str) -> Result<(), Box<dyn Error>> {
    println!("Generating documentation for: {}", path);
    println!("Format: {}", format);
    println!("Output directory: {}", output);

    let mut docs_generator = docs::DocsGenerator::new()?;
    docs_generator.generate(path, format, output)?;

    println!("Documentation generated successfully");

    Ok(())
}
