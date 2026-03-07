use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::git;

#[derive(Debug, Clone)]
pub struct StagingFile {
    pub path: String,
    pub status: git::FileStatus,
    pub is_staged: bool,
}

#[derive(Default)]
pub struct StagingState {
    pub files: Vec<StagingFile>,
    pub selected: usize,
    pub list_state: ListState,
    pub filter: String,
    pub diff_lines: Vec<git::DiffLine>,
    pub diff_scroll: u16,
    /// Hunk-level staging mode
    pub hunk_mode: bool,
    pub hunk_index: usize,
    pub file_hunks: Vec<git::diff::Hunk>,
}

impl StagingState {
    pub fn refresh(&mut self) {
        let mut files = Vec::new();

        if let Ok(status) = git::status::get_status() {
            for f in &status.staged {
                files.push(StagingFile {
                    path: f.path.clone(),
                    status: f.status.clone(),
                    is_staged: true,
                });
            }
            for f in &status.unstaged {
                // Avoid duplicates (file can be both staged and unstaged)
                if !files.iter().any(|sf| sf.path == f.path && !sf.is_staged) {
                    files.push(StagingFile {
                        path: f.path.clone(),
                        status: f.status.clone(),
                        is_staged: false,
                    });
                }
            }
            for f in &status.untracked {
                files.push(StagingFile {
                    path: f.path.clone(),
                    status: f.status.clone(),
                    is_staged: false,
                });
            }
        }

        self.files = files;
        if self.selected >= self.files.len() && !self.files.is_empty() {
            self.selected = self.files.len() - 1;
        }
        self.list_state.select(if self.files.is_empty() {
            None
        } else {
            Some(self.selected)
        });
        self.update_diff();
    }

    fn filtered_files(&self) -> Vec<(usize, &StagingFile)> {
        self.files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                self.filter.is_empty()
                    || f.path.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .collect()
    }

    fn update_diff(&mut self) {
        self.diff_lines.clear();
        self.diff_scroll = 0;
        self.file_hunks.clear();
        self.hunk_index = 0;

        if let Some(file) = self.files.get(self.selected) {
            let diffs = if file.is_staged {
                git::diff::get_staged_diff().unwrap_or_default()
            } else {
                git::diff::get_unstaged_diff().unwrap_or_default()
            };

            if let Some(fd) = diffs.iter().find(|d| d.path == file.path) {
                self.file_hunks = fd.hunks.clone();
                for hunk in &fd.hunks {
                    self.diff_lines.extend(hunk.lines.clone());
                }
            }
        }
    }

    /// Enter hunk mode for the currently selected file.
    fn enter_hunk_mode(&mut self) {
        if !self.file_hunks.is_empty() {
            self.hunk_mode = true;
            self.hunk_index = 0;
            self.scroll_to_hunk();
        }
    }

    /// Exit hunk mode.
    fn exit_hunk_mode(&mut self) {
        self.hunk_mode = false;
        self.hunk_index = 0;
        self.diff_scroll = 0;
    }

