use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};

/// The AI mentor panel mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiMode {
    Menu,
    Input,
    Result,
    History,
}

/// A single AI interaction entry for the prompt history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiHistoryEntry {
    pub query: String,
    pub response: String,
    pub timestamp: String,
}

/// Maximum history entries to keep.
const MAX_HISTORY: usize = 50;

/// Get the history file path (~/.config/zit/ai_history.json).
fn history_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("zit")
        .join("ai_history.json")
}

/// State for the AI Mentor panel.
pub struct AiMentorState {
    pub mode: AiMode,
    pub selected: usize,
    pub input: String,
    pub result_text: String,
    pub result_scroll: u16,
    pub last_action: Option<String>,
    pub history: Vec<AiHistoryEntry>,
    pub history_selected: usize,
    pub history_scroll: u16,
}

impl Default for AiMentorState {
    fn default() -> Self {
        // Load persisted history
        let history = Self::load_history();
        Self {
            mode: AiMode::Menu,
            selected: 0,
            input: String::new(),
            result_text: String::new(),
            result_scroll: 0,
            last_action: None,
            history,
            history_selected: 0,
            history_scroll: 0,
        }
    }
}

impl AiMentorState {
    /// Load history from disk. Returns empty vec on any error.
    fn load_history() -> Vec<AiHistoryEntry> {
        let path = history_path();
        if !path.exists() {
            return Vec::new();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Save history to disk (best-effort, errors silently ignored).
    fn save_history(&self) {
        let path = history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.history) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Add a new entry to the prompt history and persist to disk.
    pub fn add_history(&mut self, query: String, response: String) {
        let timestamp = {
            use std::time::SystemTime;
            let secs = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Get local time offset and compute local HH:MM
            // Use libc on unix to get timezone offset, fall back to UTC
            #[cfg(unix)]
            let offset_secs: i64 = {
                // Safe: we're single-threaded at this point
                unsafe {
                    let mut tm: libc::tm = std::mem::zeroed();
                    let time = secs as libc::time_t;
                    libc::localtime_r(&time, &mut tm);
                    tm.tm_gmtoff
                }
            };
            #[cfg(not(unix))]
            let offset_secs: i64 = 0; // Windows falls back to UTC
            let local_secs = (secs as i64 + offset_secs) as u64;
            let hours = (local_secs / 3600) % 24;
            let mins = (local_secs / 60) % 60;
            format!("{:02}:{:02}", hours, mins)
        };
        self.history.push(AiHistoryEntry {
            query,
            response,
            timestamp,
        });
        // Trim old entries
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        // Persist to disk
        self.save_history();
    }
}

const MENU_ITEMS: &[(&str, &str)] = &[
    ("🔍 Explain Repo", "Explain the current repository state"),
    ("💬 Ask a Question", "Ask the AI mentor anything about git"),
    (
        "🛡️ Recommend",
        "Get a safe recommendation for a git operation",
    ),
    ("📚 Learn", "Learn a git concept with examples"),
    ("📄 Generate .gitignore", "AI-powered .gitignore based on project structure"),
    ("🏥 Health Check", "Test connectivity to the AI service"),
    ("📜 History", "View past AI interactions"),
    ("⚙️  Switch Provider", "Change AI provider (OpenAI, Anthropic, Ollama...)"),
];

pub fn render(f: &mut Frame, area: Rect, state: &AiMentorState, ai_available: bool, loading: bool, provider_label: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Content area
            Constraint::Length(3), // Hints
        ])
        .split(area);

    // Title
    let ai_status = if loading {
        Span::styled(" ⏳ Loading... ", Style::default().fg(Color::Yellow))
    } else if ai_available {
        Span::styled(" ● Connected ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ Not configured ", Style::default().fg(Color::Red))
    };

