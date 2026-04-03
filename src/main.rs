mod ai;
mod app;
mod config;
mod event;
mod git;
mod keychain;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

use app::{App, Popup, View};
use event::{AppEvent, EventHandler};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");

fn print_help() {
    println!("{} v{}", PKG_NAME, VERSION);
    println!("{}", PKG_DESC);
    println!();
    println!("USAGE:");
    println!("    zit [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message");
    println!("    -v, --version    Print version information");
    println!("    --verbose        Enable verbose logging (ZIT_LOG=debug)");
    println!("    --no-ai          Disable AI features for this session");
    println!();
    println!("ENVIRONMENT:");
    println!("    ZIT_LOG          Set log level (error, warn, info, debug, trace)");
    println!("    ZIT_AI_ENDPOINT  AI mentor API endpoint URL");
    println!("    ZIT_AI_API_KEY   AI mentor API key");
    println!();
    println!("VIEWS:");
    println!("    s  Staging     c  Commit      b  Branches");
    println!("    l  Timeline    t  Time Travel  r  Reflog");
    println!("    g  GitHub      a  AI Mentor    x  Stash");
    println!("    ?  Help");
}

fn main() -> Result<()> {
    // Parse CLI flags
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut no_ai = false;
    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                println!("{} {}", PKG_NAME, VERSION);
                return Ok(());
            }
            "--verbose" => {
                unsafe { std::env::set_var("ZIT_LOG", "debug") };
            }
            "--no-ai" => {
                no_ai = true;
            }
            other => {
                eprintln!("Unknown option: {}", other);
                eprintln!("Run 'zit --help' for usage.");
                std::process::exit(1);
            }
        }
    }

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::new().filter("ZIT_LOG"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    log::info!("Starting {} v{}", PKG_NAME, VERSION);

    // Check if we're in a git repo
    if !git::runner::is_git_repo() {
        eprintln!(
            "Error: Not a git repository. Run 'git init' first or navigate to a git repository."
        );
        std::process::exit(1);
    }

    // Check git version meets minimum requirements
    if let Err(e) = git::runner::check_git_version() {
        eprintln!("Warning: {}", e);
        eprintln!("Some features may not work correctly. Git ≥ 2.13.0 is required.");
    }

    // Load config
    let mut config = config::Config::load().unwrap_or_default();
    log::debug!("Config loaded from {:?}", config::Config::path());

    // Apply --no-ai flag
    if no_ai {
        config.ai.enabled = false;
        log::info!("AI features disabled via --no-ai flag");
    }

    // Migrate plaintext tokens to OS keychain (one-time)
    let migrated = keychain::migrate_from_config(&mut config);
    if migrated > 0 {
        if let Err(e) = config.save() {
            log::warn!("Failed to save config after keychain migration: {}", e);
        } else {
            log::info!("Migrated {} secret(s) from config to OS keychain", migrated);
        }
    }

    let tick_rate = config.general.tick_rate_ms;

    // Install panic hook that restores the terminal before printing the panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = App::new(config);
    let events = EventHandler::new(tick_rate);

    // Main loop
    let res = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Remove panic hook since terminal is restored
    let _ = std::panic::take_hook();

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
                app.poll_ai_result();
                app.handle_key(key)?;
            }
            AppEvent::Tick => {
                app.poll_ai_result();
                // Auto-refresh on tick for the current view
                app.refresh();
                // Poll GitHub Device Flow if active
                if app.view == View::GitHub {
                    ui::github::tick_device_auth(app);
                    ui::github::tick_pr_state(app);
                    ui::github::tick_actions_state(app);
                }
            }
            AppEvent::Mouse(mouse) => {
                app.poll_ai_result();
                app.handle_mouse(mouse);
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
            let ai_loading = app.ai_loading;
            let ai_available = app.ai_client.is_some();
            ui::commit::render(f, area, &app.commit_state, ai_loading, ai_available);
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
            let config = &app.config;
            ui::github::render(f, area, &mut app.github_state, config);
        }
        View::AiMentor => {
            let ai_available = app.ai_client.is_some();
            let loading = app.ai_loading;
            let provider_label = app
                .ai_client
                .as_ref()
                .map(|c| c.provider_name().to_string())
                .unwrap_or_default();
            ui::ai_mentor::render(
                f,
                area,
                &app.ai_mentor_state,
                ai_available,
                loading,
                &provider_label,
            );
        }
        View::Stash => {
            ui::stash::render(f, area, &mut app.stash_state);
        }
        View::MergeResolve => {
            let ai_loading = app.ai_loading;
            let ai_available = app.ai_client.is_some();
            ui::merge_resolve::render(f, area, &app.merge_resolve_state, ai_loading, ai_available);
        }
        View::WorkflowBuilder => {
            ui::workflow_builder::render(f, area, &app.workflow_builder_state);
        }
        View::Bisect => {
            ui::bisect::render(f, area, &mut app.bisect_state);
        }
        View::CherryPick => {
            ui::cherry_pick::render(f, area, &mut app.cherry_pick_state);
        }
        View::Agent => {
            let ai_available = app.ai_client.is_some();
            let loading = app.ai_loading;
            let provider_label = app
                .ai_client
                .as_ref()
                .map(|c| c.provider_name().to_string())
                .unwrap_or_default();
            ui::agent::render(
                f,
                area,
                &app.agent_state,
                ai_available,
                loading,
                &provider_label,
            );
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
        Popup::Input {
            title,
            prompt,
            value,
            ..
        } => {
            let content = format!("{}{}", prompt, value);
            render_popup(f, area, title, &content, Color::Cyan);
        }
        Popup::Message { title, message } => {
            render_popup(f, area, title, message, Color::White);
        }
        Popup::FollowUp {
            title,
            context: _,
            suggestions,
            selected,
        } => {
            let popup_area = ui::utils::centered_rect(60, 50, area);
            f.render_widget(Clear, popup_area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", title),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (i, item) in suggestions.iter().enumerate() {
                let is_sel = i == *selected;
                let prefix = if is_sel { "  ▶ " } else { "    " };
                let style = if is_sel {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{}", prefix, i + 1),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(". "),
                    Span::styled(&item.label, style),
                ]));
                lines.push(Line::from(Span::styled(
                    format!("       {}", item.description),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  [Enter] Select  [j/k] Navigate  [Esc] Close",
                Style::default().fg(Color::DarkGray),
            )));

            let popup = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(Span::styled(
                            " Follow-up Actions ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(popup, popup_area);
        }
        Popup::SecretWarning {
            findings,
            pending_action: _,
            selected,
        } => {
            let popup_area = ui::utils::centered_rect(70, 60, area);
            f.render_widget(Clear, popup_area);

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  🛡 Potential secrets detected!",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "  The following sensitive data was found:",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
            ];

            for (i, finding) in findings.iter().enumerate() {
                let is_sel = i == *selected;
                let prefix = if is_sel { "  ▶ " } else { "    " };
                let style = if is_sel {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Red)),
                    Span::styled(&finding.rule_name, style),
                ]));
                lines.push(Line::from(Span::styled(
                    format!(
                        "       {}:{} — {}",
                        finding.file, finding.line, finding.preview
                    ),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(" Esc", Style::default().fg(Color::Green)),
                Span::raw(" Abort  "),
                Span::styled("f", Style::default().fg(Color::Red)),
                Span::raw(" Force  "),
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw(" Allowlist  "),
                Span::styled("j/k", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate"),
            ]));

            let popup = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(Span::styled(
                            " 🛡 Secret Scanner ",
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(popup, popup_area);
        }
        Popup::None => {}
    }
}

fn render_popup(f: &mut Frame, area: Rect, title: &str, message: &str, border_color: Color) {
    let popup_area = ui::utils::centered_rect(50, 40, area);
    f.render_widget(Clear, popup_area);

    let lines: Vec<Line> = message
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                format!("  {}", l),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(popup, popup_area);
}
