//! 交互式配置向导
//! 
//! 提供基于ratatui的交互式配置向导，用于首次启动时引导用户配置应用

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, execute, terminal};
use ratatui::{prelude::*, widgets::*};
use std::error::Error;
use std::io::{stdout, Stdout};
use std::time::Duration;

use super::{app::AppConfig, loader::ConfigLoader};

/// 配置向导状态
enum WizardStep {
    Welcome,
    LanguageSelection,
    AISettings,
    UISettings,
    Summary,
    Complete,
}

/// 向导应用状态
struct WizardApp {
    /// 当前步骤
    step: WizardStep,
    /// 配置
    config: AppConfig,
    /// 选择索引
    selection: usize,
    /// 输入缓冲区
    input: String,
    /// 是否正在输入
    is_input_mode: bool,
    /// 当前输入字段
    current_field: Option<String>,
}

impl WizardApp {
    /// 创建新的向导应用
    fn new() -> Self {
        let config_loader = ConfigLoader::new();
        let default_config = config_loader.get_default_config();
        
        Self {
            step: WizardStep::Welcome,
            config: default_config,
            selection: 0,
            input: String::new(),
            is_input_mode: false,
            current_field: None,
        }
    }
}

/// 运行配置向导
pub fn run_wizard() -> Result<(), Box<dyn Error>> {
    // 初始化终端
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建向导应用
    let mut app = WizardApp::new();

    // 运行主循环
    loop {
        // 绘制UI
        terminal.draw(|f| render_wizard(f, &app))?;

        // 处理事件
        if !handle_wizard_events(&mut app)? {
            break;
        }
    }

    // 保存配置
    let config_loader = ConfigLoader::new();
    config_loader.save(&app.config, None)?;

    // 恢复终端
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// 渲染向导UI
fn render_wizard(f: &mut Frame, app: &WizardApp) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题
            Constraint::Min(0),    // 内容
            Constraint::Length(3), // 状态/输入
        ])
        .split(f.size());

    // 渲染标题
    render_title(f, layout[0], app);
    
    // 渲染内容
    render_content(f, layout[1], app);
    
    // 渲染状态/输入
    render_input(f, layout[2], app);
}

/// 渲染标题
fn render_title(f: &mut Frame, area: Rect, app: &WizardApp) {
    let title = match &app.step {
        WizardStep::Welcome => "Welcome to Codex!",
        WizardStep::LanguageSelection => "Language Selection",
        WizardStep::AISettings => "AI Settings",
        WizardStep::UISettings => "UI Settings",
        WizardStep::Summary => "Configuration Summary",
        WizardStep::Complete => "Configuration Complete",
    };
    
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(title_widget, area);
}

/// 渲染内容
fn render_content(f: &mut Frame, area: Rect, app: &WizardApp) {
    match &app.step {
        WizardStep::Welcome => render_welcome(f, area),
        WizardStep::LanguageSelection => render_language_selection(f, area, app),
        WizardStep::AISettings => render_ai_settings(f, area, app),
        WizardStep::UISettings => render_ui_settings(f, area, app),
        WizardStep::Summary => render_summary(f, area, app),
        WizardStep::Complete => render_complete(f, area),
    }
}

/// 渲染欢迎页面
fn render_welcome(f: &mut Frame, area: Rect) {
    let welcome_text = vec![
        Line::from("🎉 Welcome to Codex AI Programming Assistant!"),
        Line::from(""),
        Line::from("Codex is a powerful CLI-based AI programming tool that helps you write better code faster."),
        Line::from(""),
        Line::from("This wizard will guide you through the initial configuration."),
        Line::from(""),
        Line::from("Press Enter to continue..."),
    ];
    
    let welcome_widget = Paragraph::new(welcome_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(welcome_widget, area);
}

/// 渲染语言选择页面
fn render_language_selection(f: &mut Frame, area: Rect, app: &WizardApp) {
    let languages = vec!["English", "中文"];
    
    let items = languages
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let style = if i == app.selection {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("  {}", lang), style))
        })
        .collect::<Vec<_>>();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select Interface Language"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("> ");
    
    f.render_widget(list, area);
}

