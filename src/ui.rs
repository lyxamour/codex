use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ai::adapter::AIClient;

/// Widget cache for optimized rendering
struct WidgetCache {
    /// Last render time
    last_render: Instant,
    /// Cached widget
    widget: Box<dyn Widget>,
    /// Cache validity flag
    valid: bool,
}

/// UI application state
struct App {
    /// Current active tab
    active_tab: usize,
    /// Tab names
    tabs: Vec<String>,
    /// Input buffer
    input: String,
    /// Input history
    history: Vec<String>,
    /// History index for navigation
    history_index: Option<usize>,
    /// Output content
    output: Vec<String>,
    /// Scrolling offset for output
    output_offset: usize,
    /// AI Client
    ai_client: Arc<AIClient>,
    /// Search mode enabled
    search_mode: bool,
    /// Search query
    search_query: String,
    /// Search results
    search_results: Vec<String>,
    /// Selected search result index
    selected_result: usize,
    /// Search history
    search_history: Vec<String>,
    /// Search history index
    search_history_index: Option<usize>,
    /// Code browser file tree
    code_files: Vec<String>,
    /// Selected file in code browser
    selected_file: String,
    /// File content
    file_content: Vec<String>,
    /// File content offset
    file_offset: usize,
    /// Settings mode enabled
    settings_mode: bool,
    /// Current setting being edited
    selected_setting: usize,
    /// Setting options
    settings: Vec<(String, String)>,

    // Rendering optimization fields
    /// Last render time
    last_render: Instant,
    /// Render count for performance tracking
    render_count: u64,
    /// Dirty flags for optimized rendering
    dirty: bool,
    /// Output area dirty flag
    output_dirty: bool,
    /// Input area dirty flag
    input_dirty: bool,
    /// Tabs area dirty flag
    tabs_dirty: bool,
    /// Widget cache for expensive widgets
    widget_cache: HashMap<String, WidgetCache>,
    /// Last terminal size for cache invalidation
    last_size: Rect,
    /// Enable render throttling
    render_throttling: bool,
    /// Minimum render interval (milliseconds)
    min_render_interval: u64,
}

impl App {
    /// Create a new App instance with AI client
    fn new(ai_client: AIClient) -> Self {
        let now = Instant::now();
        Self {
            active_tab: 0,
            tabs: vec![
                "Chat".to_string(),
                "Code".to_string(),
                "Knowledge Base".to_string(),
                "Tasks".to_string(),
                "Solo Mode".to_string(),
            ],
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            output: vec!["Welcome to Codex! Type 'help' for commands.".to_string()],
            output_offset: 0,
            ai_client: Arc::new(ai_client),
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_result: 0,
            search_history: Vec::new(),
            search_history_index: None,
            code_files: Vec::new(),
            selected_file: String::new(),
            file_content: Vec::new(),
            file_offset: 0,
            settings_mode: false,
            selected_setting: 0,
            settings: vec![
                ("Language".to_string(), "English".to_string()),
                ("Theme".to_string(), "Dark".to_string()),
                ("AI Model".to_string(), "gpt-4o-mini".to_string()),
                ("Max Tokens".to_string(), "1024".to_string()),
                ("Temperature".to_string(), "0.7".to_string()),
            ],

            // Rendering optimization defaults
            last_render: now,
            render_count: 0,
            dirty: true,
            output_dirty: true,
            input_dirty: true,
            tabs_dirty: true,
            widget_cache: HashMap::new(),
            last_size: Rect::default(),
            render_throttling: true,
            min_render_interval: 16, // ~60fps
        }
    }

    /// Mark output area as dirty
    fn mark_output_dirty(&mut self) {
        self.output_dirty = true;
        self.dirty = true;
    }

    /// Mark input area as dirty
    fn mark_input_dirty(&mut self) {
        self.input_dirty = true;
        self.dirty = true;
    }

    /// Mark tabs area as dirty
    fn mark_tabs_dirty(&mut self) {
        self.tabs_dirty = true;
        self.dirty = true;
    }

