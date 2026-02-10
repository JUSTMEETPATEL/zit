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

pub struct StagingState {
    pub files: Vec<StagingFile>,
    pub selected: usize,
    pub list_state: ListState,
    pub filter: String,
    pub show_diff: bool,
    pub diff_lines: Vec<git::DiffLine>,
    pub diff_scroll: u16,
}

impl Default for StagingState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            filter: String::new(),
            show_diff: true,
            diff_lines: Vec::new(),
            diff_scroll: 0,
        }
    }
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
        self.list_state.select(if self.files.is_empty() { None } else { Some(self.selected) });
        self.update_diff();
    }

    fn filtered_files(&self) -> Vec<(usize, &StagingFile)> {
        self.files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                self.filter.is_empty() || f.path.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .collect()
    }

    fn update_diff(&mut self) {
        self.diff_lines.clear();
        self.diff_scroll = 0;

        if let Some(file) = self.files.get(self.selected) {
            let diffs = if file.is_staged {
                git::diff::get_staged_diff().unwrap_or_default()
            } else {
                git::diff::get_unstaged_diff().unwrap_or_default()
            };

            if let Some(fd) = diffs.iter().find(|d| d.path == file.path) {
                for hunk in &fd.hunks {
                    self.diff_lines.extend(hunk.lines.clone());
                }
            }
        }
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
    let filtered: Vec<StagingFile> = state.filtered_files().into_iter().map(|(_, f)| f.clone()).collect();
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
            let staged_color = if file.is_staged { Color::Green } else { Color::DarkGray };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", staged_marker), Style::default().fg(staged_color)),
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color).add_modifier(Modifier::BOLD)),
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
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
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

    let diff_title = if let Some(file) = state.files.get(state.selected) {
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
    let state = &mut app.staging_state;

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
                if file.is_staged {
                    let _ = git::run_git(&["restore", "--staged", &file.path]);
                } else {
                    let _ = git::run_git(&["add", &file.path]);
                }
                state.refresh();
            }
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Stage all
            let _ = git::run_git(&["add", "-A"]);
            state.refresh();
            app.set_status("All files staged");
        }
        KeyCode::Char('u') => {
            // Unstage all
            let _ = git::run_git(&["reset", "HEAD"]);
            state.refresh();
            app.set_status("All files unstaged");
        }
        KeyCode::Char('/') => {
            app.popup = crate::app::Popup::Input {
                title: "Search Files".to_string(),
                prompt: "Filter: ".to_string(),
                value: state.filter.clone(),
                on_submit: crate::app::InputAction::SearchFiles,
            };
        }
        KeyCode::Char('c') => {
            // Jump to commit view
            app.view = crate::app::View::Commit;
            app.commit_state.refresh();
        }
        KeyCode::PageDown => {
            state.diff_scroll = state.diff_scroll.saturating_add(10);
        }
        KeyCode::PageUp => {
            state.diff_scroll = state.diff_scroll.saturating_sub(10);
        }
        _ => {}
    }

    Ok(())
}
