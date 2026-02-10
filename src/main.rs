#![allow(dead_code)]

mod app;
mod config;
mod event;
mod git;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

use app::{App, Popup, View};
use event::{AppEvent, EventHandler};

fn main() -> Result<()> {
    // Check if we're in a git repo
    if !git::runner::is_git_repo() {
        eprintln!("Error: Not a git repository. Run 'git init' first or navigate to a git repository.");
        std::process::exit(1);
    }

    // Load config
    let config = config::Config::load().unwrap_or_default();
    let tick_rate = config.general.tick_rate_ms;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = App::new(config);
    let events = EventHandler::new(tick_rate);

    // Main loop
    let res = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|f| draw(f, app))?;

        // Handle events
        match events.next()? {
            AppEvent::Key(key) => {
                app.handle_key(key)?;
            }
            AppEvent::Tick => {
                // Auto-refresh on tick (only for dashboard)
                if app.view == View::Dashboard {
                    app.dashboard_state.refresh();
                }
                // Poll GitHub Device Flow if active
                if app.view == View::GitHub {
                    ui::github::tick_device_auth(app);
                }
            }
            AppEvent::Resize(_, _) => {
                // Terminal will handle resize automatically
            }
        }

        if !app.running {
            return Ok(());
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Render the current view
    match app.view {
        View::Dashboard => {
            ui::dashboard::render(f, area, &app.dashboard_state, &app.status_message);
        }
        View::Staging => {
            ui::staging::render(f, area, &mut app.staging_state);
        }
        View::Commit => {
            ui::commit::render(f, area, &app.commit_state);
        }
        View::Branches => {
            ui::branches::render(f, area, &mut app.branches_state);
        }
        View::Timeline => {
            ui::timeline::render(f, area, &mut app.timeline_state);
        }
        View::TimeTravel => {
            ui::time_travel::render(f, area, &mut app.time_travel_state);
        }
        View::Reflog => {
            ui::reflog::render(f, area, &mut app.reflog_state);
        }
        View::GitHub => {
            let config = app.config.clone();
            ui::github::render(f, area, &mut app.github_state, &config);
        }
    }

    // Render popup overlay
    match &app.popup {
        Popup::Help => {
            ui::help::render(f, area, app.view);
        }
        Popup::Confirm { title, message, .. } => {
            render_popup(f, area, title, message, Color::Yellow);
        }
        Popup::Input { title, prompt, value, .. } => {
            let content = format!("{}{}", prompt, value);
            render_popup(f, area, title, &content, Color::Cyan);
        }
        Popup::Message { title, message } => {
            render_popup(f, area, title, message, Color::White);
        }
        Popup::None => {}
    }
}

fn render_popup(f: &mut Frame, area: Rect, title: &str, message: &str, border_color: Color) {
    let popup_area = centered_rect(50, 40, area);
    f.render_widget(Clear, popup_area);

    let lines: Vec<Line> = message
        .lines()
        .map(|l| Line::from(Span::styled(format!("  {}", l), Style::default().fg(Color::White))))
        .collect();

    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(border_color).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(popup, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
