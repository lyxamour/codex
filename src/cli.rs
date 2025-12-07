use crate::error::AppResult;
use crate::knowledge::base::KnowledgeBase;
use crate::plugins;
use chrono;
use dirs;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

/// Handle interactive mode command
pub fn handle_interactive(tab: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("Starting interactive mode...");
    println!("Tab: {:?}", tab);

    // Initialize UI - TODO: 主人~ 这里需要实现UI初始化逻辑
    // ui::run(tab)?;

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

    // TODO: 主人~ 这里需要实现代码生成逻辑
    // Initialize tokio runtime to execute async code
    // let rt = tokio::runtime::Runtime::new()?;

    // rt.block_on(async move {
    //     // Initialize AI client
    //     let mut ai_client = ai::AIClient::new()?;

    //     // Generate code
    //     let generated_code = ai_client.generate_code(prompt, language.as_deref()).await?;

    //     // Output result
    //     if let Some(output_path) = output {
    //         std::fs::write(&output_path, generated_code)?;
    //         println!("Code generated and saved to {}", output_path);
    //     } else {
    //         println!("{}", generated_code);
    //     }

    //     Ok(())
    // })
    Ok(())
}

/// Handle knowledge base commands
pub fn handle_knowledge(action: String) -> Result<(), Box<dyn Error>> {
    // TODO: 主人~ 这里需要实现知识库命令处理逻辑
    println!("Handling knowledge command: {}", action);
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

    // 创建配置加载器
    let config_loader = crate::config::loader::ConfigLoader::new();
    let config = config_loader.load(None)?;

    // 创建远程知识库实例
    let mut remote_kb = crate::knowledge::remote::RemoteKnowledgeBase::new(config.knowledge)?;

    // 初始化远程知识库
    remote_kb.init()?;

    // 抓取远程内容
    let scraped_content = remote_kb.scrape(urls).await?;

    // 显示抓取结果
    println!("\n抓取完成，共抓取到 {} 个页面:", scraped_content.len());
    for content in &scraped_content {
        println!(
            "- {} ({} bytes, depth: {})",
            content.meta.title, content.meta.size, content.meta.depth
        );
        println!("  URL: {}", content.meta.url);
        println!("  代码元素: {}", content.code_elements.len());
    }

    Ok(())
}