/// 渲染AI设置页面
fn render_ai_settings(f: &mut Frame, area: Rect, app: &WizardApp) {
    let items = vec![
        format!("OpenAI API Key: {}", 
            if let Some(openai) = &app.config.ai.openai {
                if openai.api_key.is_empty() {
                    "[Enter API Key]"
                } else {
                    "••••••••" // 隐藏API密钥
                }
            } else {
                "[Not configured]"
            }
        ),
        format!("Default Model: {}", 
            if let Some(openai) = &app.config.ai.openai {
                &openai.default_model
            } else {
                "gpt-4o"
            }
        ),
    ];
    
    let items = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selection {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("  {}", item), style))
        })
        .collect::<Vec<_>>();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("AI Settings"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("> ");
    
    f.render_widget(list, area);
}

/// 渲染UI设置页面
fn render_ui_settings(f: &mut Frame, area: Rect, app: &WizardApp) {
    let items = vec![
        format!("Colored Output: {}", if app.config.ui.colored { "Yes" } else { "No" }),
        format!("Theme: {}", &app.config.ui.theme),
    ];
    
    let items = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selection {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("  {}", item), style))
        })
        .collect::<Vec<_>>();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("UI Settings"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("> ");
    
    f.render_widget(list, area);
}

/// 渲染配置摘要页面
fn render_summary(f: &mut Frame, area: Rect, app: &WizardApp) {
    let summary = vec![
        Line::from("Configuration Summary:"),
        Line::from(""),
        Line::from(format!("• Language: {}", 
            if app.config.app.language == "zh" { "中文" } else { "English" }
        )),
        Line::from(format!("• AI Platform: {}", &app.config.ai.default_platform)),
        Line::from(format!("• OpenAI API Key: {}", 
            if let Some(openai) = &app.config.ai.openai {
                if openai.api_key.is_empty() { "Not configured" } else { "Configured" }
            } else { "Not configured" }
        )),
        Line::from(format!("• Default Model: {}", 
            if let Some(openai) = &app.config.ai.openai {
                &openai.default_model
            } else { "gpt-4o" }
        )),
        Line::from(format!("• Colored Output: {}", if app.config.ui.colored { "Yes" } else { "No" })),
        Line::from(format!("• Theme: {}", &app.config.ui.theme)),
        Line::from(""),
        Line::from("Press Enter to save configuration..."),
    ];
    
    let summary_widget = Paragraph::new(summary)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Summary"));
    
    f.render_widget(summary_widget, area);
}

/// 渲染完成页面
fn render_complete(f: &mut Frame, area: Rect) {
    let complete_text = vec![
        Line::from("✅ Configuration Complete!"),
        Line::from(""),
        Line::from("Your configuration has been saved successfully."),
        Line::from(""),
        Line::from("You can always update your configuration later by editing:"),
        Line::from("~/.codex/config.yaml"),
        Line::from(""),
        Line::from("Press Enter to start Codex..."),
    ];
    
    let complete_widget = Paragraph::new(complete_text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(complete_widget, area);
}

/// 渲染输入区域
fn render_input(f: &mut Frame, area: Rect, app: &WizardApp) {
    let mut content = vec![];
    
    match &app.step {
        WizardStep::Welcome => {
            content.push(Line::from("Use Enter to continue, Esc to exit"));
        }
        WizardStep::LanguageSelection => {
            content.push(Line::from("Use ↑/↓ to navigate, Enter to select, Esc to exit"));
        }
        WizardStep::AISettings => {
            if app.is_input_mode {
                content.push(Line::from(format!("Enter value: {}", &app.input)));
                content.push(Line::from("Press Enter to save, Esc to cancel"));
            } else {
                content.push(Line::from("Use ↑/↓ to navigate, Enter to edit, Esc to exit"));
            }
        }
        WizardStep::UISettings => {
            content.push(Line::from("Use ↑/↓ to navigate, Enter to toggle, Esc to exit"));
        }
        WizardStep::Summary => {
            content.push(Line::from("Press Enter to save, Esc to exit"));
        }
        WizardStep::Complete => {
            content.push(Line::from("Press Enter to continue, Esc to exit"));
        }
    }
    
    let input_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(input_widget, area);
    
    // 设置光标位置
    if app.is_input_mode {
        f.set_cursor(
            area.x + 14 + app.input.len() as u16, // 14 is the length of "Enter value: "
            area.y + 1
        );
    }
}

/// 处理向导事件
fn handle_wizard_events(app: &mut WizardApp) -> Result<bool, Box<dyn Error>> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key_event) => {
                return handle_wizard_key_event(app, key_event);
            }
            Event::Mouse(_mouse_event) => {
                // 暂不处理鼠标事件
            }
            Event::Resize(_, _) => {
                // 终端大小变化，会在下一次绘制时处理
            }
            Event::FocusGained | Event::FocusLost | Event::Paste(_) => {
                // 忽略这些事件
            }
        }
    }
    
    Ok(true)
}