    /// Clear dirty flags after render
    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.output_dirty = false;
        self.input_dirty = false;
        self.tabs_dirty = false;
    }

    /// Check if render is needed based on dirty flags and throttling
    fn should_render(&mut self) -> bool {
        if !self.dirty {
            return false;
        }

        if !self.render_throttling {
            return true;
        }

        // Check if enough time has passed since last render
        let elapsed = self.last_render.elapsed().as_millis() as u64;
        elapsed >= self.min_render_interval
    }

    /// Invalidate widget cache for a specific widget
    fn invalidate_cache(&mut self, cache_key: &str) {
        if let Some(cache) = self.widget_cache.get_mut(cache_key) {
            cache.valid = false;
        }
    }

    /// Invalidate all widget caches
    fn invalidate_all_caches(&mut self) {
        for cache in self.widget_cache.values_mut() {
            cache.valid = false;
        }
    }

    /// Update last render time and increment render count
    fn update_render_stats(&mut self) {
        self.last_render = Instant::now();
        self.render_count += 1;
    }
}

/// Render search mode UI
fn render_search_mode(f: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Search results
            Constraint::Length(1), // Status bar
        ])
        .split(f.size());

    // Render search input
    let search_block = Block::default().borders(Borders::ALL).title("Search");

    let search_input = Paragraph::new(format!("/ {}", app.search_query))
        .block(search_block)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    f.render_widget(search_input, layout[0]);

    // Render cursor position
    f.set_cursor(
        layout[0].x + app.search_query.len() as u16 + 2, // +2 for "/ " prefix
        layout[0].y + 1,
    );

    // Render search results
    let results_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Results");

    let mut items = Vec::new();
    for (i, result) in app.search_results.iter().enumerate() {
        let style = if i == app.selected_result {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        items.push(ListItem::new(Span::styled(result.clone(), style)));
    }

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No results found",
            Style::default().fg(Color::Red),
        )));
    }

    // Save the items length before moving items
    let items_len = items.len();

    let results_list = List::new(items)
        .block(results_block)
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(
        results_list,
        layout[1],
        &mut ListState::default().with_selected(Some(app.selected_result.min(items_len - 1))),
    );

    // Render status bar
    let status_text = format!("Press Enter to select, Esc to exit search");
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(status_bar, layout[2]);
}

/// Render settings mode UI
fn render_settings_mode(f: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Settings list
            Constraint::Length(1), // Status bar
        ])
        .split(f.size());

    // Render title
    let title = Paragraph::new("Settings")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(title, layout[0]);

    // Render settings list
    let settings_block = Block::default()
        .borders(Borders::ALL)
        .title("Configuration Options");

    let mut items = Vec::new();
    for (i, (setting, value)) in app.settings.iter().enumerate() {
        let style = if i == app.selected_setting {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        let item_text = format!("{:20} : {}", setting, value);
        items.push(ListItem::new(Span::styled(item_text, style)));
    }

    let settings_list = List::new(items)
        .block(settings_block)
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(
        settings_list,
        layout[1],
        &mut ListState::default().with_selected(Some(app.selected_setting)),
    );

    // Render status bar
    let status_text = format!("Press Enter to edit, Esc to exit settings");
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(status_bar, layout[2]);
}

/// Render code browser UI
fn render_code_browser(f: &mut Frame, area: Rect, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // File tree
            Constraint::Percentage(70), // File content
        ])
        .split(area);

    // Render file tree
    let file_tree_block = Block::default().borders(Borders::ALL).title("File Tree");

    let mut items = Vec::new();
    for file in &app.code_files {
        let style = if *file == app.selected_file {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        items.push(ListItem::new(Span::styled(file.clone(), style)));
    }

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No files found",
            Style::default().fg(Color::Red),
        )));
    }

    let file_list = List::new(items)
        .block(file_tree_block)
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(file_list, layout[0], &mut ListState::default());

    // Render file content
    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("File: {}", app.selected_file));

    let content_items = app
        .file_content
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect::<Vec<_>>();

    let content_list = List::new(content_items)
        .block(content_block)
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(
        content_list,
        layout[1],
        &mut ListState::default().with_offset(app.file_offset),
    );
}

