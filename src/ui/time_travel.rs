use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::git;

pub struct TimeTravelState {
    pub commits: Vec<git::CommitEntry>,
    pub selected: usize,
    pub list_state: ListState,
}

impl Default for TimeTravelState {
    fn default() -> Self {
        Self {
            commits: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
        }
    }
}

impl TimeTravelState {
    pub fn refresh(&mut self) {
        match git::log::get_log(50, 0, None) {
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
}

pub fn render(f: &mut Frame, area: Rect, state: &mut TimeTravelState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Commit list
            Constraint::Length(5),  // Action hints
        ])
        .split(area);

    // Commit list
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .map(|c| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", c.short_hash), Style::default().fg(Color::Yellow)),
                Span::styled(&c.message, Style::default().fg(Color::White)),
                Span::styled(format!("  {}", c.date), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    " ⏪ Time Travel — Select a commit ",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // Action hints
    let hints = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" [s]", Style::default().fg(Color::Green)),
            Span::raw(" Soft Reset (safe) "),
            Span::styled("[m]", Style::default().fg(Color::Yellow)),
            Span::raw(" Mixed Reset "),
            Span::styled("[h]", Style::default().fg(Color::Red)),
            Span::raw(" Hard Reset (⚠ destructive) "),
        ]),
        Line::from(vec![
            Span::styled(" [b]", Style::default().fg(Color::Cyan)),
            Span::raw(" Create Branch "),
            Span::styled("[f]", Style::default().fg(Color::Cyan)),
            Span::raw(" Restore File "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Back"),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(" Actions ", Style::default().fg(Color::DarkGray)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(hints, chunks[1]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.time_travel_state;

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
        KeyCode::Char('s') => {
            // Soft reset
            if let Some(commit) = state.commits.get(state.selected) {
                let hash = commit.hash.clone();
                let short = &commit.short_hash;
                app.popup = crate::app::Popup::Confirm {
                    title: "Soft Reset".to_string(),
                    message: format!(
                        "Soft reset to {}?\n\nThis will move HEAD back but keep all changes staged.\nYour working files will NOT be modified.\n\n[y] Yes  [n] No",
                        short
                    ),
                    on_confirm: crate::app::ConfirmAction::SoftReset(hash),
                };
            }
        }
        KeyCode::Char('m') => {
            // Mixed reset
            if let Some(commit) = state.commits.get(state.selected) {
                let hash = commit.hash.clone();
                let short = &commit.short_hash;
                app.popup = crate::app::Popup::Confirm {
                    title: "Mixed Reset".to_string(),
                    message: format!(
                        "Mixed reset to {}?\n\nThis will move HEAD back and unstage changes.\nYour working files will NOT be modified.\n\n[y] Yes  [n] No",
                        short
                    ),
                    on_confirm: crate::app::ConfirmAction::MixedReset(hash),
                };
            }
        }
        KeyCode::Char('h') => {
            // Hard reset - 2-step confirmation
            if let Some(commit) = state.commits.get(state.selected) {
                let hash = commit.hash.clone();
                let short = &commit.short_hash;
                app.popup = crate::app::Popup::Confirm {
                    title: "⚠ HARD RESET — DESTRUCTIVE".to_string(),
                    message: format!(
                        "Hard reset to {}?\n\n⚠ WARNING: This will PERMANENTLY DELETE all uncommitted changes!\n⚠ All staged and unstaged work will be LOST.\n⚠ This cannot be undone (but lost commits may be in reflog).\n\nAre you ABSOLUTELY sure? [y] Yes  [n] No",
                        short
                    ),
                    on_confirm: crate::app::ConfirmAction::HardReset(hash),
                };
            }
        }
        KeyCode::Char('b') => {
            // Create branch from selected commit
            if let Some(_commit) = state.commits.get(state.selected) {
                app.popup = crate::app::Popup::Input {
                    title: "Create Branch".to_string(),
                    prompt: "Branch name: ".to_string(),
                    value: String::new(),
                    on_submit: crate::app::InputAction::CreateBranch,
                };
            }
        }
        _ => {}
    }

    Ok(())
}