/// 处理向导按键事件
fn handle_wizard_key_event(app: &mut WizardApp, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
    match key_event.code {
        // 退出向导
        KeyCode::Esc => {
            return Ok(false);
        }
        
        // 输入模式下的处理
        _ if app.is_input_mode => {
            return handle_input_mode_key_event(app, key_event);
        }
        
        // 普通模式下的处理
        KeyCode::Enter => {
            handle_enter_key(app);
        }
        KeyCode::Up => {
            if let Some(selection) = app.selection.checked_sub(1) {
                app.selection = selection;
            }
        }
        KeyCode::Down => {
            let max_selection = match &app.step {
                WizardStep::LanguageSelection => 1, // English, 中文
                WizardStep::AISettings => 1, // API Key, Model
                WizardStep::UISettings => 1, // Colored, Theme
                _ => 0,
            };
            
            if app.selection < max_selection {
                app.selection += 1;
            }
        }
        
        _ => { /* 忽略其他按键 */ }
    }
    
    Ok(true)
}

/// 处理输入模式下的按键事件
fn handle_input_mode_key_event(app: &mut WizardApp, key_event: KeyEvent) -> Result<bool, Box<dyn Error>> {
    match key_event.code {
        // 保存输入
        KeyCode::Enter => {
            save_input(app);
            app.is_input_mode = false;
            app.input.clear();
        }
        // 取消输入
        KeyCode::Esc => {
            app.is_input_mode = false;
            app.input.clear();
        }
        // 删除字符
        KeyCode::Backspace => {
            app.input.pop();
        }
        // 输入字符
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        
        _ => { /* 忽略其他按键 */ }
    }
    
    Ok(true)
}

/// 处理Enter键
fn handle_enter_key(app: &mut WizardApp) {
    match &mut app.step {
        WizardStep::Welcome => {
            app.step = WizardStep::LanguageSelection;
        }
        WizardStep::LanguageSelection => {
            // 保存语言选择
            app.config.app.language = match app.selection {
                0 => "en",
                1 => "zh",
                _ => "en",
            }.to_string();
            app.step = WizardStep::AISettings;
        }
        WizardStep::AISettings => {
            if let Some(openai_config) = &mut app.config.ai.openai {
                match app.selection {
                    0 => {
                        // 开始编辑API密钥
                        app.is_input_mode = true;
                        app.current_field = Some("api_key".to_string());
                    }
                    1 => {
                        // 开始编辑默认模型
                        app.is_input_mode = true;
                        app.current_field = Some("default_model".to_string());
                    }
                    _ => {}
                }
            }
        }
        WizardStep::UISettings => {
            match app.selection {
                0 => {
                    // 切换彩色输出
                    app.config.ui.colored = !app.config.ui.colored;
                }
                1 => {
                    // 切换主题
                    let themes = vec!["default", "dark", "light"];
                    let current_index = themes.iter().position(|t| t == &app.config.ui.theme).unwrap_or(0);
                    let next_index = (current_index + 1) % themes.len();
                    app.config.ui.theme = themes[next_index].to_string();
                }
                _ => {}
            }
        }
        WizardStep::Summary => {
            app.step = WizardStep::Complete;
        }
        WizardStep::Complete => {
            // 向导完成，退出
            // 配置会在run_wizard函数中保存
        }
    }
}

/// 保存输入
fn save_input(app: &mut WizardApp) {
    if let Some(field) = &app.current_field {
        if let Some(openai_config) = &mut app.config.ai.openai {
            match field.as_str() {
                "api_key" => {
                    openai_config.api_key = app.input.clone();
                }
                "default_model" => {
                    openai_config.default_model = app.input.clone();
                }
                _ => {}
            }
        }
    }
}