    // Provider info in title
    let provider_info = if ai_available && !provider_label.is_empty() {
        Span::styled(
            format!(" [{}] ", provider_label),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "🤖 AI Mentor",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" — "),
        ai_status,
        provider_info,
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    f.render_widget(title, chunks[0]);

    // Content area
    match state.mode {
        AiMode::Menu => render_menu(f, chunks[1], state, ai_available),
        AiMode::Input => render_input(f, chunks[1], state),
        AiMode::Result => render_result(f, chunks[1], state),
        AiMode::History => render_history(f, chunks[1], state),
    }

    // Hints
    let hints = match state.mode {
        AiMode::Menu => Line::from(vec![
            Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate  "),
            Span::styled("Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("Select  "),
            Span::styled("p ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Provider  "),
            Span::styled("q ", Style::default().fg(Color::Red)),
            Span::raw("Back"),
        ]),
        AiMode::Input => Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("Send  "),
            Span::styled("Esc ", Style::default().fg(Color::Red)),
            Span::raw("Cancel"),
        ]),
        AiMode::Result => Line::from(vec![
            Span::styled(" PgDn/PgUp ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll  "),
            Span::styled("Esc ", Style::default().fg(Color::Red)),
            Span::raw("Back to menu"),
        ]),
        AiMode::History => Line::from(vec![
            Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate  "),
            Span::styled("Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("View  "),
            Span::styled("Esc ", Style::default().fg(Color::Red)),
            Span::raw("Back"),
        ]),
    };
    let hints_widget = Paragraph::new(hints).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(hints_widget, chunks[2]);
}

fn render_menu(f: &mut Frame, area: Rect, state: &AiMentorState, ai_available: bool) {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::raw("")));

    for (i, (label, desc)) in MENU_ITEMS.iter().enumerate() {
        let is_selected = i == state.selected;
        let arrow = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", arrow),
                Style::default().fg(if is_selected {
                    Color::Cyan
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(*label, style),
        ]));
        lines.push(Line::from(Span::styled(
            format!("       {}", desc),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::raw("")));
    }

    if !ai_available {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "  ⚠ AI not configured. Press Enter or 'p' to set up a provider.",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(Span::styled(
            "    Supports: Bedrock, OpenAI, Anthropic, OpenRouter, Ollama",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let menu = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(
                " Choose an action ",
                Style::default().fg(Color::White),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(menu, area);
}

fn render_input(f: &mut Frame, area: Rect, state: &AiMentorState) {
    let action_label = state.last_action.as_deref().unwrap_or("Question");

    let lines = vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("  {}: ", action_label),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("  > {}_", state.input),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let input_widget = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(
                format!(" {} ", action_label),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(input_widget, area);
}

fn render_result(f: &mut Frame, area: Rect, state: &AiMentorState) {
    let title_text = state.last_action.as_deref().unwrap_or("AI Response");

    let lines: Vec<Line> = state
        .result_text
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                format!("  {}", l),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    let result = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", title_text),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .scroll((state.result_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(result, area);
}

fn render_history(f: &mut Frame, area: Rect, state: &AiMentorState) {
    if state.history.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  No AI interactions yet.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  Use the AI features and your history will appear here.",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(
                    " 📜 History ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, area);
        return;
    }

    let mut lines = Vec::new();
    lines.push(Line::from(Span::raw("")));

    for (i, entry) in state.history.iter().rev().enumerate() {
        let is_selected = i == state.history_selected;
        let arrow = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", arrow),
                Style::default().fg(if is_selected {
                    Color::Cyan
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(
                format!("[{}] ", entry.timestamp),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(entry.query.chars().take(60).collect::<String>(), style),
        ]));

        // Show truncated response preview
        let preview: String = entry
            .response
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(50)
            .collect();
        lines.push(Line::from(Span::styled(
            format!("       → {}...", preview),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::raw("")));
    }

    let history_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" 📜 History ({} entries) ", state.history.len()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .scroll((state.history_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(history_widget, area);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match app.ai_mentor_state.mode {
        AiMode::Menu => handle_menu_key(app, key),
        AiMode::Input => handle_input_key(app, key),
        AiMode::Result => handle_result_key(app, key),
        AiMode::History => handle_history_key(app, key),
    }
}

fn handle_menu_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.ai_mentor_state.selected > 0 {
                app.ai_mentor_state.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.ai_mentor_state.selected + 1 < MENU_ITEMS.len() {
                app.ai_mentor_state.selected += 1;
            }
        }
        KeyCode::Enter => {
            if app.ai_client.is_none() && app.ai_mentor_state.selected != 6 && app.ai_mentor_state.selected != 7 {
                // Launch interactive AI setup wizard (except for history/switch which don't need AI)
                app.start_ai_setup();
                return Ok(());
            }
            match app.ai_mentor_state.selected {
                0 => {
                    // Explain repo — no input needed, fire directly
                    app.ai_mentor_state.last_action = Some("Explain Repo".to_string());
                    app.start_ai_query("explain_repo".to_string(), None);
                }
                1 => {
                    // Ask a question — needs input
                    app.ai_mentor_state.last_action = Some("Ask AI".to_string());
                    app.ai_mentor_state.mode = AiMode::Input;
                    app.ai_mentor_state.input.clear();
                }
                2 => {
                    // Recommend — needs input
                    app.ai_mentor_state.last_action = Some("Recommend".to_string());
                    app.ai_mentor_state.mode = AiMode::Input;
                    app.ai_mentor_state.input.clear();
                }
                3 => {
                    // Learn — needs input (topic)
                    app.ai_mentor_state.last_action = Some("Learn".to_string());
                    app.ai_mentor_state.mode = AiMode::Input;
                    app.ai_mentor_state.input.clear();
                }
                4 => {
                    // Generate .gitignore — no input needed, fire directly
                    app.start_ai_gitignore();
                }
                5 => {
                    // Health check — fire directly
                    app.ai_mentor_state.last_action = Some("Health Check".to_string());
                    app.start_ai_query("health_check".to_string(), None);
                }
                6 => {
                    // History — switch to history mode
                    app.ai_mentor_state.mode = AiMode::History;
                    app.ai_mentor_state.history_selected = 0;
                    app.ai_mentor_state.history_scroll = 0;
                }
                7 => {
                    // Switch Provider — launch setup wizard
                    app.start_ai_setup();
                }
                _ => {}
            }
        }
        KeyCode::Char('p') => {
            // Quick key to switch provider
            app.start_ai_setup();
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.ai_mentor_state.mode = AiMode::Menu;
        }
        KeyCode::Enter => {
            if app.ai_mentor_state.input.trim().is_empty() {
                return Ok(());
            }
            let query = app.ai_mentor_state.input.clone();
            let action = app.ai_mentor_state.last_action.clone().unwrap_or_default();

            if action.contains("Recommend") {
                app.start_ai_query("recommend".to_string(), Some(query));
            } else if action.contains("Learn") {
                app.start_ai_learn(query);
            } else {
                // "Ask AI" — use the dedicated ask method
                app.start_ai_ask(query);
            }
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.ai_mentor_state.input.push(c);
            }
        }
        KeyCode::Backspace => {
            app.ai_mentor_state.input.pop();
        }
        _ => {}
    }
    Ok(())
}

fn handle_result_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ai_mentor_state.mode = AiMode::Menu;
            app.ai_mentor_state.result_scroll = 0;
        }
        KeyCode::PageDown | KeyCode::Char('j') => {
            app.ai_mentor_state.result_scroll = app.ai_mentor_state.result_scroll.saturating_add(3);
        }
        KeyCode::PageUp | KeyCode::Char('k') => {
            app.ai_mentor_state.result_scroll = app.ai_mentor_state.result_scroll.saturating_sub(3);
        }
        KeyCode::Down => {
            app.ai_mentor_state.result_scroll = app.ai_mentor_state.result_scroll.saturating_add(1);
        }
        KeyCode::Up => {
            app.ai_mentor_state.result_scroll = app.ai_mentor_state.result_scroll.saturating_sub(1);
        }
        _ => {}
    }
    Ok(())
}

