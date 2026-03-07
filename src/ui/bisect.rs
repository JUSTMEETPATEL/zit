//! Bisect UI — binary search for the commit that introduced a bug.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::git;

/// Sub-view within the Bisect screen.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BisectMode {
    /// Picking the bad (newest) commit to start bisect.
    PickBad,
    /// Picking the good (oldest) commit to start bisect.
    PickGood,
    /// Bisect session is running — user marks commits good/bad/skip.
    Running,
}

pub struct BisectState {
    /// Current sub-mode of the bisect view.
    mode: BisectMode,
    /// Commits shown in the picker (for start).
    pub commits: Vec<(String, String)>, // (full_hash, message)
    pub selected: usize,
    pub list_state: ListState,
    /// The selected bad commit hash.
    bad_commit: Option<String>,

    /// Current bisect phase from git.
    pub phase: git::bisect::BisectPhase,
    /// Bisect log entries.
    pub log_entries: Vec<git::bisect::BisectLogEntry>,
    /// Log scroll offset.
    pub log_scroll: u16,
    /// Output message from last bisect operation.
    pub last_output: String,
}

impl Default for BisectState {
    fn default() -> Self {
        Self {
            mode: BisectMode::PickBad,
            commits: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            bad_commit: None,
            phase: git::bisect::BisectPhase::Inactive,
            log_entries: Vec::new(),
            log_scroll: 0,
            last_output: String::new(),
        }
    }
}

