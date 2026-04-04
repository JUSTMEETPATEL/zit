use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::sync::mpsc;
use std::time::SystemTime;

// ─── Message types ─────────────────────────────────────────────

/// Role of a message in the agent conversation.
#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Agent,
    ToolUse {
        command: String,
        output: String,
        success: bool,
        collapsed: bool,
    },
    System,
    Permission {
        command: String,
        approved: Option<bool>,
    },
}

/// A single message in the agent conversation.
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub role: MessageRole,
    pub content: String,
}

/// A pending command awaiting user permission.
#[derive(Debug, Clone)]
pub struct PendingCommand {
    pub command: Vec<String>,
    pub description: String,
    pub is_destructive: bool,
}

// ─── State ─────────────────────────────────────────────────────

/// State for the Agent view.
pub struct AgentState {
    /// Conversation messages.
    pub messages: Vec<AgentMessage>,
    /// Scroll offset for the conversation area (0 = top).
    pub scroll: u16,
    /// Whether user has manually scrolled (disables auto-scroll).
    pub user_scrolled: bool,
    /// Current user input text.
    pub input: String,
    /// Whether the input bar is active (insert mode).
    pub input_active: bool,
    /// A pending command that needs user approval.
    pub pending_command: Option<PendingCommand>,
    /// Whether the AI is currently thinking.
    pub thinking: bool,
    /// Label for the currently executing command (if any).
    pub executing_label: Option<String>,
    /// If true, auto-approve all commands for this session.
    pub auto_approve: bool,
    /// Remaining tool-use blocks to process from the last AI response.
    pub pending_tool_uses: Vec<(String, Vec<String>)>,
    /// Pending agent text accumulated before/between tool uses.
    pub pending_agent_text: Option<String>,
    /// Input history for Up/Down navigation.
    pub input_history: Vec<String>,
    /// Current position in input history.
    pub history_index: usize,
    /// Receiver for async git command execution results.
    pub command_receiver: Option<mpsc::Receiver<(String, String, bool)>>,
    /// Whether a git command is currently executing asynchronously.
    pub command_executing: bool,
    /// Whether the conversation content has changed (for cached rendering).
    pub dirty: bool,
    /// Cached rendered lines (to avoid rebuilding every frame).
    pub cached_lines: Option<Vec<Line<'static>>>,
    /// Cached total line count.
    pub cached_line_count: usize,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            messages: vec![AgentMessage {
                role: MessageRole::System,
                content: "Agent ready. Describe what you want to do with your repo.".to_string(),
            }],
            scroll: 0,
            user_scrolled: false,
            input: String::new(),
            input_active: true,
            pending_command: None,
            thinking: false,
            executing_label: None,
            auto_approve: false,
            pending_tool_uses: Vec::new(),
            pending_agent_text: None,
            input_history: Vec::new(),
            history_index: 0,
            command_receiver: None,
            command_executing: false,
            dirty: true,
            cached_lines: None,
            cached_line_count: 0,
        }
    }
}

impl AgentState {
    /// Reset state for a new session.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ─── Spinner ───────────────────────────────────────────────────

const SPINNER_FRAMES: &[char] = &[
    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}',
    '\u{2807}', '\u{280F}',
];

fn spinner_char() -> char {
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 100) as usize
        % SPINNER_FRAMES.len();
    SPINNER_FRAMES[idx]
}

// ─── Safe command allowlist ────────────────────────────────────

/// Check if a git command (args after "git") is safe to auto-execute.
pub fn is_safe_command(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    let sub = args[0].as_str();
    match sub {
        "status" | "log" | "diff" | "branch" | "remote" | "stash" | "config" | "ls-files"
        | "describe" | "show" | "rev-parse" | "shortlog" | "tag" | "reflog" | "blame"
        | "ls-tree" | "cat-file" | "count-objects" | "fsck" | "verify-pack" => {
            // Only safe if no write flags present
            !args.iter().any(|a| {
                a == "--delete"
                    || a == "-d"
                    || a == "-D"
                    || a.starts_with("--set-upstream")
                    || a == "--force"
                    || a == "-f"
            })
        }
        _ => false,
    }
}

