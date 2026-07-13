// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — TUI Admin Interface (ratatui)
//
// Terminal UI for quick admin tasks:
// - Config management
// - License activation
// - System status
// ═══════════════════════════════════════════════════════════════════════

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;
use tracing::info;

use crate::app::AppState;

/// Run the TUI admin interface.
pub async fn run(app_state: AppState) -> shared::Result<()> {
    info!("Starting TUI admin mode");

    enable_raw_mode().map_err(|e| shared::AppError::internal(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| shared::AppError::internal(format!("Failed to enter alternate screen: {}", e)))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| shared::AppError::internal(format!("Failed to create terminal: {}", e)))?;

    let result = run_app(&mut terminal, app_state).await;

    disable_raw_mode().map_err(|e| shared::AppError::internal(format!("Failed to disable raw mode: {}", e)))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(|e| shared::AppError::internal(format!("Failed to leave alternate screen: {}", e)))?;
    terminal.show_cursor()
        .map_err(|e| shared::AppError::internal(format!("Failed to show cursor: {}", e)))?;

    result
}

/// Tabs in the TUI.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Config,
    License,
    Logs,
}

impl Tab {
    fn title(&self) -> &'static str {
        match self {
            Tab::Overview => "📊 Overview",
            Tab::Config => "⚙️ Config",
            Tab::License => "🔑 License",
            Tab::Logs => "📋 Logs",
        }
    }
}

/// Application state for the TUI.
struct TuiState {
    selected_tab: Tab,
    tabs: Vec<Tab>,
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app_state: AppState) -> shared::Result<()> {
    let mut state = TuiState {
        selected_tab: Tab::Overview,
        tabs: vec![Tab::Overview, Tab::Config, Tab::License, Tab::Logs],
    };

    loop {
        terminal.draw(|f| {
            ui(f, &state, &app_state);
        }).map_err(|e| shared::AppError::internal(format!("Draw error: {}", e)))?;

        if let Event::Key(key) = event::read()
            .map_err(|e| shared::AppError::internal(format!("Event error: {}", e)))? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('1') => state.selected_tab = Tab::Overview,
                KeyCode::Char('2') => state.selected_tab = Tab::Config,
                KeyCode::Char('3') => state.selected_tab = Tab::License,
                KeyCode::Char('4') => state.selected_tab = Tab::Logs,
                KeyCode::Left => {
                    let idx = state.tabs.iter().position(|t| *t == state.selected_tab).unwrap_or(0);
                    state.selected_tab = state.tabs[if idx == 0 { state.tabs.len() - 1 } else { idx - 1 }];
                }
                KeyCode::Right => {
                    let idx = state.tabs.iter().position(|t| *t == state.selected_tab).unwrap_or(0);
                    state.selected_tab = state.tabs[(idx + 1) % state.tabs.len()];
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Render the TUI.
fn ui(f: &mut Frame, state: &TuiState, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("🔷 TRRUSTT", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" — "),
        Span::styled("Admin Console", Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled("Press q to quit, 1-4 for tabs", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Tabs
    let tab_titles: Vec<Line> = state.tabs.iter().map(|t| {
        let style = if *t == state.selected_tab {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        Line::from(Span::styled(t.title(), style))
    }).collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .select(state.tabs.iter().position(|t| *t == state.selected_tab).unwrap_or(0))
        .highlight_style(Style::default().fg(Color::Cyan));
    f.render_widget(tabs, chunks[1]);

    // Content area
    let content = match state.selected_tab {
        Tab::Overview => render_overview(app_state),
        Tab::Config => render_config(app_state),
        Tab::License => render_license(app_state),
        Tab::Logs => render_logs(app_state),
    };

    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(state.selected_tab.title());
    f.render_widget(content.block(content_block), chunks[2]);

    // Status bar
    let db_path = app_state.db.db_path().display().to_string();
    let status = Line::from(vec![
        Span::styled(" DB: ", Style::default().fg(Color::DarkGray)),
        Span::styled(db_path, Style::default().fg(Color::Gray)),
    ]);
    f.render_widget(Paragraph::new(status), chunks[2]);
}

fn render_overview(app_state: &AppState) -> Paragraph {
    let text = vec![
        Line::from(vec![
            Span::raw(" TRRUSTT v"),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]),
        Line::from(""),
        Line::from(vec![Span::raw(" Database: "), Span::styled(app_state.db.db_path().display().to_string(), Style::default().fg(Color::Green))]),
        Line::from(vec![Span::raw(" SSAS Port: "), Span::styled(app_state.args.port.map_or("auto".into(), |p| p.to_string()), Style::default().fg(Color::Yellow))]),
        Line::from(""),
        Line::from(Span::styled(" trRUSTt your data. One binary. Infinite dashboards.", Style::default().fg(Color::Cyan))),
    ];
    Paragraph::new(text)
}

fn render_config(_app_state: &AppState) -> Paragraph {
    Paragraph::new(vec![
        Line::from(Span::styled(" Configuration Management", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(" Use CLI for config management:"),
        Line::from("   TRRUSTT config show"),
        Line::from("   TRRUSTT config set ai.default_provider openai"),
        Line::from("   TRRUSTT config export config.json"),
    ])
}

fn render_license(_app_state: &AppState) -> Paragraph {
    Paragraph::new(vec![
        Line::from(Span::styled(" License Management", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::raw(" Status: "), Span::styled("Free Tier (no license required)", Style::default().fg(Color::Green))]),
        Line::from(""),
        Line::from(" Use CLI for license management:"),
        Line::from("   TRRUSTT license activate <key>"),
        Line::from("   TRRUSTT license status"),
    ])
}

fn render_logs(_app_state: &AppState) -> Paragraph {
    Paragraph::new(vec![
        Line::from(Span::styled(" Recent Logs", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(" Logs are written to ~/.trrustt/logs/"),
        Line::from(" Use --log-level debug for verbose output"),
    ])
}
