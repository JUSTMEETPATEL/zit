//! Cherry-pick UI — select commits from another branch and apply them.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::git;
use crate::git::log::CommitEntry;

/// Sub-view within the Cherry Pick screen.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CherryPickMode {
    /// Selecting a source branch.
    BranchSelect,
    /// Selecting commits from the chosen branch.
    CommitSelect,
    /// Cherry-pick in progress (conflict or success).
    InProgress,
}

pub struct CherryPickState {
    mode: CherryPickMode,
    /// Available source branches.
    pub branches: Vec<String>,
    pub branch_selected: usize,
    pub branch_list_state: ListState,

    /// The chosen source branch.
    pub source_branch: Option<String>,
    /// Current branch name.
    pub current_branch: String,

    /// Commits from the source branch (not yet on current branch).
    pub commits: Vec<CommitEntry>,
    pub commit_selected: usize,
    pub commit_list_state: ListState,
    /// Multi-select: hashes of commits marked for cherry-pick.
    pub marked: Vec<String>,

    /// Diff preview of the currently highlighted commit.
    pub diff_text: String,
    pub diff_scroll: u16,

    /// Whether a cherry-pick conflict is active.
    pub conflict_active: bool,
}

impl Default for CherryPickState {
    fn default() -> Self {
        Self {
            mode: CherryPickMode::BranchSelect,
            branches: Vec::new(),
            branch_selected: 0,
            branch_list_state: ListState::default(),
            source_branch: None,
            current_branch: String::new(),
            commits: Vec::new(),
            commit_selected: 0,
            commit_list_state: ListState::default(),
            marked: Vec::new(),
            diff_text: String::new(),
            diff_scroll: 0,
            conflict_active: false,
        }
    }
}

