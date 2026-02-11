use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::git;

pub struct TimelineState {
    pub commits: Vec<git::CommitEntry>,
    pub selected: usize,
    pub list_state: ListState,
    pub detail_commit: Option<git::CommitEntry>,
    pub detail_diff: Vec<git::DiffLine>,
    pub detail_scroll: u16,
    pub search_query: String,
    pub page: usize,
    pub show_detail: bool,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            commits: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            detail_commit: None,
            detail_diff: Vec::new(),
            detail_scroll: 0,
            search_query: String::new(),
            page: 0,
            show_detail: false,
        }
    }
}

impl TimelineState {
    pub fn refresh(&mut self) {
        let count = 100;
        let skip = self.page * count;
        match git::log::get_log(count, skip, None) {
            Ok(commits) => {
                self.commits = commits;
                if self.selected >= self.commits.len() && !self.commits.is_empty() {
                    self.selected = self.commits.len() - 1;
                }
                self.list_state.select(if self.commits.is_empty() { None } else { Some(self.selected) });
            }
            Err(_) => {
                self.commits = Vec::new();
            }
        }
    }

    pub fn do_search(&mut self) {
        if self.search_query.is_empty() {
            self.refresh();
            return;
        }
        match git::log::search_commits(&self.search_query, 100) {
            Ok(commits) => {
                self.commits = commits;
                self.selected = 0;
                self.list_state.select(if self.commits.is_empty() { None } else { Some(0) });
            }
            Err(_) => {}
        }
    }

    fn load_detail(&mut self) {
        if let Some(commit) = self.commits.get(self.selected) {
            if commit.hash.is_empty() {
                self.detail_commit = None;
                self.detail_diff.clear();
                return;
            }

            self.detail_commit = Some(commit.clone());
            self.detail_diff.clear();
            self.detail_scroll = 0;

            if let Ok(diffs) = git::diff::get_commit_diff(&commit.hash) {
                for fd in &diffs {
                    for hunk in &fd.hunks {
                        self.detail_diff.extend(hunk.lines.clone());
                    }
                }
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut TimelineState) {
    if state.show_detail {
        render_detail(f, area, state);
        return;
    }

    // Commit list
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .map(|c| {
            let graph_span = if c.graph.is_empty() {
                Span::raw("")
            } else {
                Span::styled(&c.graph, Style::default().fg(Color::Magenta))
            };

            if c.hash.is_empty() {
                // Graph-only line
                return ListItem::new(Line::from(vec![graph_span]));
            }

            let hash_span = Span::styled(
                format!("{} ", c.short_hash),
                Style::default().fg(Color::Yellow),
            );

            let refs_span = if c.refs.is_empty() {
                Span::raw("")
            } else {
                Span::styled(
                    format!("({}) ", c.refs),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )
            };

            let msg_span = Span::styled(&c.message, Style::default().fg(Color::White));

            let meta_span = Span::styled(
                format!("  {} · {}", c.author, c.date),
                Style::default().fg(Color::DarkGray),
            );

            ListItem::new(Line::from(vec![
                graph_span, hash_span, refs_span, msg_span, meta_span,
            ]))
        })
        .collect();

    let title = if state.search_query.is_empty() {
        format!(" Commit Timeline (page {}) ", state.page + 1)
    } else {
        format!(" Search: '{}' ({} results) ", state.search_query, state.commits.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state.list_state);
}

fn render_detail(f: &mut Frame, area: Rect, state: &TimelineState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Commit info
            Constraint::Min(10),   // Diff
        ])
        .split(area);

    if let Some(commit) = &state.detail_commit {
        let info = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("  Commit: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.hash, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("  Author: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.author, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Date:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.date, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Message: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.message, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
        ])
        .block(
            Block::default()
                .title(Span::styled(" Commit Details ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(info, chunks[0]);
    }

    let diff_lines: Vec<Line> = state
        .detail_diff
        .iter()
        .map(|dl| {
            let color = match dl.line_type {
                git::DiffLineType::Added => Color::Green,
                git::DiffLineType::Removed => Color::Red,
                git::DiffLineType::Header => Color::Cyan,
                git::DiffLineType::Context => Color::DarkGray,
            };
            Line::from(Span::styled(&dl.content, Style::default().fg(color)))
        })
        .collect();

    let diff = Paragraph::new(diff_lines)
        .block(
            Block::default()
                .title(Span::styled(" Diff ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((state.detail_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(diff, chunks[1]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    if app.timeline_state.show_detail {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.timeline_state.show_detail = false;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.timeline_state.detail_scroll = app.timeline_state.detail_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.timeline_state.detail_scroll = app.timeline_state.detail_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.timeline_state.detail_scroll = app.timeline_state.detail_scroll.saturating_add(20);
            }
            KeyCode::PageUp => {
                app.timeline_state.detail_scroll = app.timeline_state.detail_scroll.saturating_sub(20);
            }
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.timeline_state.selected > 0 {
                app.timeline_state.selected -= 1;
                let sel = app.timeline_state.selected;
                app.timeline_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.timeline_state.selected + 1 < app.timeline_state.commits.len() {
                app.timeline_state.selected += 1;
                let sel = app.timeline_state.selected;
                app.timeline_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Enter => {
            if let Some(commit) = app.timeline_state.commits.get(app.timeline_state.selected) {
                 if !commit.hash.is_empty() {
                    app.timeline_state.load_detail();
                    app.timeline_state.show_detail = true;
                 }
            }
        }
        KeyCode::Char('/') => {
            let query = app.timeline_state.search_query.clone();
            app.popup = crate::app::Popup::Input {
                title: "Search Commits".to_string(),
                prompt: "Search: ".to_string(),
                value: query,
                on_submit: crate::app::InputAction::SearchCommits,
            };
        }
        KeyCode::Char('y') => {
            // Copy hash to clipboard
            let selected = app.timeline_state.selected;
            if let Some(commit) = app.timeline_state.commits.get(selected) {
                if !commit.hash.is_empty() {
                    let hash = commit.short_hash.clone();
                    app.set_status(format!("Copied: {}", hash));
                    // In a real app, integrate with a clipboard crate here
                }
            }
        }
        KeyCode::PageDown => {
            app.timeline_state.page += 1;
            app.timeline_state.selected = 0;
            app.timeline_state.refresh();
        }
        KeyCode::PageUp => {
            if app.timeline_state.page > 0 {
                app.timeline_state.page -= 1;
                app.timeline_state.selected = 0;
                app.timeline_state.refresh();
            }
        }
        _ => {}
    }

    Ok(())
}