/// Check if a command is destructive/dangerous.
pub fn is_destructive_command(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    let sub = args[0].as_str();
    let has_force = args.iter().any(|a| {
        a == "--force"
            || a == "-f"
            || a == "--force-with-lease"
            || a == "--hard"
            || a == "--delete"
            || a == "-D"
    });

    matches!(sub, "push" if has_force)
        || matches!(sub, "reset" if has_force)
        || matches!(sub, "clean")
        || matches!(sub, "checkout" if args.iter().any(|a| a == "--force" || a == "-f"))
        || matches!(sub, "rebase")
        || matches!(sub, "filter-branch" | "filter-repo")
}

// ─── Rendering ─────────────────────────────────────────────────

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &mut AgentState,
    ai_available: bool,
    loading: bool,
    provider_label: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(6),    // Conversation area
            Constraint::Length(3), // Input bar
        ])
        .split(area);

    render_title(f, chunks[0], ai_available, loading, provider_label);
    render_conversation(f, chunks[1], state, loading);
    render_input(f, chunks[2], state);
}

fn render_title(
    f: &mut Frame,
    area: Rect,
    ai_available: bool,
    loading: bool,
    provider_label: &str,
) {
    // Get current branch
    let branch = crate::git::branch::BranchOps::current().unwrap_or_else(|_| "unknown".to_string());

    let status_span = if loading {
        Span::styled(" Thinking... ", Style::default().fg(Color::Yellow))
    } else if ai_available {
        Span::styled(" * Connected ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" o Not configured ", Style::default().fg(Color::Red))
    };

    let provider_span = if ai_available && !provider_label.is_empty() {
        Span::styled(
            format!(" [{}] ", provider_label),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Zit Agent ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", branch), Style::default().fg(Color::Magenta)),
        Span::raw(" "),
        status_span,
        provider_span,
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, area);
}

fn render_conversation(f: &mut Frame, area: Rect, state: &mut AgentState, loading: bool) {
    // Rebuild lines when content changed OR when dynamic indicators are active
    // (thinking/executing spinners change every frame, so cache would show stale state)
    let needs_rebuild =
        state.dirty || state.cached_lines.is_none() || loading || state.command_executing;

    let lines: Vec<Line> = if needs_rebuild {
        let built = build_conversation_lines(state, loading);
        state.cached_line_count = built.len();
        state.cached_lines = Some(built.clone());
        if !loading && !state.command_executing {
            state.dirty = false;
        }
        built
    } else {
        state.cached_lines.clone().unwrap_or_default()
    };

    // Scrolling
    let total_lines = lines.len() as u16;
    let visible_height = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(visible_height);

    let effective_scroll = if state.user_scrolled {
        state.scroll.min(max_scroll)
    } else {
        max_scroll
    };

    let conversation = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((effective_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(conversation, area);

    // Scroll indicator (right side)
    if total_lines > visible_height {
        let pct = if max_scroll == 0 {
            1.0
        } else {
            effective_scroll as f64 / max_scroll as f64
        };
        let bar_height = visible_height.max(1);
        let thumb_pos = (pct * (bar_height - 1) as f64).round() as u16;
        let thumb_char = '\u{2588}';
        let track_char = '\u{2591}';

        for y in 0..bar_height {
            let ch = if y == thumb_pos {
                thumb_char
            } else {
                track_char
            };
            let x = area.x + area.width.saturating_sub(1);
            let style = if y == thumb_pos {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            f.buffer_mut()[(x, area.y + 1 + y)]
                .set_char(ch)
                .set_style(style);
        }
    }
}

fn build_conversation_lines(state: &AgentState, loading: bool) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::raw("")));

    for msg in &state.messages {
        match &msg.role {
            MessageRole::System => {
                lines.push(Line::from(Span::styled(
                    format!("  --- {} ---", msg.content),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )));
                lines.push(Line::from(Span::raw("")));
            }
            MessageRole::User => {
                lines.push(Line::from(Span::styled(
                    "  ╭─ You",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                for l in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  │ {}", l),
                        Style::default().fg(Color::White),
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(Span::raw("")));
            }
            MessageRole::Agent => {
                lines.push(Line::from(Span::styled(
                    "  ╭─ Agent",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                for l in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  │ {}", l),
                        Style::default().fg(Color::White),
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(Span::raw("")));
            }
            MessageRole::ToolUse {
                command,
                output,
                success,
                collapsed,
            } => {
                let status_marker = if *success { "✓" } else { "✗" };
                let status_color = if *success { Color::Green } else { Color::Red };
                let toggle = if *collapsed { "▶" } else { "▼" };
                let output_line_count = output.lines().count();

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", status_marker),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(format!("{} ", toggle), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("git {}", command),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" ({} lines)", output_line_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));

                if !*collapsed && !output.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  ┌─────────────────────────────────────────────────",
                        Style::default().fg(Color::DarkGray),
                    )));
                    for l in output.lines().take(30) {
                        let line_style = if l.starts_with('+') {
                            Style::default().fg(Color::Green)
                        } else if l.starts_with('-') {
                            Style::default().fg(Color::Red)
                        } else if l.starts_with("@@")
                            || l.starts_with("diff")
                            || l.starts_with("index")
                        {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        lines.push(Line::from(Span::styled(format!("  │ {}", l), line_style)));
                    }
                    if output_line_count > 30 {
                        lines.push(Line::from(Span::styled(
                            format!(
                                "  │ ... ({} more lines, press Enter to expand)",
                                output_line_count - 30
                            ),
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::ITALIC),
                        )));
                    }
                    lines.push(Line::from(Span::styled(
                        "  └─────────────────────────────────────────────────",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                lines.push(Line::from(Span::raw("")));
            }
            MessageRole::Permission { command, approved } => {
                let (label, color) = match approved {
                    Some(true) => ("ALLOWED", Color::Green),
                    Some(false) => ("DENIED", Color::Red),
                    None => ("PENDING", Color::Yellow),
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  [{}] ", label),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("git {}", command),
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(Span::raw("")));
            }
        }
    }

    // Pending permission dialog
    if let Some(ref pending) = state.pending_command {
        let border_color = if pending.is_destructive {
            Color::Red
        } else {
            Color::Yellow
        };
        let level = if pending.is_destructive {
            "DANGEROUS"
        } else {
            "MODIFY"
        };
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  [{}] ", level),
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Allow: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("git {}", pending.command.join(" ")),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        if !pending.description.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("     {}", pending.description),
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::from(vec![
            Span::styled("     [y]", Style::default().fg(Color::Green)),
            Span::raw(" Allow  "),
            Span::styled("[n]", Style::default().fg(Color::Red)),
            Span::raw(" Deny  "),
            Span::styled("[a]", Style::default().fg(Color::Cyan)),
            Span::raw(" Always allow  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Cancel"),
        ]));
        lines.push(Line::from(Span::raw("")));
    }

    // Thinking indicator — Claude Code style with animated dots
    if loading && state.pending_command.is_none() && !state.command_executing {
        let dots = match (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 500) as usize
            % 4
        {
            0 => "",
            1 => ".",
            2 => "..",
            3 => "...",
            _ => "",
        };
        lines.push(Line::from(vec![
            Span::styled(
                "  ⏳ Thinking",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(dots, Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(Span::raw("")));
    }

    // Executing command indicator
    if let Some(ref label) = state.executing_label {
        let dots = match (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 500) as usize
            % 4
        {
            0 => "",
            1 => ".",
            2 => "..",
            3 => "...",
            _ => "",
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} git {}", spinner_char(), label),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
            ),
            Span::styled(dots, Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(Span::raw("")));
    }

    lines
}

fn render_input(f: &mut Frame, area: Rect, state: &AgentState) {
    let (input_text, input_style) = if state.pending_command.is_some() {
        (
            "  Awaiting permission... [y/n/a]".to_string(),
            Style::default().fg(Color::DarkGray),
        )
    } else if state.thinking {
        (
            "  AI is thinking...".to_string(),
            Style::default().fg(Color::DarkGray),
        )
    } else if state.input_active {
        (
            format!("  > {}_ ", state.input),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "  Press 'i' to type, Esc/q to exit".to_string(),
            Style::default().fg(Color::DarkGray),
        )
    };

    let hints = if state.pending_command.is_some() {
        vec![
            Span::styled(" y ", Style::default().fg(Color::Green)),
            Span::raw("Allow "),
            Span::styled("n ", Style::default().fg(Color::Red)),
            Span::raw("Deny "),
            Span::styled("a ", Style::default().fg(Color::Cyan)),
            Span::raw("Always"),
        ]
    } else if state.input_active {
        vec![
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("Send "),
            Span::styled("↑/↓ ", Style::default().fg(Color::DarkGray)),
            Span::raw("History "),
            Span::styled("Esc ", Style::default().fg(Color::DarkGray)),
            Span::raw("Exit input"),
        ]
    } else {
        vec![
            Span::styled(" i ", Style::default().fg(Color::Cyan)),
            Span::raw("Type "),
            Span::styled("j/k ", Style::default().fg(Color::DarkGray)),
            Span::raw("Scroll "),
            Span::styled("G ", Style::default().fg(Color::DarkGray)),
            Span::raw("Bottom "),
            Span::styled("Ctrl+L ", Style::default().fg(Color::DarkGray)),
            Span::raw("Clear"),
        ]
    };

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let input_widget = Paragraph::new(Line::from(Span::styled(input_text, input_style))).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.input_active {
                Color::Cyan
            } else {
                Color::DarkGray
            })),
    );
    f.render_widget(input_widget, input_chunks[0]);

    let hints_widget = Paragraph::new(Line::from(hints)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(hints_widget, input_chunks[1]);
}

// ─── Key handling ──────────────────────────────────────────────

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    // Handle Ctrl+C: cancel AI request
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        if app.ai_loading {
            app.ai_loading = false;
            app.agent_state.thinking = false;
            app.agent_state.messages.push(AgentMessage {
                role: MessageRole::System,
                content: "Request cancelled.".to_string(),
            });
            app.set_status("Agent request cancelled");
        }
        return Ok(());
    }

    // Handle Ctrl+L: clear conversation
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('l') {
        app.agent_state.reset();
        app.set_status("Agent conversation cleared");
        return Ok(());
    }

    // Handle pending permission dialog
    if app.agent_state.pending_command.is_some() {
        return handle_permission_key(app, key);
    }

    // If input is active, handle text input
    if app.agent_state.input_active {
        return handle_input_key(app, key);
    }

    // Normal mode (input not active)
    handle_normal_key(app, key)
}

fn handle_permission_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Approve this command
            if let Some(pending) = app.agent_state.pending_command.take() {
                app.agent_state.messages.push(AgentMessage {
                    role: MessageRole::Permission {
                        command: pending.command.join(" "),
                        approved: Some(true),
                    },
                    content: String::new(),
                });
                app.execute_agent_command(pending.command);
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            // Deny this command
            if let Some(pending) = app.agent_state.pending_command.take() {
                app.agent_state.messages.push(AgentMessage {
                    role: MessageRole::Permission {
                        command: pending.command.join(" "),
                        approved: Some(false),
                    },
                    content: String::new(),
                });
                app.agent_state.messages.push(AgentMessage {
                    role: MessageRole::System,
                    content: "Command denied. You can ask me to try a different approach."
                        .to_string(),
                });
                // Clear remaining pending tool uses
                app.agent_state.pending_tool_uses.clear();
                app.agent_state.pending_agent_text = None;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            // Auto-approve all for session
            app.agent_state.auto_approve = true;
            if let Some(pending) = app.agent_state.pending_command.take() {
                app.agent_state.messages.push(AgentMessage {
                    role: MessageRole::Permission {
                        command: pending.command.join(" "),
                        approved: Some(true),
                    },
                    content: String::new(),
                });
                app.agent_state.messages.push(AgentMessage {
                    role: MessageRole::System,
                    content: "Auto-approve enabled for this session.".to_string(),
                });
                app.execute_agent_command(pending.command);
            }
        }
        KeyCode::Esc => {
            // Cancel all pending
            app.agent_state.pending_command = None;
            app.agent_state.pending_tool_uses.clear();
            app.agent_state.pending_agent_text = None;
            app.agent_state.messages.push(AgentMessage {
                role: MessageRole::System,
                content: "All pending commands cancelled.".to_string(),
            });
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            if app.agent_state.input.is_empty() {
                app.view = crate::app::View::Dashboard;
                app.agent_state.input_active = false;
            } else {
                app.agent_state.input_active = false;
            }
        }
        KeyCode::Up => {
            if app.agent_state.input.is_empty() && !app.agent_state.input_history.is_empty() {
                if app.agent_state.history_index < app.agent_state.input_history.len() {
                    let idx =
                        app.agent_state.input_history.len() - 1 - app.agent_state.history_index;
                    app.agent_state.input = app.agent_state.input_history[idx].clone();
                    app.agent_state.history_index += 1;
                }
            }
        }
        KeyCode::Down => {
            if app.agent_state.history_index > 0 {
                app.agent_state.history_index -= 1;
                let idx = app.agent_state.input_history.len() - app.agent_state.history_index;
                if idx < app.agent_state.input_history.len() {
                    app.agent_state.input = app.agent_state.input_history[idx].clone();
                } else {
                    app.agent_state.input.clear();
                }
            } else {
                app.agent_state.input.clear();
            }
        }
        KeyCode::Enter => {
            let input = app.agent_state.input.trim().to_string();
            if input.is_empty() {
                return Ok(());
            }
            if !app.agent_state.input_history.contains(&input) {
                app.agent_state.input_history.push(input.clone());
            }
            app.agent_state.history_index = 0;

            app.agent_state.messages.push(AgentMessage {
                role: MessageRole::User,
                content: input.clone(),
            });
            app.agent_state.input.clear();
            app.agent_state.scroll = 0;
            app.agent_state.user_scrolled = false;
            app.agent_state.dirty = true;

            app.start_agent_chat();
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.agent_state.input.push(c);
            }
        }
        KeyCode::Backspace => {
            app.agent_state.input.pop();
        }
        _ => {}
    }
    Ok(())
}

fn handle_normal_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.agent_state;
    match key.code {
        KeyCode::Char('i') => {
            state.input_active = true;
        }
        KeyCode::Char('q') => {
            app.view = crate::app::View::Dashboard;
        }
        KeyCode::Esc => {
            if state.user_scrolled {
                state.scroll = 0;
                state.user_scrolled = false;
            } else {
                app.view = crate::app::View::Dashboard;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll = state.scroll.saturating_add(3);
            state.user_scrolled = true;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll = state.scroll.saturating_sub(3);
            state.user_scrolled = true;
        }
        KeyCode::Char('g') => {
            state.scroll = 0;
            state.user_scrolled = false;
        }
        KeyCode::Char('G') => {
            state.scroll = 0;
            state.user_scrolled = false;
        }
        KeyCode::PageUp => {
            state.scroll = state.scroll.saturating_add(10);
            state.user_scrolled = true;
        }
        KeyCode::PageDown => {
            state.scroll = state.scroll.saturating_sub(10);
            state.user_scrolled = true;
        }
        KeyCode::Enter => {
            toggle_tool_expand(state);
        }
        _ => {}
    }
    Ok(())
}

/// Toggle expand/collapse on the last ToolUse message.
fn toggle_tool_expand(state: &mut AgentState) {
    for msg in state.messages.iter_mut().rev() {
        if let MessageRole::ToolUse { collapsed, .. } = &mut msg.role {
            *collapsed = !*collapsed;
            state.dirty = true;
            break;
        }
    }
}

/// Handle mouse events for scrolling and tool toggle.
pub fn handle_mouse(app: &mut crate::app::App, mouse: MouseEvent) {
    let state = &mut app.agent_state;
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            state.scroll = state.scroll.saturating_add(3);
            state.user_scrolled = true;
        }
        MouseEventKind::ScrollDown => {
            state.scroll = state.scroll.saturating_sub(3);
            state.user_scrolled = true;
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            toggle_tool_expand(state);
        }
        _ => {}
    }
}