    /// Scroll the diff view so the current hunk header is visible.
    fn scroll_to_hunk(&mut self) {
        if self.hunk_index >= self.file_hunks.len() {
            return;
        }
        // Count lines before the current hunk
        let mut line_offset: u16 = 0;
        for (i, hunk) in self.file_hunks.iter().enumerate() {
            if i == self.hunk_index {
                break;
            }
            line_offset += hunk.lines.len() as u16;
        }
        self.diff_scroll = line_offset;
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut StagingState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // File list
            Constraint::Percentage(60), // Diff preview
        ])
        .split(area);

    // File list — collect into owned data to avoid borrow conflict with list_state
    let filtered: Vec<StagingFile> = state
        .filtered_files()
        .into_iter()
        .map(|(_, f)| f.clone())
        .collect();
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|file| {
            let icon = match file.status {
                git::FileStatus::Modified => "M",
                git::FileStatus::Added => "A",
                git::FileStatus::Deleted => "D",
                git::FileStatus::Renamed => "R",
                git::FileStatus::Copied => "C",
                git::FileStatus::Untracked => "?",
                git::FileStatus::Conflicted => "!",
                git::FileStatus::Ignored => "·",
            };
            let icon_color = match file.status {
                git::FileStatus::Modified => Color::Yellow,
                git::FileStatus::Added => Color::Green,
                git::FileStatus::Deleted => Color::Red,
                git::FileStatus::Untracked => Color::Gray,
                git::FileStatus::Conflicted => Color::Red,
                _ => Color::White,
            };
            let staged_marker = if file.is_staged { "●" } else { "○" };
            let staged_color = if file.is_staged {
                Color::Green
            } else {
                Color::DarkGray
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", staged_marker),
                    Style::default().fg(staged_color),
                ),
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&file.path, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let staged_count = state.files.iter().filter(|f| f.is_staged).count();
    let total = state.files.len();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Files ({}/{} staged) ", staged_count, total),
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // Diff preview
    let diff_items: Vec<Line> = state
        .diff_lines
        .iter()
        .map(|dl| {
            let (color, _prefix) = match dl.line_type {
                git::DiffLineType::Added => (Color::Green, "+"),
                git::DiffLineType::Removed => (Color::Red, "-"),
                git::DiffLineType::Header => (Color::Cyan, "@"),
                git::DiffLineType::Context => (Color::DarkGray, " "),
            };
            Line::from(Span::styled(&dl.content, Style::default().fg(color)))
        })
        .collect();

    let diff_title = if state.hunk_mode {
        let total = state.file_hunks.len();
        let current = state.hunk_index + 1;
        if let Some(file) = state.files.get(state.selected) {
            format!(" Hunk {}/{} — {} ", current, total, file.path)
        } else {
            format!(" Hunk {}/{} ", current, total)
        }
    } else if let Some(file) = state.files.get(state.selected) {
        format!(" Diff: {} ", file.path)
    } else {
        " Diff Preview ".to_string()
    };

    let diff = Paragraph::new(diff_items)
        .block(
            Block::default()
                .title(Span::styled(diff_title, Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((state.diff_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(diff, chunks[1]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    // Collect a status message to set after releasing the staging_state borrow
    let mut status_msg: Option<String> = None;
    let mut ai_error: Option<String> = None;
    let mut ai_review: Option<(String, String)> = None; // (file_path, diff_content)

    {
        let state = &mut app.staging_state;

        // Hunk mode key handling
        if state.hunk_mode {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.hunk_index > 0 {
                        state.hunk_index -= 1;
                        state.scroll_to_hunk();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.hunk_index + 1 < state.file_hunks.len() {
                        state.hunk_index += 1;
                        state.scroll_to_hunk();
                    }
                }
                KeyCode::Char(' ') => {
                    // Stage/unstage current hunk
                    if let Some(file) = state.files.get(state.selected).cloned() {
                        if let Some(hunk) = state.file_hunks.get(state.hunk_index).cloned() {
                            let result = if file.is_staged {
                                git::diff::unstage_hunk(&file.path, &hunk)
                            } else {
                                git::diff::stage_hunk(&file.path, &hunk)
                            };
                            match result {
                                Ok(_) => {
                                    let action = if file.is_staged { "Unstaged" } else { "Staged" };
                                    status_msg =
                                        Some(format!("{} hunk {}", action, state.hunk_index + 1));
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    status_msg = Some(format!("Hunk error: {}", err_str));
                                    ai_error = Some(err_str);
                                }
                            }
                            state.refresh();
                            // Stay in hunk mode if there are still hunks
                            if state.file_hunks.is_empty() {
                                state.exit_hunk_mode();
                            } else if state.hunk_index >= state.file_hunks.len() {
                                state.hunk_index = state.file_hunks.len() - 1;
                                state.scroll_to_hunk();
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('h') => {
                    state.exit_hunk_mode();
                }
                KeyCode::PageDown => {
                    state.diff_scroll = state.diff_scroll.saturating_add(10);
                }
                KeyCode::PageUp => {
                    state.diff_scroll = state.diff_scroll.saturating_sub(10);
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.selected > 0 {
                        state.selected -= 1;
                        state.list_state.select(Some(state.selected));
                        state.update_diff();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.selected + 1 < state.files.len() {
                        state.selected += 1;
                        state.list_state.select(Some(state.selected));
                        state.update_diff();
                    }
                }
                KeyCode::Char(' ') => {
                    // Toggle stage/unstage
                    if let Some(file) = state.files.get(state.selected).cloned() {
                        let result = if file.is_staged {
                            git::run_git(&["restore", "--staged", &file.path])
                        } else {
                            git::run_git(&["add", &file.path])
                        };
                        if let Err(e) = result {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Error: {}", err_str));
                            ai_error = Some(err_str);
                        }
                        state.refresh();
                    }
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Stage all
                    match git::run_git(&["add", "-A"]) {
                        Ok(_) => status_msg = Some("All files staged".to_string()),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Failed to stage: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
                KeyCode::Char('A') => {
                    // Mac-friendly alternative for Ctrl+A (stage all)
                    match git::run_git(&["add", "-A"]) {
                        Ok(_) => status_msg = Some("All files staged".to_string()),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Failed to stage: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // AI diff review for selected file
                    if let Some(file) = state.files.get(state.selected) {
                        if state.diff_lines.is_empty() {
                            status_msg = Some("No diff available for this file".to_string());
                        } else {
                            let diff_content: String = state
                                .diff_lines
                                .iter()
                                .map(|dl| dl.content.as_str())
                                .collect::<Vec<&str>>()
                                .join("\n");
                            ai_review = Some((file.path.clone(), diff_content));
                        }
                    } else {
                        status_msg = Some("No file selected".to_string());
                    }
                }
                KeyCode::Char('R') => {
                    // Mac-friendly alternative for Ctrl+R (AI diff review)
                    if let Some(file) = state.files.get(state.selected) {
                        if state.diff_lines.is_empty() {
                            status_msg = Some("No diff available for this file".to_string());
                        } else {
                            let diff_content: String = state
                                .diff_lines
                                .iter()
                                .map(|dl| dl.content.as_str())
                                .collect::<Vec<&str>>()
                                .join("\n");
                            ai_review = Some((file.path.clone(), diff_content));
                        }
                    } else {
                        status_msg = Some("No file selected".to_string());
                    }
                }
                KeyCode::Char('u') => {
                    // Unstage all
                    match git::run_git(&["reset", "HEAD"]) {
                        Ok(_) => status_msg = Some("All files unstaged".to_string()),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Failed to unstage: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
                KeyCode::Char('/') => {
                    // handled below after borrow is released
                }
                KeyCode::Char('h') => {
                    // Enter hunk mode
                    state.enter_hunk_mode();
                }
                KeyCode::Char('c') => {
                    // handled below after borrow is released
                }
                KeyCode::Char('d') => {
                    // handled below after borrow is released (discard changes)
                }
                KeyCode::PageDown => {
                    state.diff_scroll = state.diff_scroll.saturating_add(10);
                }
                KeyCode::PageUp => {
                    state.diff_scroll = state.diff_scroll.saturating_sub(10);
                }
                _ => {}
            }
        } // close else block for non-hunk mode
    } // release mutable borrow of staging_state

    // Handle actions that need full App access (no staging_state borrow)
    match key.code {
        KeyCode::Char('/') => {
            let filter = app.staging_state.filter.clone();
            app.popup = crate::app::Popup::Input {
                title: "Search Files".to_string(),
                prompt: "Filter: ".to_string(),
                value: filter,
                on_submit: crate::app::InputAction::SearchFiles,
            };
        }
        KeyCode::Char('c') => {
            app.view = crate::app::View::Commit;
            app.commit_state.refresh();
            app.auto_suggest_if_ready();
        }
        KeyCode::Char('d') => {
            // Discard changes for the selected unstaged file
            if let Some(file) = app.staging_state.files.get(app.staging_state.selected) {
                if !file.is_staged {
                    let path = file.path.clone();
                    app.popup = crate::app::Popup::Confirm {
                        title: "Discard Changes".to_string(),
                        message: format!(
                            "Discard all changes to '{}'? This cannot be undone.",
                            path
                        ),
                        on_confirm: crate::app::ConfirmAction::DiscardFile(path),
                    };
                }
            }
        }
        _ => {}
    }

    if let Some(msg) = status_msg {
        app.set_status(&msg);
    }

    if let Some(err) = ai_error {
        app.start_ai_error_explain(err);
    }

    if let Some((file_path, diff_content)) = ai_review {
        app.start_ai_diff_review(file_path, diff_content);
    }

    Ok(())
}