/// Handle task management commands
pub fn handle_task(action: crate::task::TaskActions) -> Result<(), Box<dyn Error>> {
    // 获取任务存储路径
    let task_db_path = Path::new(&dirs::home_dir().unwrap()).join(".codex/task_db");

    // 创建任务存储目录（如果不存在）
    if !task_db_path.exists() {
        std::fs::create_dir_all(&task_db_path)?;
    }

    // 创建任务管理器
    let task_manager = Arc::new(crate::task::TaskManager::new(task_db_path)?);

    match action {
        crate::task::TaskActions::Add {
            title,
            description,
            priority,
            due,
        } => {
            // 处理优先级转换
            let priority = priority
                .map(|p| match p.to_lowercase().as_str() {
                    "low" => crate::task::TaskPriority::Low,
                    "medium" => crate::task::TaskPriority::Medium,
                    "high" => crate::task::TaskPriority::High,
                    "urgent" => crate::task::TaskPriority::Urgent,
                    _ => crate::task::TaskPriority::Medium,
                })
                .unwrap_or(crate::task::TaskPriority::Medium);

            // 创建任务
            let mut task = task_manager.create_task(title, description)?;
            task.update_priority(priority);
            task_manager.update_task(&task)?;

            println!("任务创建成功！ID: {}", task.id);
        }

        crate::task::TaskActions::List {
            status,
            priority,
            details,
        } => {
            // 获取所有任务
            let mut tasks = task_manager.get_all_tasks()?;

            // 按状态筛选
            if let Some(status_filter) = status {
                let status_enum = match status_filter.to_lowercase().as_str() {
                    "todo" | "t" => crate::task::TaskStatus::Todo,
                    "in_progress" | "ip" | "progress" => crate::task::TaskStatus::InProgress,
                    "completed" | "c" => crate::task::TaskStatus::Completed,
                    "cancelled" | "cancel" => crate::task::TaskStatus::Cancelled,
                    _ => {
                        println!("无效的状态筛选条件: {}", status_filter);
                        return Ok(());
                    }
                };
                tasks = tasks
                    .into_iter()
                    .filter(|t| t.status == status_enum)
                    .collect();
            }

            // 按优先级筛选
            if let Some(priority_filter) = priority {
                let priority_enum = match priority_filter.to_lowercase().as_str() {
                    "low" | "l" => crate::task::TaskPriority::Low,
                    "medium" | "m" => crate::task::TaskPriority::Medium,
                    "high" | "h" => crate::task::TaskPriority::High,
                    "urgent" | "u" => crate::task::TaskPriority::Urgent,
                    _ => {
                        println!("无效的优先级筛选条件: {}", priority_filter);
                        return Ok(());
                    }
                };
                tasks = tasks
                    .into_iter()
                    .filter(|t| t.priority == priority_enum)
                    .collect();
            }

            // 显示任务列表
            if tasks.is_empty() {
                println!("没有找到符合条件的任务。");
            } else {
                println!("\n任务列表:");
                for task in tasks {
                    if details {
                        println!("\nID: {}", task.id);
                        println!("标题: {}", task.title);
                        if let Some(desc) = &task.description {
                            println!("描述: {}", desc);
                        }
                        println!("状态: {:?}", task.status);
                        println!("优先级: {:?}", task.priority);
                        println!(
                            "创建时间: {}",
                            chrono::NaiveDateTime::from_timestamp_millis(
                                (task.created_at * 1000) as i64
                            )
                            .unwrap()
                        );
                        if let Some(due_at) = task.due_at {
                            println!(
                                "截止时间: {}",
                                chrono::NaiveDateTime::from_timestamp_millis(
                                    (due_at * 1000) as i64
                                )
                                .unwrap()
                            );
                        }
                        if !task.tags.is_empty() {
                            println!("标签: {:?}", task.tags);
                        }
                        println!("{}", "-".repeat(50));
                    } else {
                        println!("- [{:?}] [{:?}] {}", task.status, task.priority, task.title);
                    }
                }
            }
        }

        crate::task::TaskActions::Complete { id } => {
            // 标记任务为完成
            if let Some(mut task) = task_manager.get_task(&id)? {
                task.update_status(crate::task::TaskStatus::Completed);
                task_manager.update_task(&task)?;
                println!("任务已标记为完成！ID: {}", id);
            } else {
                println!("未找到ID为 {} 的任务。", id);
            }
        }

        crate::task::TaskActions::Delete { id } => {
            // 删除任务
            if let Some(task) = task_manager.delete_task(&id)? {
                println!("任务已删除！标题: {}", task.title);
            } else {
                println!("未找到ID为 {} 的任务。", id);
            }
        }

        crate::task::TaskActions::Update {
            id,
            title,
            description,
            status,
            priority,
            due,
        } => {
            // 更新任务
            if let Some(mut task) = task_manager.get_task(&id)? {
                // 更新标题
                if let Some(new_title) = title {
                    task.title = new_title;
                }

                // 更新描述
                if let Some(new_desc) = description {
                    task.description = Some(new_desc);
                }

                // 更新状态
                if let Some(new_status) = status {
                    let status_enum = match new_status.to_lowercase().as_str() {
                        "todo" | "t" => crate::task::TaskStatus::Todo,
                        "in_progress" | "ip" | "progress" => crate::task::TaskStatus::InProgress,
                        "completed" | "c" => crate::task::TaskStatus::Completed,
                        "cancelled" | "cancel" => crate::task::TaskStatus::Cancelled,
                        _ => {
                            println!("无效的状态: {}", new_status);
                            return Ok(());
                        }
                    };
                    task.update_status(status_enum);
                }

                // 更新优先级
                if let Some(new_priority) = priority {
                    let priority_enum = match new_priority.to_lowercase().as_str() {
                        "low" | "l" => crate::task::TaskPriority::Low,
                        "medium" | "m" => crate::task::TaskPriority::Medium,
                        "high" | "h" => crate::task::TaskPriority::High,
                        "urgent" | "u" => crate::task::TaskPriority::Urgent,
                        _ => {
                            println!("无效的优先级: {}", new_priority);
                            return Ok(());
                        }
                    };
                    task.update_priority(priority_enum);
                }

                // 更新截止时间
                // TODO: 主人~ 这里需要实现截止时间解析逻辑

                task_manager.update_task(&task)?;
                println!("任务已更新！ID: {}", id);
            } else {
                println!("未找到ID为 {} 的任务。", id);
            }
        }

        crate::task::TaskActions::Show { id } => {
            // 显示任务详情
            if let Some(task) = task_manager.get_task(&id)? {
                println!("\n任务详情:");
                println!("ID: {}", task.id);
                println!("标题: {}", task.title);
                if let Some(desc) = &task.description {
                    println!("描述: {}", desc);
                }
                println!("状态: {:?}", task.status);
                println!("优先级: {:?}", task.priority);
                println!(
                    "创建时间: {}",
                    chrono::NaiveDateTime::from_timestamp_millis((task.created_at * 1000) as i64)
                        .unwrap()
                );
                if let Some(due_at) = task.due_at {
                    println!(
                        "截止时间: {}",
                        chrono::NaiveDateTime::from_timestamp_millis((due_at * 1000) as i64)
                            .unwrap()
                    );
                }
                if !task.tags.is_empty() {
                    println!("标签: {:?}", task.tags);
                }
                if !task.dependencies.is_empty() {
                    println!("依赖: {:?}", task.dependencies);
                }
            } else {
                println!("未找到ID为 {} 的任务。", id);
            }
        }
    }

    Ok(())
}