fn handle_history_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let history_len = app.ai_mentor_state.history.len();
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ai_mentor_state.mode = AiMode::Menu;
            app.ai_mentor_state.history_scroll = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.ai_mentor_state.history_selected > 0 {
                app.ai_mentor_state.history_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if history_len > 0 && app.ai_mentor_state.history_selected + 1 < history_len {
                app.ai_mentor_state.history_selected += 1;
            }
        }
        KeyCode::Enter => {
            // View selected history entry in the result view
            if history_len > 0 {
                let idx = history_len.saturating_sub(1) - app.ai_mentor_state.history_selected;
                if let Some(entry) = app.ai_mentor_state.history.get(idx) {
                    app.ai_mentor_state.result_text = format!(
                        "── {} ──\n[{}]\n\n{}",
                        entry.query, entry.timestamp, entry.response
                    );
                    app.ai_mentor_state.result_scroll = 0;
                    app.ai_mentor_state.last_action = Some("History".to_string());
                    app.ai_mentor_state.mode = AiMode::Result;
                }
            }
        }
        KeyCode::PageDown => {
            app.ai_mentor_state.history_scroll =
                app.ai_mentor_state.history_scroll.saturating_add(5);
        }
        KeyCode::PageUp => {
            app.ai_mentor_state.history_scroll =
                app.ai_mentor_state.history_scroll.saturating_sub(5);
        }
        _ => {}
    }
    Ok(())
}