/// Run the UI application
pub fn run(tab: Option<String>) -> Result<(), Box<dyn Error>> {
    // Initialize AI client
    let ai_client = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { crate::ai::adapter::AIClient::new().await })?;

    // Initialize terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with AI client
    let mut app = App::new(ai_client);

    // Set initial tab if specified
    if let Some(tab_name) = tab {
        if let Some(index) = app.tabs.iter().position(|t| t == &tab_name) {
            app.active_tab = index;
            app.mark_tabs_dirty();
        }
    }

    // Run main loop
    loop {
        // Check if render is needed based on dirty flags and throttling
        if app.should_render() {
            // Draw UI with optimized rendering
            terminal.draw(|f| {
                render_app(f, &app);
                app.update_render_stats();
                app.clear_dirty();
            })?;
        }

        // Handle events with a timeout to support throttling
        if event::poll(Duration::from_millis(16))? {
            if !handle_events(&mut app)? {
                break;
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Render the application UI
fn render_app(f: &mut Frame, app: &App) {
    // Check if we're in a special mode
    if app.search_mode {
        render_search_mode(f, app);
    } else if app.settings_mode {
        render_settings_mode(f, app);
    } else {
        // Normal mode layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Output area
                Constraint::Length(3), // Input area
            ])
            .split(f.size());

        // Render tabs
        render_tabs(f, layout[0], app);

        // Render output area - check if we're in code browser
        if app.active_tab == 1 {
            // Code tab
            render_code_browser(f, layout[1], app);
        } else {
            render_output(f, layout[1], app);
        }

        // Render input area
        render_input(f, layout[2], app);
    }
}

/// Render tab bar
fn render_tabs(f: &mut Frame, area: Rect, app: &App) {
    let tabs = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let style = if i == app.active_tab {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            };
            Line::from(vec![Span::styled(format!(" {}", tab), style)])
        })
        .collect::<Vec<_>>();

    let tabs_widget = Paragraph::new(tabs)
        .style(Style::default().bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(tabs_widget, area);
}

/// Render output area
fn render_output(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Output - {}", app.tabs[app.active_tab]));

    let items = app
        .output
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");

    f.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_offset(app.output_offset),
    );
}

/// Render input area
fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("Input");

    let input = Paragraph::new(app.input.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, app.input.len() as u16))
        .alignment(Alignment::Left);

    f.render_widget(input, area);

    // Set cursor position
    f.set_cursor(area.x + app.input.len() as u16 + 1, area.y + 1);
}

/// Handle terminal events
fn handle_events(app: &mut App) -> Result<bool, Box<dyn Error>> {
    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(key_event) => {
                handle_key_event(app, key_event)?;
                // Set dirty flags based on key event
                if app.search_mode || app.settings_mode {
                    app.mark_output_dirty();
                } else {
                    app.mark_input_dirty();
                    if app.active_tab != 1 {
                        app.mark_output_dirty();
                    }
                }
            }
            Event::Mouse(_mouse_event) => {
                // Handle mouse events if needed
                app.mark_output_dirty();
            }
            Event::Resize(_, _) => {
                // Terminal resized, invalidate all caches and set all areas as dirty
                app.invalidate_all_caches();
                app.mark_output_dirty();
                app.mark_input_dirty();
                app.mark_tabs_dirty();
            }
            Event::FocusGained | Event::FocusLost | Event::Paste(_) => {
                // Ignore focus and paste events for now
            }
        }
    }

    Ok(true)
}