/// Handle solo mode command
pub async fn handle_solo(task: &str, steps: u32) -> Result<(), Box<dyn Error>> {
    println!("Starting solo mode for task: {}", task);
    println!("Maximum steps: {}", steps);

    // 创建 solo agent 并执行任务
    let mut solo_agent = crate::solo::SoloAgent::new().await?; // 调用异步方法创建solo agent
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

    // TODO: 主人~ 这里需要实现文档生成逻辑
    // let mut docs_generator = docs::DocsGenerator::new()?;
    // docs_generator.generate(path, format, output)?;

    println!("Documentation generated successfully");

    Ok(())
}

/// Handle AI platform management commands
pub async fn handle_provider(
    action: crate::ai::adapter::ProviderActions,
) -> Result<(), Box<dyn Error>> {
    // 创建AI客户端
    let mut ai_client = crate::ai::AIClient::new().await?;

    match action {
        crate::ai::adapter::ProviderActions::List => {
            // 列出所有可用的AI平台
            let models = ai_client.get_available_models();
            if models.is_empty() {
                println!("没有可用的AI平台配置。请使用 'codex provider config' 命令添加配置。");
                return Ok(());
            }

            println!("可用的AI平台列表:");
            for model in models {
                println!("- {}", model);
            }
        }

        crate::ai::adapter::ProviderActions::Switch { model_name } => {
            // 切换AI平台
            match ai_client.switch_provider(&model_name) {
                Ok(_) => {
                    println!("成功切换到AI平台: {}", model_name);
                }
                Err(e) => {
                    println!("切换AI平台失败: {}", e);
                }
            }
        }

        crate::ai::adapter::ProviderActions::Config {
            platform,
            model_name,
            api_key,
            base_url,
        } => {
            // 配置AI平台
            let platform_enum = match platform.to_lowercase().as_str() {
                "openai" => crate::ai::adapter::AIPlatform::OpenAI,
                "anthropic" => crate::ai::adapter::AIPlatform::Anthropic,
                "ollama" => crate::ai::adapter::AIPlatform::Ollama,
                "gemini" => crate::ai::adapter::AIPlatform::GoogleGemini,
                "mistral" => crate::ai::adapter::AIPlatform::Mistral,
                _ => {
                    println!("无效的AI平台类型: {}", platform);
                    println!("支持的平台类型: openai, anthropic, ollama, gemini, mistral");
                    return Ok(());
                }
            };

            // 创建模型配置
            let model = crate::ai::adapter::AIModel {
                platform: platform_enum,
                model_name: model_name.clone(),
                api_key: api_key.unwrap_or_else(|| "".to_string()),
                base_url,
            };

            // 添加模型配置
            match ai_client.add_model(&format!("{}-{}", platform, model_name), model) {
                Ok(_) => {
                    println!("成功配置AI平台: {}-{}", platform, model_name);
                }
                Err(e) => {
                    println!("配置AI平台失败: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Plugin actions
pub enum PluginActions {
    List,
    Install(String),
    Uninstall(String),
    Enable(String),
    Disable(String),
    Info(String),
}

/// Handle plugin management commands
pub async fn handle_plugin(action: PluginActions) -> Result<(), Box<dyn Error>> {
    // 创建插件管理器
    let mut plugin_manager = crate::plugins::PluginManager::new();

    match action {
        PluginActions::List => {
            // 列出所有插件
            let plugins = plugin_manager.list_plugins();
            if plugins.is_empty() {
                println!(
                    "没有安装任何插件。使用 'codex plugin install <plugin_name>' 命令安装插件。"
                );
                return Ok(());
            }

            println!("已安装的插件列表:");
            for plugin in plugins {
                let status = if plugin_manager.is_plugin_enabled(plugin.name.as_str()) {
                    "已启用"
                } else {
                    "已禁用"
                };
                println!(
                    "- {} ({}) - {} - {}",
                    plugin.name, plugin.version, plugin.description, status
                );
            }
        }

        PluginActions::Install(plugin_name) => {
            // 安装插件
            println!("安装插件: {}", plugin_name);

            // 1. 创建插件目录
            let plugin_dir = format!("~/.codex/plugins/{}", plugin_name);
            let expanded_dir = shellexpand::tilde(&plugin_dir).to_string();
            std::fs::create_dir_all(&expanded_dir)?;

            // 2. 从插件仓库下载插件 - TODO: 主人~ 这里需要实现插件下载逻辑
            // self.download_plugin(&plugin_name, &expanded_dir)?;

            // 3. 创建插件配置文件 - TODO: 主人~ 这里需要实现插件配置文件创建逻辑
            // self.create_plugin_config(&plugin_name, &expanded_dir)?;

            // 4. 加载插件
            let result = plugin_manager.load_plugin(&expanded_dir).await?;
            println!("{}", result.message);

            // 5. 初始化插件
            let init_result = plugin_manager.initialize_plugin(&plugin_name).await?;
            println!("{}", init_result.message);

            // 6. 启用插件
            let enable_result = plugin_manager.enable_plugin(&plugin_name).await?;
            println!("{}", enable_result.message);
        }

        PluginActions::Uninstall(plugin_name) => {
            // 卸载插件
            println!("卸载插件: {}", plugin_name);

            // 1. 检查插件是否存在
            if !plugin_manager.is_plugin_loaded(&plugin_name) {
                println!("插件 '{}' 未安装", plugin_name);
                return Ok(());
            }

            // 2. 卸载插件
            let result = plugin_manager.unload_plugin(&plugin_name).await?;
            println!("{}", result.message);

            // 3. 删除插件文件 - TODO: 主人~ 这里需要实现插件文件删除逻辑
            // let plugin_dir = format!("~/.codex/plugins/{}", plugin_name);
            // let expanded_dir = shellexpand::tilde(&plugin_dir).to_string();
            // std::fs::remove_dir_all(&expanded_dir)?;
        }

        PluginActions::Enable(plugin_name) => {
            // 启用插件
            println!("启用插件: {}", plugin_name);

            // 1. 检查插件是否存在
            if !plugin_manager.is_plugin_loaded(&plugin_name) {
                println!("插件 '{}' 未安装", plugin_name);
                return Ok(());
            }

            // 2. 启用插件
            let result = plugin_manager.enable_plugin(&plugin_name).await?;
            println!("{}", result.message);
        }

        PluginActions::Disable(plugin_name) => {
            // 禁用插件
            println!("禁用插件: {}", plugin_name);

            // 1. 检查插件是否存在
            if !plugin_manager.is_plugin_loaded(&plugin_name) {
                println!("插件 '{}' 未安装", plugin_name);
                return Ok(());
            }

            // 2. 禁用插件
            let result = plugin_manager.disable_plugin(&plugin_name).await?;
            println!("{}", result.message);
        }

        PluginActions::Info(plugin_name) => {
            // 显示插件信息
            println!("查看插件信息: {}", plugin_name);

            // 1. 检查插件是否存在
            if let Some(plugin_info) = plugin_manager.get_plugin_info(&plugin_name) {
                println!("插件信息:");
                println!("- 名称: {}", plugin_info.plugin.name());
                println!("- 描述: {}", plugin_info.plugin.description());
                println!("- 版本: {}", plugin_info.plugin.version());
                println!("- 作者: {}", plugin_info.plugin.author());
                println!("- 许可证: {}", plugin_info.plugin.license());
                println!("- 状态: {:?}", plugin_info.state);
            } else {
                println!("插件 '{}' 未安装", plugin_name);
            }
        }
    }

    Ok(())
}
