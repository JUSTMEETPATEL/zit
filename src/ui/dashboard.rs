use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::git;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DashboardFocus {
    #[default]
    Left,
    Right,
}

pub struct DashboardState {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub conflict_count: usize,
    pub stash_count: u32,
    pub commit_count: usize,
    pub is_clean: bool,
    pub recent_commits: Vec<git::CommitEntry>,
    pub error: Option<String>,
    pub focus: DashboardFocus,
}

impl Default for DashboardState {
    fn default() -> Self {
        let mut state = Self {
            branch: String::new(),
            upstream: None,
            ahead: 0,
            behind: 0,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            conflict_count: 0,
            stash_count: 0,
            commit_count: 0,
            is_clean: true,
            recent_commits: Vec::new(),
            error: None,
            focus: DashboardFocus::default(),
        };
        state.refresh();
        state
    }
}

impl DashboardState {
    pub fn refresh(&mut self) {
        match git::status::get_status() {
            Ok(status) => {
                self.branch = status.branch.clone();
                self.upstream = status.upstream.clone();
                self.ahead = status.ahead;
                self.behind = status.behind;
                self.staged_count = status.staged.len();
                self.unstaged_count = status.unstaged.len();
                self.untracked_count = status.untracked.len();
                self.conflict_count = status.conflicts.len();
                self.stash_count = status.stash_count;
                self.is_clean = status.is_clean();
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }

        match git::log::get_recent_commits(5) {
            Ok(commits) => self.recent_commits = commits,
            Err(_) => self.recent_commits = Vec::new(),
        }

        self.commit_count = git::log::commit_count().unwrap_or(0);
    }
}

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &DashboardState,
    status_msg: &Option<String>,
    ai_mentor_state: &crate::ui::ai_mentor::AiMentorState,
    ai_available: bool,
    ai_loading: bool,
    provider_label: &str,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top bar (title + AI title on same row)
            Constraint::Min(5),    // Main content area (dashboard content + AI panel)
            Constraint::Length(3), // Keybindings
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // ── Top bar: split horizontally for dashboard title (left) and AI title (right) ──
    let top_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[0]);

    // Dashboard title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "⚡ zit",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" — Repository Dashboard"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.focus == DashboardFocus::Left {
                Color::Cyan
            } else {
                Color::DarkGray
            })),
    );
    f.render_widget(title, top_panels[0]);

    // AI Mentor title bar
    let ai_status = if ai_loading {
        Span::styled(" ⏳ Loading... ", Style::default().fg(Color::Yellow))
    } else if ai_available {
        Span::styled(" ● Connected ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ Not configured ", Style::default().fg(Color::Red))
    };

    let provider_info = if ai_available && !provider_label.is_empty() {
        Span::styled(
            format!(" [{}] ", provider_label),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    let ai_title = Paragraph::new(Line::from(vec![
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
            .border_style(
                Style::default().fg(if state.focus == DashboardFocus::Right {
                    Color::Magenta
                } else {
                    Color::DarkGray
                }),
            ),
    );
    f.render_widget(ai_title, top_panels[1]);

    // ── Main content area: split horizontally ──
    let content_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[1]);

    // ── Left panel: Dashboard content ──
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Branch info
            Constraint::Length(3), // File counts
            Constraint::Min(5),    // Recent commits
        ])
        .split(content_panels[0]);

    // Branch info
    let status_icon = if state.is_clean { "✓" } else { "✗" };
    let status_color = if state.is_clean {
        Color::Green
    } else {
        Color::Yellow
    };

    let mut branch_spans = vec![
        Span::styled("  Branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &state.branch,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if state.ahead > 0 || state.behind > 0 {
        branch_spans.push(Span::raw("  "));
        if state.ahead > 0 {
            branch_spans.push(Span::styled(
                format!("⬆{}", state.ahead),
                Style::default().fg(Color::Green),
            ));
            branch_spans.push(Span::raw(" "));
        }
        if state.behind > 0 {
            branch_spans.push(Span::styled(
                format!("⬇{}", state.behind),
                Style::default().fg(Color::Red),
            ));
        }
    }

    branch_spans.push(Span::raw("  │  "));
    branch_spans.push(Span::styled(
        format!(
            "{} {}",
            status_icon,
            if state.is_clean { "Clean" } else { "Dirty" }
        ),
        Style::default().fg(status_color),
    ));

    if state.conflict_count > 0 {
        branch_spans.push(Span::raw("  "));
        branch_spans.push(Span::styled(
            format!("⚠ {} conflicts", state.conflict_count),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    let branch_info = Paragraph::new(Line::from(branch_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.focus == DashboardFocus::Left {
                Color::Cyan
            } else {
                Color::DarkGray
            })),
    );
    f.render_widget(branch_info, left_chunks[0]);

    // File counts
    let counts = Paragraph::new(Line::from(vec![
        Span::styled("  Staged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.staged_count),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  │  "),
        Span::styled("Unstaged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.unstaged_count),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  │  "),
        Span::styled("Untracked: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.untracked_count),
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  │  "),
        Span::styled("Stash: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.stash_count),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw("  │  "),
        Span::styled("Commits: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.commit_count),
            Style::default().fg(Color::Blue),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.focus == DashboardFocus::Left {
                Color::Cyan
            } else {
                Color::DarkGray
            })),
    );
    f.render_widget(counts, left_chunks[1]);

    // Recent commits
    let commit_items: Vec<ListItem> = state
        .recent_commits
        .iter()
        .map(|c| {
            let graph_span = if c.graph.is_empty() {
                Span::raw("  ")
            } else {
                Span::styled(format!("{} ", c.graph), Style::default().fg(Color::Magenta))
            };

            if c.hash.is_empty() {
                return ListItem::new(Line::from(vec![graph_span]));
            }

            ListItem::new(Line::from(vec![
                graph_span,
                Span::styled(
                    format!("{} ", c.short_hash),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(&c.message, Style::default().fg(Color::White)),
                Span::styled(
                    format!(" ({})", c.date),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let commits = if commit_items.is_empty() {
        List::new(vec![ListItem::new(Span::styled(
            "  No commits yet",
            Style::default().fg(Color::DarkGray),
        ))])
    } else {
        List::new(commit_items)
    };

    let commits = commits.block(
        Block::default()
            .title(Span::styled(
                " Recent Commits ",
                Style::default().fg(Color::White),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.focus == DashboardFocus::Left {
                Color::Cyan
            } else {
                Color::DarkGray
            })),
    );
    f.render_widget(commits, left_chunks[2]);

    // ── Right panel: AI Mentor content ──
    let ai_content_area = content_panels[1];
    let ai_border_color = if state.focus == DashboardFocus::Right {
        Color::Magenta
    } else {
        Color::DarkGray
    };

    match ai_mentor_state.mode {
        crate::ui::ai_mentor::AiMode::Menu => {
            render_ai_menu(
                f,
                ai_content_area,
                ai_mentor_state,
                ai_available,
                ai_border_color,
            );
        }
        crate::ui::ai_mentor::AiMode::Input => {
            render_ai_input(f, ai_content_area, ai_mentor_state, ai_border_color);
        }
        crate::ui::ai_mentor::AiMode::Result => {
            render_ai_result(f, ai_content_area, ai_mentor_state, ai_border_color);
        }
        crate::ui::ai_mentor::AiMode::History => {
            render_ai_history(f, ai_content_area, ai_mentor_state, ai_border_color);
        }
    }

    // ── Keybindings bar ──
    let key_spans = vec![
        Span::styled(" [s]", Style::default().fg(Color::Cyan)),
        Span::raw(" Stage "),
        Span::styled("[c]", Style::default().fg(Color::Cyan)),
        Span::raw(" Commit "),
        Span::styled("[b]", Style::default().fg(Color::Cyan)),
        Span::raw(" Branches "),
        Span::styled("[l]", Style::default().fg(Color::Cyan)),
        Span::raw(" Log "),
        Span::styled("[t]", Style::default().fg(Color::Cyan)),
        Span::raw(" Time Travel "),
        Span::styled("[r]", Style::default().fg(Color::Cyan)),
        Span::raw(" Reflog "),
        Span::styled("[g]", Style::default().fg(Color::Cyan)),
        Span::raw(" GitHub "),
        Span::styled("[a]", Style::default().fg(Color::Magenta)),
        Span::raw(" AI Focus "),
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch Panel "),
        Span::styled("[m]", Style::default().fg(Color::Red)),
        Span::raw(" Merge "),
        Span::styled("[w]", Style::default().fg(Color::Cyan)),
        Span::raw(" Workflow "),
        Span::styled("[B]", Style::default().fg(Color::Cyan)),
        Span::raw(" Bisect "),
        Span::styled("[p]", Style::default().fg(Color::Magenta)),
        Span::raw(" Cherry Pick "),
        Span::styled("[?]", Style::default().fg(Color::Cyan)),
        Span::raw(" Help "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" Quit"),
    ];

    let keys = Paragraph::new(Line::from(key_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys, main_chunks[2]);

    // ── Status bar ──
    if let Some(msg) = status_msg {
        let status = Paragraph::new(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ));
        f.render_widget(status, main_chunks[3]);
    } else if let Some(err) = &state.error {
        let status = Paragraph::new(Span::styled(
            format!(" Error: {}", err),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(status, main_chunks[3]);
    }
}

fn render_ai_menu(
    f: &mut Frame,
    area: Rect,
    state: &crate::ui::ai_mentor::AiMentorState,
    ai_available: bool,
    border_color: Color,
) {
    use crate::ui::ai_mentor::MENU_ITEMS;

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
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(menu, area);
}

fn render_ai_input(
    f: &mut Frame,
    area: Rect,
    state: &crate::ui::ai_mentor::AiMentorState,
    border_color: Color,
) {
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
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(input_widget, area);
}

fn render_ai_result(
    f: &mut Frame,
    area: Rect,
    state: &crate::ui::ai_mentor::AiMentorState,
    border_color: Color,
) {
    use ratatui::widgets::Wrap;

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
                .border_style(Style::default().fg(border_color)),
        )
        .scroll((state.result_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(result, area);
}

fn render_ai_history(
    f: &mut Frame,
    area: Rect,
    state: &crate::ui::ai_mentor::AiMentorState,
    border_color: Color,
) {
    use ratatui::widgets::Wrap;

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
                .border_style(Style::default().fg(border_color)),
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
                .border_style(Style::default().fg(border_color)),
        )
        .scroll((state.history_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(history_widget, area);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.dashboard_state;

    match key.code {
        KeyCode::Tab | KeyCode::BackTab => {
            state.focus = match state.focus {
                DashboardFocus::Left => DashboardFocus::Right,
                DashboardFocus::Right => DashboardFocus::Left,
            };
            return Ok(());
        }
        _ => {}
    }

    match state.focus {
        DashboardFocus::Left => match key.code {
            KeyCode::Char('a') => {
                state.focus = DashboardFocus::Right;
                return Ok(());
            }
            _ => {}
        },
        DashboardFocus::Right => {
            if let crate::ui::ai_mentor::AiMode::Menu = app.ai_mentor_state.mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        state.focus = DashboardFocus::Left;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            return crate::ui::ai_mentor::handle_key(app, key);
        }
    }

    Ok(())
}
