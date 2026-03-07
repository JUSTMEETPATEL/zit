//! Stash management UI — list, push, pop, apply, drop stash entries.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::git;

#[derive(Default)]
pub struct StashState {
    pub entries: Vec<git::stash::StashEntry>,
    pub selected: usize,
    pub list_state: ListState,
    pub diff_text: String,
    pub diff_scroll: u16,
}

impl StashState {
    pub fn refresh(&mut self) {
        self.entries = git::stash::list_stashes().unwrap_or_default();
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
        self.list_state.select(if self.entries.is_empty() {
            None
        } else {
            Some(self.selected)
        });
        self.update_diff();
    }

    fn update_diff(&mut self) {
        self.diff_text.clear();
        self.diff_scroll = 0;

        if let Some(entry) = self.entries.get(self.selected) {
            if let Ok(diff) = git::stash::stash_show(entry.index) {
                self.diff_text = diff;
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut StashState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Stash list
            Constraint::Percentage(60), // Diff preview
        ])
        .split(area);

    // Stash list
    let items: Vec<ListItem> = state
        .entries
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", entry.index),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if entry.branch.is_empty() {
                        String::new()
                    } else {
                        format!("[{}] ", entry.branch)
                    },
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let title = format!(" Stash ({}) ", state.entries.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // Diff preview — use structured diff coloring
    let diff_lines: Vec<Line> = state
        .diff_text
        .lines()
        .map(|line| {
            let color = crate::ui::utils::diff_line_color(line);
            Line::from(Span::styled(line, Style::default().fg(color)))
        })
        .collect();

    let diff_title = if let Some(entry) = state.entries.get(state.selected) {
        format!(" stash@{{{}}} ", entry.index)
    } else {
        " Stash Diff ".to_string()
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

    f.render_widget(diff, chunks[1]);

    // Keybinding hints at bottom if area is big enough
    if area.height > 5 && state.entries.is_empty() {
        let hint = Paragraph::new(Line::from(vec![
            Span::styled(
                " No stash entries. Press ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("n", Style::default().fg(Color::Yellow)),
            Span::styled(
                " to stash current changes.",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        // Render hint centered in the list area
        let hint_area = Rect {
            x: chunks[0].x + 1,
            y: chunks[0].y + 2,
            width: chunks[0].width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(hint, hint_area);
    }
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let mut status_msg: Option<String> = None;
    let mut ai_error: Option<String> = None;

    {
        let state = &mut app.stash_state;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if state.selected > 0 {
                    state.selected -= 1;
                    state.list_state.select(Some(state.selected));
                    state.update_diff();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.selected + 1 < state.entries.len() {
                    state.selected += 1;
                    state.list_state.select(Some(state.selected));
                    state.update_diff();
                }
            }
            KeyCode::Char('p') => {
                // Pop stash
                if let Some(entry) = state.entries.get(state.selected) {
                    match git::stash::stash_pop(entry.index) {
                        Ok(_) => status_msg = Some(format!("Popped stash@{{{}}}", entry.index)),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Pop failed: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
            }
            KeyCode::Char('a') => {
                // Apply stash (keep in list)
                if let Some(entry) = state.entries.get(state.selected) {
                    match git::stash::stash_apply(entry.index) {
                        Ok(_) => status_msg = Some(format!("Applied stash@{{{}}}", entry.index)),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Apply failed: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
            }
            KeyCode::Char('d') => {
                // Drop stash
                if let Some(entry) = state.entries.get(state.selected) {
                    match git::stash::stash_drop(entry.index) {
                        Ok(_) => status_msg = Some(format!("Dropped stash@{{{}}}", entry.index)),
                        Err(e) => {
                            let err_str = e.to_string();
                            status_msg = Some(format!("Drop failed: {}", err_str));
                            ai_error = Some(err_str);
                        }
                    }
                    state.refresh();
                }
            }
            KeyCode::Char('n') => {
                // handled below (needs popup for message input)
            }
            KeyCode::Char('D') => {
                // Clear all stashes — handled below (needs confirm popup)
            }
            KeyCode::PageDown => {
                state.diff_scroll = state.diff_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                state.diff_scroll = state.diff_scroll.saturating_sub(10);
            }
            _ => {}
        }
    } // release mutable borrow

    // Actions needing full App access
    match key.code {
        KeyCode::Char('n') => {
            app.popup = crate::app::Popup::Input {
                title: "Stash Push".to_string(),
                prompt: "Message (empty for default): ".to_string(),
                value: String::new(),
                on_submit: crate::app::InputAction::StashPush,
            };
        }
        KeyCode::Char('D') => {
            if !app.stash_state.entries.is_empty() {
                app.popup = crate::app::Popup::Confirm {
                    title: "Clear All Stashes".to_string(),
                    message: format!(
                        "Drop all {} stash entries? This cannot be undone.",
                        app.stash_state.entries.len()
                    ),
                    on_confirm: crate::app::ConfirmAction::ClearStash,
                };
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

    Ok(())
}