/// Handle key events
fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
    // Handle search mode separately
    if app.search_mode {
        return handle_search_key_event(app, key_event);
    }

    // Handle settings mode separately
    if app.settings_mode {
        return handle_settings_key_event(app, key_event);
    }

    // Handle code browser mode
    if app.active_tab == 1 {
        return handle_code_browser_key_event(app, key_event);
    }

    // Normal mode key handling
    match key_event.code {
        // Exit on Ctrl+C or Esc
        KeyCode::Esc | KeyCode::Char('c')
            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            return Ok(());
        }
        // Start search with '/' key
        KeyCode::Char('/') => {
            app.search_mode = true;
            app.search_query.clear();
            app.selected_result = 0;
        }
        // Start settings mode with ':' key
        KeyCode::Char(':') => {
            app.settings_mode = true;
            app.selected_setting = 0;
        }
        // Tab navigation
        KeyCode::Tab => {
            app.active_tab = (app.active_tab + 1) % app.tabs.len();
        }
        KeyCode::BackTab => {
            app.active_tab = (app.active_tab + app.tabs.len() - 1) % app.tabs.len();
        }
        // Enter key - process input
        KeyCode::Enter => {
            let input = app.input.clone();
            if !input.is_empty() {
                // Add to history
                app.history.push(input.clone());
                app.history_index = None;

                // Process input
                process_input(app, &input)?;

                // Clear input
                app.input.clear();
            }
        }
        // Backspace - delete last character
        KeyCode::Backspace => {
            app.input.pop();
        }
        // Delete key - delete current character
        KeyCode::Delete => {
            // Not implemented yet
        }
        // Arrow keys for history navigation
        KeyCode::Up => {
            if app.history_index.is_none() && !app.history.is_empty() {
                app.history_index = Some(app.history.len() - 1);
            } else if let Some(index) = app.history_index.as_mut() {
                if *index > 0 {
                    *index -= 1;
                }
            }

            if let Some(index) = app.history_index {
                app.input = app.history[index].clone();
            }
        }
        KeyCode::Down => {
            if let Some(index) = app.history_index.as_mut() {
                if *index < app.history.len() - 1 {
                    *index += 1;
                    app.input = app.history[*index].clone();
                } else {
                    app.history_index = None;
                    app.input.clear();
                }
            }
        }
        // Arrow keys for input navigation
        KeyCode::Left => {
            // Not implemented yet
        }
        KeyCode::Right => {
            // Not implemented yet
        }
        // Character input
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        // Other keys
        _ => {
            // Ignore other keys
        }
    }

    Ok(())
}

/// Handle key events in search mode
fn handle_search_key_event(app: &mut App, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
    match key_event.code {
        // Exit search mode
        KeyCode::Esc => {
            app.search_mode = false;
            app.search_query.clear();
        }
        // Select search result
        KeyCode::Enter => {
            if !app.search_results.is_empty() {
                // Process the selected result
                let selected = app.search_results[app.selected_result].clone();
                app.output.push(format!("Selected: {}", selected));
                app.search_history.push(app.search_query.clone());
            }
            app.search_mode = false;
        }
        // Navigate search results
        KeyCode::Up => {
            if app.selected_result > 0 {
                app.selected_result -= 1;
            }
        }
        KeyCode::Down => {
            if app.selected_result < app.search_results.len() - 1 {
                app.selected_result += 1;
            }
        }
        // Backspace - delete last character
        KeyCode::Backspace => {
            app.search_query.pop();
            // TODO: 主人~ 这里需要实现实时搜索功能，根据 search_query 更新 search_results
        }
        // Character input
        KeyCode::Char(c) => {
            app.search_query.push(c);
            // TODO: 主人~ 这里需要实现实时搜索功能，根据 search_query 更新 search_results
        }
        // Search history navigation
        KeyCode::PageUp => {
            if app.search_history_index.is_none() && !app.search_history.is_empty() {
                app.search_history_index = Some(app.search_history.len() - 1);
                app.search_query = app.search_history[app.search_history_index.unwrap()].clone();
            } else if let Some(index) = app.search_history_index.as_mut() {
                if *index > 0 {
                    *index -= 1;
                    app.search_query = app.search_history[*index].clone();
                }
            }
        }
        KeyCode::PageDown => {
            if let Some(index) = app.search_history_index.as_mut() {
                if *index < app.search_history.len() - 1 {
                    *index += 1;
                    app.search_query = app.search_history[*index].clone();
                } else {
                    app.search_history_index = None;
                    app.search_query.clear();
                }
            }
        }
        _ => {
            // Ignore other keys
        }
    }

    Ok(())
}

