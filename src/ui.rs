use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use ratatui::{prelude::*, widgets::*};
use std::error::Error;
use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;

use crate::ai::adapter::AIClient;

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
}

impl App {
    /// Create a new App instance with AI client
    fn new(ai_client: AIClient) -> Self {
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
        }
    }
}

/// Run the UI application
pub fn run(tab: Option<String>) -> Result<(), Box<dyn Error>> {
    // Initialize AI client
    let ai_client = crate::ai::adapter::AIClient::new()?;

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
        }
    }

    // Run main loop
    loop {
        // Draw UI
        terminal.draw(|f| render_app(f, &app))?;

        // Handle events
        if !handle_events(&mut app)? {
            break;
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

    // Render output area
    render_output(f, layout[1], app);

    // Render input area
    render_input(f, layout[2], app);
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
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key_event) => {
                handle_key_event(app, key_event)?;
            }
            Event::Mouse(_mouse_event) => {
                // Handle mouse events if needed
            }
            Event::Resize(_, _) => {
                // Terminal resized, will be handled by next draw
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
    match key_event.code {
        // Exit on Ctrl+C or Esc
        KeyCode::Esc | KeyCode::Char('c')
            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            return Ok(());
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

/// Process user input
fn process_input(app: &mut App, input: &str) -> Result<(), Box<dyn Error>> {
    // Add input to output
    app.output.push(format!("$ {}", input));

    // Simple command processing for demo
    match input.trim() {
        "help" => {
            app.output.push("Available commands:".to_string());
            app.output
                .push("  help - Show this help message".to_string());
            app.output.push("  clear - Clear output".to_string());
            app.output.push("  exit - Exit the application".to_string());
            app.output.push("  tabs - List available tabs".to_string());
        }
        "clear" => {
            app.output.clear();
        }
        "exit" => {
            return Ok(());
        }
        "tabs" => {
            app.output.push("Available tabs:".to_string());
            for (i, tab) in app.tabs.iter().enumerate() {
                app.output.push(format!("  {}: {}", i + 1, tab));
            }
        }
        _ => {
            // Call AI client to generate response
            app.output.push("Thinking...".to_string());

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
        }
    }

    // Auto-scroll to bottom
    app.output_offset = app.output.len().saturating_sub(1);

    Ok(())
}
