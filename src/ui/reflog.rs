use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph, Wrap},
    Frame,
};

use crate::git;

pub struct ReflogState {
    pub entries: Vec<git::ReflogEntry>,
    pub selected: usize,
    pub table_state: TableState,
    pub filter_op: Option<String>,
    pub show_diff: bool,
    pub detail_diff: Vec<git::DiffLine>,
    pub detail_scroll: u16,
}

impl Default for ReflogState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selected: 0,
            table_state: TableState::default(),
            filter_op: None,
            show_diff: false,
            detail_diff: Vec::new(),
            detail_scroll: 0,
        }
    }
}

impl ReflogState {
    pub fn refresh(&mut self) {
        match git::reflog::get_reflog(200) {
            Ok(entries) => {
                self.entries = if let Some(ref op) = self.filter_op {
                    git::reflog::filter_reflog(&entries, op)
                } else {
                    entries
                };
                if self.selected >= self.entries.len() && !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                }
                self.table_state.select(if self.entries.is_empty() { None } else { Some(self.selected) });
            }
            Err(_) => {
                self.entries = Vec::new();
            }
        }
    }

    fn load_diff(&mut self) {
        self.detail_diff.clear();
        self.detail_scroll = 0;
        if let Some(entry) = self.entries.get(self.selected) {
            if let Ok(diffs) = git::diff::get_commit_diff(&entry.hash) {
                for fd in &diffs {
                    for hunk in &fd.hunks {
                        self.detail_diff.extend(hunk.lines.clone());
                    }
                }
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut ReflogState) {
    if state.show_diff {
        render_detail(f, area, state);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Table
            Constraint::Length(3), // Hints
        ])
        .split(area);

    let header_cells = ["#", "Hash", "Operation", "Message", "When"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .entries
        .iter()
        .map(|e| {
            let op_color = match e.operation.as_str() {
                "commit" => Color::Green,
                "reset" => Color::Yellow,
                "checkout" => Color::Cyan,
                "merge" => Color::Magenta,
                "rebase" => Color::Red,
                _ => Color::White,
            };

            Row::new(vec![
                Cell::from(format!("{}", e.index)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(e.short_hash.as_str()).style(Style::default().fg(Color::Yellow)),
                Cell::from(e.operation.as_str()).style(Style::default().fg(op_color).add_modifier(Modifier::BOLD)),
                Cell::from(e.message.as_str()).style(Style::default().fg(Color::White)),
                Cell::from(e.date.as_str()).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let filter_text = state.filter_op.as_deref().unwrap_or("all");
    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Percentage(50),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(
                format!(" 🔄 Reflog ({}) ", filter_text),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .highlight_style(Style::default().bg(Color::DarkGray))
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[0], &mut state.table_state);

    // Hints
    let hints = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" View diff "),
        Span::styled("[b]", Style::default().fg(Color::Cyan)),
        Span::raw(" Branch from "),
        Span::styled("[f]", Style::default().fg(Color::Cyan)),
        Span::raw(" Filter "),
        Span::styled("[c]", Style::default().fg(Color::Cyan)),
        Span::raw(" Clear filter "),
        Span::styled("[q]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Back"),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(hints, chunks[1]);
}

fn render_detail(f: &mut Frame, area: Rect, state: &ReflogState) {
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

    let title = if let Some(entry) = state.entries.get(state.selected) {
        format!(" Reflog #{} — {} {} ", entry.index, entry.operation, entry.short_hash)
    } else {
        " Reflog Detail ".to_string()
    };

    let diff = Paragraph::new(diff_lines)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .scroll((state.detail_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(diff, area);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.reflog_state;

    if state.show_diff {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => state.show_diff = false,
            KeyCode::Down | KeyCode::Char('j') => state.detail_scroll = state.detail_scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => state.detail_scroll = state.detail_scroll.saturating_sub(1),
            KeyCode::PageDown => state.detail_scroll = state.detail_scroll.saturating_add(20),
            KeyCode::PageUp => state.detail_scroll = state.detail_scroll.saturating_sub(20),
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.selected > 0 {
                state.selected -= 1;
                state.table_state.select(Some(state.selected));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.selected + 1 < state.entries.len() {
                state.selected += 1;
                state.table_state.select(Some(state.selected));
            }
        }
        KeyCode::Enter => {
            state.load_diff();
            state.show_diff = true;
        }
        KeyCode::Char('b') => {
            if state.entries.get(state.selected).is_some() {
                app.popup = crate::app::Popup::Input {
                    title: "Create Branch from Reflog".to_string(),
                    prompt: "Branch name: ".to_string(),
                    value: String::new(),
                    on_submit: crate::app::InputAction::CreateBranch,
                };
            }
        }
        KeyCode::Char('f') => {
            // Cycle through operation filters
            let next = match state.filter_op.as_deref() {
                None => Some("commit".to_string()),
                Some("commit") => Some("reset".to_string()),
                Some("reset") => Some("checkout".to_string()),
                Some("checkout") => Some("merge".to_string()),
                Some("merge") => Some("rebase".to_string()),
                _ => None,
            };
            state.filter_op = next;
            state.selected = 0;
            state.refresh();
        }
        KeyCode::Char('c') => {
            state.filter_op = None;
            state.selected = 0;
            state.refresh();
        }
        _ => {}
    }

    Ok(())
}