impl BisectState {
    pub fn refresh(&mut self) {
        self.phase = git::bisect::bisect_status();
        match &self.phase {
            git::bisect::BisectPhase::Inactive => {
                // Load commits for the picker
                self.mode = BisectMode::PickBad;
                self.bad_commit = None;
                self.commits = git::bisect::recent_commits_for_picker(50).unwrap_or_default();
                self.selected = 0;
                self.list_state.select(if self.commits.is_empty() {
                    None
                } else {
                    Some(0)
                });
                self.log_entries.clear();
                self.last_output.clear();
            }
            git::bisect::BisectPhase::InProgress { .. }
            | git::bisect::BisectPhase::Found { .. } => {
                self.mode = BisectMode::Running;
                self.log_entries = git::bisect::parse_bisect_log();
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut BisectState) {
    match state.mode {
        BisectMode::PickBad | BisectMode::PickGood => render_picker(f, area, state),
        BisectMode::Running => render_running(f, area, state),
    }
}

/// Render the commit picker for selecting bad/good commits.
fn render_picker(f: &mut Frame, area: Rect, state: &mut BisectState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions
            Constraint::Min(5),    // Commit list
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Instructions
    let (title, instructions) = match state.mode {
        BisectMode::PickBad => (
            " Bisect — Step 1: Select the BAD commit ",
            "Choose the commit where the bug EXISTS (usually HEAD). Press Enter to confirm.",
        ),
        BisectMode::PickGood => (
            " Bisect — Step 2: Select the GOOD commit ",
            "Choose a commit where the bug does NOT exist. Press Enter to start bisect.",
        ),
        _ => unreachable!(),
    };

    let instruction_widget = Paragraph::new(Line::from(Span::styled(
        instructions,
        Style::default().fg(Color::Yellow),
    )))
    .block(
        Block::default()
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(instruction_widget, chunks[0]);

    // Commit list
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .enumerate()
        .map(|(i, (hash, msg))| {
            let short = &hash[..7.min(hash.len())];
            let marker = if state.mode == BisectMode::PickGood {
                if let Some(bad) = &state.bad_commit {
                    if bad == hash {
                        "✗ "
                    } else if i == state.selected {
                        "▶ "
                    } else {
                        "  "
                    }
                } else {
                    "  "
                }
            } else if i == state.selected {
                "▶ "
            } else {
                "  "
            };

            let hash_color = if state.mode == BisectMode::PickGood
                && state.bad_commit.as_deref() == Some(hash)
            {
                Color::Red
            } else {
                Color::Yellow
            };

            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(
                    format!("{} ", short),
                    Style::default().fg(hash_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Commits ({}) ", state.commits.len()),
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[1], &mut state.list_state);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [↑/↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Select "),
        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
        Span::raw(" Back "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" Dashboard"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys, chunks[2]);
}

/// Render the active bisect session.
fn render_running(f: &mut Frame, area: Rect, state: &mut BisectState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Status panel
            Constraint::Min(5),    // Bisect log
            Constraint::Length(3), // Keybindings
        ])
        .split(area);

    // Status panel
    let status_lines = match &state.phase {
        git::bisect::BisectPhase::InProgress {
            steps_remaining,
            revisions_left,
            current_commit,
        } => {
            vec![
                Line::from(vec![
                    Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "BISECTING",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Testing: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        current_commit,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "  (~{} steps, {} revisions left)",
                            steps_remaining, revisions_left
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(Span::styled(
                    "  Is the bug present in this commit? Mark [g]ood, [b]ad, or [s]kip",
                    Style::default().fg(Color::White),
                )),
            ]
        }
        git::bisect::BisectPhase::Found {
            commit_hash,
            summary,
        } => {
            vec![
                Line::from(vec![
                    Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "FOUND!",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  First bad commit: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        commit_hash,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(summary, Style::default().fg(Color::White)),
                ]),
            ]
        }
        git::bisect::BisectPhase::Inactive => {
            vec![Line::from(Span::styled(
                "  No bisect session active.",
                Style::default().fg(Color::DarkGray),
            ))]
        }
    };

    let status = Paragraph::new(status_lines).block(
        Block::default()
            .title(Span::styled(
                " 🔍 Bisect ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(status, chunks[0]);

    // Bisect log + last output
    let mut log_lines: Vec<Line> = Vec::new();

    if !state.last_output.is_empty() {
        log_lines.push(Line::from(Span::styled(
            format!("  {}", state.last_output),
            Style::default().fg(Color::Yellow),
        )));
        log_lines.push(Line::from(Span::raw("")));
    }

    for entry in &state.log_entries {
        let verdict_color = match entry.verdict.as_str() {
            "good" => Color::Green,
            "bad" => Color::Red,
            "skip" => Color::Yellow,
            _ => Color::White,
        };
        let short_hash = &entry.hash[..7.min(entry.hash.len())];
        log_lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>5} ", entry.verdict),
                Style::default()
                    .fg(verdict_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} ", short_hash),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(&entry.message, Style::default().fg(Color::White)),
        ]));
    }

    if log_lines.is_empty() {
        log_lines.push(Line::from(Span::styled(
            "  No bisect steps recorded yet.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let log_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Bisect Log ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((state.log_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(log_widget, chunks[1]);

    // Keybindings
    let key_spans = match &state.phase {
        git::bisect::BisectPhase::InProgress { .. } => vec![
            Span::styled(" [g]", Style::default().fg(Color::Green)),
            Span::raw(" Good "),
            Span::styled("[b]", Style::default().fg(Color::Red)),
            Span::raw(" Bad "),
            Span::styled("[s]", Style::default().fg(Color::Yellow)),
            Span::raw(" Skip "),
            Span::styled("[R]", Style::default().fg(Color::Red)),
            Span::raw(" Reset "),
            Span::styled("[q]", Style::default().fg(Color::Red)),
            Span::raw(" Dashboard"),
        ],
        git::bisect::BisectPhase::Found { .. } => vec![
            Span::styled(" [R]", Style::default().fg(Color::Cyan)),
            Span::raw(" Reset (end bisect) "),
            Span::styled("[q]", Style::default().fg(Color::Red)),
            Span::raw(" Dashboard"),
        ],
        git::bisect::BisectPhase::Inactive => vec![
            Span::styled(" [q]", Style::default().fg(Color::Red)),
            Span::raw(" Dashboard"),
        ],
    };

    let keys = Paragraph::new(Line::from(key_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys, chunks[2]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let mut status_msg: Option<String> = None;
    let mut ai_error: Option<String> = None;

    {
        let state = &mut app.bisect_state;

        match state.mode {
            BisectMode::PickBad | BisectMode::PickGood => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if state.selected > 0 {
                            state.selected -= 1;
                            state.list_state.select(Some(state.selected));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if state.selected + 1 < state.commits.len() {
                            state.selected += 1;
                            state.list_state.select(Some(state.selected));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some((hash, _)) = state.commits.get(state.selected) {
                            let hash = hash.clone();
                            match state.mode {
                                BisectMode::PickBad => {
                                    state.bad_commit = Some(hash);
                                    state.mode = BisectMode::PickGood;
                                    // Reset selection for good commit picker
                                    state.selected = state.commits.len().saturating_sub(1);
                                    state.list_state.select(Some(state.selected));
                                    status_msg = Some(
                                        "Bad commit selected. Now pick the good commit."
                                            .to_string(),
                                    );
                                }
                                BisectMode::PickGood => {
                                    if let Some(bad) = &state.bad_commit {
                                        let bad = bad.clone();
                                        match git::bisect::bisect_start(&bad, &hash) {
                                            Ok(output) => {
                                                status_msg = Some(format!(
                                                    "Bisect started! {}",
                                                    output.lines().last().unwrap_or("")
                                                ));
                                                state.refresh();
                                            }
                                            Err(e) => {
                                                let err_str = e.to_string();
                                                status_msg = Some(format!(
                                                    "Bisect start failed: {}",
                                                    err_str
                                                ));
                                                ai_error = Some(err_str);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if state.mode == BisectMode::PickGood {
                            // Go back to picking bad commit
                            state.mode = BisectMode::PickBad;
                            state.bad_commit = None;
                            state.selected = 0;
                            state.list_state.select(Some(0));
                        }
                    }
                    _ => {}
                }
            }
            BisectMode::Running => {
                match key.code {
                    KeyCode::Char('g') => {
                        // Mark current commit as good
                        match git::bisect::bisect_good() {
                            Ok(output) => {
                                state.last_output = output.lines().last().unwrap_or("").to_string();
                                status_msg = Some("Marked as GOOD".to_string());
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                status_msg = Some(format!("Error: {}", err_str));
                                ai_error = Some(err_str);
                            }
                        }
                    }
                    KeyCode::Char('b') => {
                        // Mark current commit as bad
                        match git::bisect::bisect_bad() {
                            Ok(output) => {
                                state.last_output = output.lines().last().unwrap_or("").to_string();
                                status_msg = Some("Marked as BAD".to_string());
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                status_msg = Some(format!("Error: {}", err_str));
                                ai_error = Some(err_str);
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        // Skip current commit
                        match git::bisect::bisect_skip() {
                            Ok(output) => {
                                state.last_output = output.lines().last().unwrap_or("").to_string();
                                status_msg = Some("Skipped commit".to_string());
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                status_msg = Some(format!("Error: {}", err_str));
                                ai_error = Some(err_str);
                            }
                        }
                    }
                    KeyCode::Char('R') => {
                        // Reset bisect session
                        match git::bisect::bisect_reset() {
                            Ok(_) => {
                                status_msg = Some("Bisect session ended.".to_string());
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                status_msg = Some(format!("Error: {}", err_str));
                                ai_error = Some(err_str);
                            }
                        }
                    }
                    KeyCode::PageDown => {
                        state.log_scroll = state.log_scroll.saturating_add(5);
                    }
                    KeyCode::PageUp => {
                        state.log_scroll = state.log_scroll.saturating_sub(5);
                    }
                    _ => {}
                }
            }
        }
    } // release borrow

    if let Some(msg) = status_msg {
        app.set_status(&msg);
    }
    if let Some(err) = ai_error {
        app.start_ai_error_explain(err);
    }

    Ok(())
}