impl CherryPickState {
    pub fn refresh(&mut self) {
        self.current_branch = git::cherry_pick::get_current_branch();
        self.conflict_active = git::cherry_pick::is_cherry_picking();

        if self.conflict_active {
            self.mode = CherryPickMode::InProgress;
            return;
        }

        match self.mode {
            CherryPickMode::BranchSelect => {
                self.branches = git::cherry_pick::list_source_branches().unwrap_or_default();
                self.branch_selected = 0;
                self.branch_list_state.select(if self.branches.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            CherryPickMode::CommitSelect => {
                if let Some(ref branch) = self.source_branch {
                    self.commits =
                        git::cherry_pick::get_cherry_candidates(branch, 100).unwrap_or_default();
                    self.commit_selected = 0;
                    self.commit_list_state.select(if self.commits.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
                    self.marked.clear();
                    self.update_diff();
                }
            }
            CherryPickMode::InProgress => {}
        }
    }

    fn update_diff(&mut self) {
        self.diff_text.clear();
        self.diff_scroll = 0;
        if let Some(commit) = self.commits.get(self.commit_selected) {
            if let Ok(diff) = git::cherry_pick::commit_diff(&commit.hash) {
                self.diff_text = diff;
            }
        }
    }

    fn load_commits(&mut self) {
        if let Some(ref branch) = self.source_branch {
            self.commits = git::cherry_pick::get_cherry_candidates(branch, 100).unwrap_or_default();
            self.commit_selected = 0;
            self.commit_list_state.select(if self.commits.is_empty() {
                None
            } else {
                Some(0)
            });
            self.marked.clear();
            self.update_diff();
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut CherryPickState) {
    match state.mode {
        CherryPickMode::BranchSelect => render_branch_select(f, area, state),
        CherryPickMode::CommitSelect => render_commit_select(f, area, state),
        CherryPickMode::InProgress => render_in_progress(f, area, state),
    }
}

/// Render branch selection screen.
fn render_branch_select(f: &mut Frame, area: Rect, state: &mut CherryPickState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Branch list
            Constraint::Length(3), // Keys
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  On branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &state.current_branch,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Select a branch to cherry-pick from",
            Style::default().fg(Color::White),
        ),
    ]))
    .block(
        Block::default()
            .title(Span::styled(
                " 🍒 Cherry Pick ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    f.render_widget(header, chunks[0]);

    // Branch list
    let items: Vec<ListItem> = state
        .branches
        .iter()
        .map(|b| {
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    b,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Branches ({}) ", state.branches.len()),
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

    f.render_stateful_widget(list, chunks[1], &mut state.branch_list_state);

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [↑/↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Select branch "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" Dashboard"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys, chunks[2]);

    // Empty state
    if state.branches.is_empty() {
        let hint = Paragraph::new(Line::from(Span::styled(
            " No other branches found. Create a branch first.",
            Style::default().fg(Color::DarkGray),
        )));
        let hint_area = Rect {
            x: chunks[1].x + 1,
            y: chunks[1].y + 2,
            width: chunks[1].width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(hint, hint_area);
    }
}

/// Render commit selection + diff preview.
fn render_commit_select(f: &mut Frame, area: Rect, state: &mut CherryPickState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Keys
        ])
        .split(area);

    // Header
    let source = state.source_branch.as_deref().unwrap_or("?");
    let marked_count = state.marked.len();
    let header_text = if marked_count > 0 {
        format!(
            "  {} → {} | {} commit{} selected (Space to toggle, Enter to apply)",
            source,
            state.current_branch,
            marked_count,
            if marked_count == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "  {} → {} | Select commits to cherry-pick (Space to mark, Enter for single)",
            source, state.current_branch
        )
    };

    let header = Paragraph::new(Line::from(Span::styled(
        header_text,
        Style::default().fg(Color::White),
    )))
    .block(
        Block::default()
            .title(Span::styled(
                " 🍒 Cherry Pick — Select Commits ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    f.render_widget(header, main_chunks[0]);

    // Content: commit list + diff preview
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45), // Commit list
            Constraint::Percentage(55), // Diff preview
        ])
        .split(main_chunks[1]);

    // Commit list
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .map(|c| {
            let is_marked = state.marked.contains(&c.hash);
            let marker = if is_marked { "● " } else { "  " };
            let marker_color = if is_marked {
                Color::Green
            } else {
                Color::DarkGray
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    marker,
                    Style::default()
                        .fg(marker_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", c.short_hash),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&c.message, Style::default().fg(Color::White)),
                Span::styled(
                    format!(" ({})", c.date),
                    Style::default().fg(Color::DarkGray),
                ),
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

    f.render_stateful_widget(list, content_chunks[0], &mut state.commit_list_state);

    // Diff preview
    let diff_lines: Vec<Line> = state
        .diff_text
        .lines()
        .map(|line| {
            let color = crate::ui::utils::diff_line_color(line);
            Line::from(Span::styled(line, Style::default().fg(color)))
        })
        .collect();

    let diff_title = if let Some(c) = state.commits.get(state.commit_selected) {
        format!(" {} — {} ", c.short_hash, c.message)
    } else {
        " Diff Preview ".to_string()
    };

    let diff = Paragraph::new(diff_lines)
        .block(
            Block::default()
                .title(Span::styled(diff_title, Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((state.diff_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(diff, content_chunks[1]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [↑/↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate "),
        Span::styled("[Space]", Style::default().fg(Color::Cyan)),
        Span::raw(" Mark "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Apply "),
        Span::styled("[PgDn/Up]", Style::default().fg(Color::Cyan)),
        Span::raw(" Scroll diff "),
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
    f.render_widget(keys, main_chunks[2]);

    // Empty state
    if state.commits.is_empty() {
        let hint = Paragraph::new(Line::from(Span::styled(
            format!(
                " No unique commits on {} (already merged or same history).",
                state.source_branch.as_deref().unwrap_or("?")
            ),
            Style::default().fg(Color::DarkGray),
        )));
        let hint_area = Rect {
            x: content_chunks[0].x + 1,
            y: content_chunks[0].y + 2,
            width: content_chunks[0].width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(hint, hint_area);
    }
}

/// Render the in-progress / conflict screen.
fn render_in_progress(f: &mut Frame, area: Rect, state: &mut CherryPickState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Status
            Constraint::Min(5),    // Diff / conflict info
            Constraint::Length(3), // Keys
        ])
        .split(area);

    // Status
    let status_lines = if state.conflict_active {
        vec![
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "CONFLICT",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "  Cherry-pick has conflicts. Resolve them and press [c] to continue or [A] to abort.",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  Tip: Switch to Merge Resolve view (Esc → m) for guided conflict resolution.",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![Line::from(Span::styled(
            "  Cherry-pick completed successfully!",
            Style::default().fg(Color::Green),
        ))]
    };

    let status = Paragraph::new(status_lines).block(
        Block::default()
            .title(Span::styled(
                " 🍒 Cherry Pick — In Progress ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    f.render_widget(status, chunks[0]);

    // Show git status
    let status_output =
        git::run_git(&["status", "--short"]).unwrap_or_else(|_| "Unable to get status".to_string());
    let status_lines: Vec<Line> = status_output
        .lines()
        .map(|line| {
            let color = if line.starts_with("UU") || line.starts_with("AA") {
                Color::Red
            } else if line.starts_with("M") || line.starts_with("A") {
                Color::Green
            } else {
                Color::White
            };
            Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(color),
            ))
        })
        .collect();

    let status_widget = Paragraph::new(status_lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Working Tree Status ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(status_widget, chunks[1]);

    // Keys
    let keys = if state.conflict_active {
        Paragraph::new(Line::from(vec![
            Span::styled(" [c]", Style::default().fg(Color::Green)),
            Span::raw(" Continue "),
            Span::styled("[A]", Style::default().fg(Color::Red)),
            Span::raw(" Abort "),
            Span::styled("[q]", Style::default().fg(Color::Red)),
            Span::raw(" Dashboard"),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" [q]", Style::default().fg(Color::Red)),
            Span::raw(" Dashboard"),
        ]))
    };

    let keys = keys.block(
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
        let state = &mut app.cherry_pick_state;

        match state.mode {
            CherryPickMode::BranchSelect => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.branch_selected > 0 {
                        state.branch_selected -= 1;
                        state.branch_list_state.select(Some(state.branch_selected));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.branch_selected + 1 < state.branches.len() {
                        state.branch_selected += 1;
                        state.branch_list_state.select(Some(state.branch_selected));
                    }
                }
                KeyCode::Enter => {
                    if let Some(branch) = state.branches.get(state.branch_selected) {
                        state.source_branch = Some(branch.clone());
                        state.mode = CherryPickMode::CommitSelect;
                        state.load_commits();
                    }
                }
                _ => {}
            },

            CherryPickMode::CommitSelect => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.commit_selected > 0 {
                        state.commit_selected -= 1;
                        state.commit_list_state.select(Some(state.commit_selected));
                        state.update_diff();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.commit_selected + 1 < state.commits.len() {
                        state.commit_selected += 1;
                        state.commit_list_state.select(Some(state.commit_selected));
                        state.update_diff();
                    }
                }
                KeyCode::Char(' ') => {
                    // Toggle mark on current commit
                    if let Some(commit) = state.commits.get(state.commit_selected) {
                        let hash = commit.hash.clone();
                        if let Some(pos) = state.marked.iter().position(|h| *h == hash) {
                            state.marked.remove(pos);
                        } else {
                            state.marked.push(hash);
                        }
                    }
                }
                KeyCode::Enter => {
                    if !state.marked.is_empty() {
                        // Cherry-pick all marked commits (in reverse order for chronological application)
                        let hashes: Vec<String> = state.marked.iter().rev().cloned().collect();
                        let hash_refs: Vec<&str> = hashes.iter().map(|s| s.as_str()).collect();
                        match git::cherry_pick::cherry_pick_multiple(&hash_refs) {
                            Ok(_) => {
                                status_msg = Some(format!(
                                    "Cherry-picked {} commit{}!",
                                    hashes.len(),
                                    if hashes.len() == 1 { "" } else { "s" }
                                ));
                                state.mode = CherryPickMode::BranchSelect;
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                if err_str.contains("conflict") || err_str.contains("CONFLICT") {
                                    state.conflict_active = true;
                                    state.mode = CherryPickMode::InProgress;
                                    status_msg = Some(
                                        "Cherry-pick has conflicts. Resolve and continue."
                                            .to_string(),
                                    );
                                } else {
                                    status_msg = Some(format!("Cherry-pick failed: {}", err_str));
                                    ai_error = Some(err_str);
                                }
                            }
                        }
                    } else if let Some(commit) = state.commits.get(state.commit_selected) {
                        // Cherry-pick single commit
                        let hash = commit.hash.clone();
                        let short = commit.short_hash.clone();
                        match git::cherry_pick::cherry_pick(&hash) {
                            Ok(_) => {
                                status_msg = Some(format!("Cherry-picked {} ✓", short));
                                state.mode = CherryPickMode::BranchSelect;
                                state.refresh();
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                if err_str.contains("conflict") || err_str.contains("CONFLICT") {
                                    state.conflict_active = true;
                                    state.mode = CherryPickMode::InProgress;
                                    status_msg = Some(
                                        "Cherry-pick has conflicts. Resolve and continue."
                                            .to_string(),
                                    );
                                } else {
                                    status_msg = Some(format!("Cherry-pick failed: {}", err_str));
                                    ai_error = Some(err_str);
                                }
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    // Go back to branch selection
                    state.mode = CherryPickMode::BranchSelect;
                    state.source_branch = None;
                    state.commits.clear();
                    state.marked.clear();
                    state.diff_text.clear();
                    state.refresh();
                }
                KeyCode::PageDown => {
                    state.diff_scroll = state.diff_scroll.saturating_add(10);
                }
                KeyCode::PageUp => {
                    state.diff_scroll = state.diff_scroll.saturating_sub(10);
                }
                _ => {}
            },

            CherryPickMode::InProgress => match key.code {
                KeyCode::Char('c') => match git::cherry_pick::cherry_pick_continue() {
                    Ok(_) => {
                        status_msg = Some("Cherry-pick continued ✓".to_string());
                        state.conflict_active = false;
                        state.mode = CherryPickMode::BranchSelect;
                        state.refresh();
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        status_msg = Some(format!("Continue failed: {}", err_str));
                        ai_error = Some(err_str);
                    }
                },
                KeyCode::Char('A') => match git::cherry_pick::cherry_pick_abort() {
                    Ok(_) => {
                        status_msg = Some("Cherry-pick aborted.".to_string());
                        state.conflict_active = false;
                        state.mode = CherryPickMode::BranchSelect;
                        state.refresh();
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        status_msg = Some(format!("Abort failed: {}", err_str));
                        ai_error = Some(err_str);
                    }
                },
                _ => {}
            },
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
