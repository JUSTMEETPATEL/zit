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
    pub display_staged: usize,
    pub display_unstaged: usize,
    pub display_untracked: usize,
    pub display_conflict: usize,
    pub display_stash: u32,
    pub display_commit: usize,
    pub display_ahead: u32,
    pub display_behind: u32,
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
            display_staged: 0,
            display_unstaged: 0,
            display_untracked: 0,
            display_conflict: 0,
            display_stash: 0,
            display_commit: 0,
            display_ahead: 0,
            display_behind: 0,
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

        self.display_staged = self.staged_count;
        self.display_unstaged = self.unstaged_count;
        self.display_untracked = self.untracked_count;
        self.display_conflict = self.conflict_count;
        self.display_stash = self.stash_count;
        self.display_commit = self.commit_count;
        self.display_ahead = self.ahead;
        self.display_behind = self.behind;
    }

    pub fn tick_animations(&mut self) {
        fn step_toward(current: usize, target: usize) -> usize {
            if current < target {
                current + 1
            } else if current > target {
                current - 1
            } else {
                current
            }
        }
        fn step_toward_u32(current: u32, target: u32) -> u32 {
            if current < target {
                current + 1
            } else if current > target {
                current - 1
            } else {
                current
            }
        }

        self.display_staged = step_toward(self.display_staged, self.staged_count);
        self.display_unstaged = step_toward(self.display_unstaged, self.unstaged_count);
        self.display_untracked = step_toward(self.display_untracked, self.untracked_count);
        self.display_conflict = step_toward(self.display_conflict, self.conflict_count);
        self.display_stash = step_toward_u32(self.display_stash, self.stash_count);
        self.display_commit = step_toward(self.display_commit, self.commit_count);
        self.display_ahead = step_toward_u32(self.display_ahead, self.ahead);
        self.display_behind = step_toward_u32(self.display_behind, self.behind);
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
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let top_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[0]);

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

    const SPINNER_FRAMES: &[char] = &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

    let ai_status = if ai_loading {
        let spinner = SPINNER_FRAMES[ai_mentor_state.spinner_frame as usize];
        Span::styled(
            format!(" {} Loading... ", spinner),
            Style::default().fg(Color::Yellow),
        )
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

    let content_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(content_panels[0]);

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

    if state.display_ahead > 0 || state.display_behind > 0 {
        branch_spans.push(Span::raw("  "));
        if state.display_ahead > 0 {
            branch_spans.push(Span::styled(
                format!("⬆{}", state.display_ahead),
                Style::default().fg(Color::Green),
            ));
            branch_spans.push(Span::raw(" "));
        }
        if state.display_behind > 0 {
            branch_spans.push(Span::styled(
                format!("⬇{}", state.display_behind),
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

    if state.display_conflict > 0 {
        branch_spans.push(Span::raw("  "));
        branch_spans.push(Span::styled(
            format!("⚠ {} conflicts", state.display_conflict),
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

    fn gauge_bar(value: usize, max: usize, color: Color) -> Span<'static> {
        let filled = if value == 0 || max == 0 {
            0
        } else {
            ((value as f32 / max as f32) * 6.0).ceil() as usize
        };
        let filled = filled.min(6);
        let bar: String = (0..6).map(|i| if i < filled { '█' } else { '░' }).collect();
        Span::styled(bar, Style::default().fg(color))
    }

    let counts = Paragraph::new(Line::from(vec![
        Span::styled("  Staged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.display_staged),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        gauge_bar(state.display_staged, 10, Color::Green),
        Span::raw("  │  "),
        Span::styled("Unstaged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.display_unstaged),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        gauge_bar(state.display_unstaged, 10, Color::Yellow),
        Span::raw("  │  "),
        Span::styled("Untracked: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.display_untracked),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" "),
        gauge_bar(state.display_untracked, 10, Color::Gray),
        Span::raw("  │  "),
        Span::styled("Stash: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.display_stash),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" "),
        gauge_bar(state.display_stash as usize, 10, Color::Magenta),
        Span::raw("  │  "),
        Span::styled("Commits: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", state.display_commit),
            Style::default().fg(Color::Blue),
        ),
        Span::raw(" "),
        gauge_bar(state.display_commit, 50, Color::Blue),
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
            render_ai_result(
                f,
                ai_content_area,
                ai_mentor_state,
                ai_border_color,
                ai_loading,
            );
        }
        crate::ui::ai_mentor::AiMode::History => {
            render_ai_history(f, ai_content_area, ai_mentor_state, ai_border_color);
        }
    }

    let key_spans = Line::from(vec![
        Span::styled(
            "[s]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(82, 175, 209)),
        ),
        Span::raw("Stage "),
        Span::styled(
            "[c]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(82, 175, 209)),
        ),
        Span::raw("Commit "),
        Span::styled(
            "[b]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(82, 175, 209)),
        ),
        Span::raw("Branches "),
        Span::styled(
            "[l]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(82, 175, 209)),
        ),
        Span::raw("Log "),
        Span::styled(
            "[t]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(155, 114, 215)),
        ),
        Span::raw("TimeTravel "),
        Span::styled(
            "[r]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(155, 114, 215)),
        ),
        Span::raw("Reflog "),
        Span::styled(
            "[g]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(50, 190, 140)),
        ),
        Span::raw("GitHub "),
        Span::styled(
            "[a]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(220, 80, 200)),
        ),
        Span::raw("AI "),
        Span::styled(
            "[m]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(220, 160, 50)),
        ),
        Span::raw("Merge "),
        Span::styled(
            "[p]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(220, 160, 50)),
        ),
        Span::raw("CherryPick "),
        Span::styled(
            "[w]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(50, 190, 140)),
        ),
        Span::raw("Workflow "),
        Span::styled(
            "[B]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(155, 114, 215)),
        ),
        Span::raw("Bisect "),
        Span::styled(
            "[?]",
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(80, 85, 100)),
        ),
        Span::raw("Help "),
        Span::styled(
            "[q]",
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(190, 50, 60)),
        ),
        Span::raw("Quit"),
    ]);

    let keys = Paragraph::new(key_spans)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(keys, main_chunks[2]);

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
    ai_loading: bool,
) {
    use ratatui::widgets::Wrap;

    let title_text = state.last_action.as_deref().unwrap_or("AI Response");

    let visible_chars = state.typewriter_chars;
    let total_chars = state.result_text.chars().count();
    let is_typing = ai_loading && visible_chars < total_chars;

    let visible_text: String = state.result_text.chars().take(visible_chars).collect();

    let mut lines: Vec<Line> = visible_text
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                format!("  {}", l),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    if is_typing {
        if let Some(last_line) = lines.last_mut() {
            last_line.spans.push(Span::styled(
                "▊",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "  ▊",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
    }

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