/// Handle key events in settings mode
fn handle_settings_key_event(app: &mut App, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
    match key_event.code {
        // Exit settings mode
        KeyCode::Esc => {
            app.settings_mode = false;
        }
        // Select setting to edit
        KeyCode::Enter => {
            // TODO: 主人~ 这里需要实现编辑设置的功能
            app.output.push(format!(
                "Editing setting: {}",
                app.settings[app.selected_setting].0
            ));
        }
        // Navigate settings
        KeyCode::Up => {
            if app.selected_setting > 0 {
                app.selected_setting -= 1;
            }
        }
        KeyCode::Down => {
            if app.selected_setting < app.settings.len() - 1 {
                app.selected_setting += 1;
            }
        }
        _ => {
            // Ignore other keys
        }
    }

    Ok(())
}

/// Handle key events in code browser mode
fn handle_code_browser_key_event(app: &mut App, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
    match key_event.code {
        // Exit code browser mode
        KeyCode::Esc => {
            app.active_tab = 0;
        }
        // Navigate file list
        KeyCode::Up => {
            // TODO: 主人~ 这里需要实现文件列表向上导航
        }
        KeyCode::Down => {
            // TODO: 主人~ 这里需要实现文件列表向下导航
        }
        // Select file
        KeyCode::Enter => {
            // TODO: 主人~ 这里需要实现选择文件并加载其内容
        }
        // Scroll file content
        KeyCode::PageUp => {
            if app.file_offset > 0 {
                app.file_offset = app.file_offset.saturating_sub(10);
            }
        }
        KeyCode::PageDown => {
            app.file_offset += 10;
        }
        _ => {
            // Ignore other keys
        }
    }

    Ok(())
}

/// Process user input
fn process_input(app: &mut App, input: &str) -> Result<(), Box<dyn Error>> {
    // Add input to output
    app.output.push(format!("$ {}", input));
    app.mark_output_dirty();

    // Simple command processing for demo
    match input.trim() {
        "help" => {
            app.output.push("Available commands:".to_string());
            app.output
                .push("  help - Show this help message".to_string());
            app.output.push("  clear - Clear output".to_string());
            app.output.push("  exit - Exit the application".to_string());
            app.output.push("  tabs - List available tabs".to_string());
            app.mark_output_dirty();
        }
        "clear" => {
            app.output.clear();
            app.mark_output_dirty();
        }
        "exit" => {
            return Ok(());
        }
        "tabs" => {
            app.output.push("Available tabs:".to_string());
            for (i, tab) in app.tabs.iter().enumerate() {
                app.output.push(format!("  {}: {}", i + 1, tab));
            }
            app.mark_output_dirty();
        }
        _ => {
            // Call AI client to generate response
            app.output.push("Thinking...".to_string());
            app.mark_output_dirty();

            // Auto-scroll to bottom
            app.output_offset = app.output.len().saturating_sub(1);

            // Use tokio to handle async call
            let ai_client = app.ai_client.clone();
            let prompt = input.to_string();

            // Run async task in blocking mode
            let response = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async move { ai_client.generate_response(&prompt, None).await })?;

            // Replace "Thinking..." with actual response
            app.output.pop();
            app.output.push(format!("Codex: {}", response.content()));
            app.mark_output_dirty();
        }
    }

    // Auto-scroll to bottom
    app.output_offset = app.output.len().saturating_sub(1);

    Ok(())
}
