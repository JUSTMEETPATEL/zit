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
pub struct TimeTravelState {
    pub commits: Vec<git::CommitEntry>,
    pub selected: usize,
    pub list_state: ListState,
    pub ai_suggestion: Option<String>,
    pub ai_loading: bool,
    pub ai_scroll: u16,
}

impl TimeTravelState {
    pub fn refresh(&mut self) {
        match git::log::get_log(50, 0, None) {
            Ok(commits) => {
                self.commits = commits;
                if self.selected >= self.commits.len() && !self.commits.is_empty() {
                    self.selected = self.commits.len() - 1;
                }
                self.list_state.select(if self.commits.is_empty() {
                    None
                } else {
                    Some(self.selected)
                });
            }
            Err(_) => {
                self.commits = Vec::new();
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut TimeTravelState) {
    let has_ai = state.ai_suggestion.is_some() || state.ai_loading;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_ai {
            vec![
                Constraint::Percentage(40), // Commit list
                Constraint::Percentage(50), // AI suggestion panel
                Constraint::Length(5),      // Action hints
            ]
        } else {
            vec![
                Constraint::Min(10),   // Commit list
                Constraint::Length(0), // No AI panel
                Constraint::Length(5), // Action hints
            ]
        })
        .split(area);

    // Commit list
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .map(|c| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", c.short_hash),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(&c.message, Style::default().fg(Color::White)),
                Span::styled(
                    format!("  {}", c.date),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    " ⏪ Time Travel — Select a commit ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ))
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

    // AI suggestion panel
    if has_ai {
        let ai_content = if state.ai_loading {
            vec![Line::from(Span::styled(
                "  ⏳ AI is analyzing reset options...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))]
        } else if let Some(ref suggestion) = state.ai_suggestion {
            suggestion
                .lines()
                .map(|line| {
                    let color = if line.starts_with("##") || line.starts_with("**") {
                        Color::Cyan
                    } else if line.contains("--soft") {
                        Color::Green
                    } else if line.contains("--mixed") {
                        Color::Yellow
                    } else if line.contains("--hard") {
                        Color::Red
                    } else if line.contains("recommend") || line.contains("Recommend") {
                        Color::Magenta
                    } else {
                        Color::White
                    };
                    Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(color),
                    ))
                })
                .collect()
        } else {
            vec![]
        };

        let ai_panel = Paragraph::new(ai_content)
            .block(
                Block::default()
                    .title(Span::styled(
                        " 🤖 AI Reset Insight — [Esc] dismiss ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .scroll((state.ai_scroll, 0))
            .wrap(Wrap { trim: false });

        f.render_widget(ai_panel, chunks[1]);
    }

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
            Span::styled("[i]", Style::default().fg(Color::Magenta)),
            Span::raw(" AI Insight "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Back"),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(
                " Actions ",
                Style::default().fg(Color::DarkGray),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(hints, chunks[2]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.time_travel_state;

    // If AI panel is visible, handle scroll/dismiss first
    if state.ai_suggestion.is_some() {
        match key.code {
            KeyCode::Esc => {
                state.ai_suggestion = None;
                state.ai_scroll = 0;
                return Ok(());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.ai_scroll = state.ai_scroll.saturating_add(1);
                return Ok(());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.ai_scroll = state.ai_scroll.saturating_sub(1);
                return Ok(());
            }
            KeyCode::PageDown => {
                state.ai_scroll = state.ai_scroll.saturating_add(10);
                return Ok(());
            }
            KeyCode::PageUp => {
                state.ai_scroll = state.ai_scroll.saturating_sub(10);
                return Ok(());
            }
            _ => {
                // Let other keys pass through (s/m/h for reset, etc.)
            }
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if state.ai_suggestion.is_none() => {
            if state.selected > 0 {
                state.selected -= 1;
                state.list_state.select(Some(state.selected));
            }
        }
        KeyCode::Down | KeyCode::Char('j') if state.ai_suggestion.is_none() => {
            if state.selected + 1 < state.commits.len() {
                state.selected += 1;
                state.list_state.select(Some(state.selected));
            }
        }
        KeyCode::Char('i') => {
            // AI reset insight
            if let Some(commit) = state.commits.get(state.selected) {
                let target_hash = commit.short_hash.clone();
                let target_msg = commit.message.clone();
                let commits_back = state.selected;

                // Get current HEAD hash
                let current_hash = state
                    .commits
                    .first()
                    .map(|c| c.short_hash.clone())
                    .unwrap_or_else(|| "HEAD".to_string());

                app.start_ai_reset_suggest(current_hash, target_hash, target_msg, commits_back);
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
        KeyCode::Char('f') => {
            // Restore a specific file from the selected commit
            if let Some(commit) = state.commits.get(state.selected) {
                let hash = commit.short_hash.clone();
                app.popup = crate::app::Popup::Input {
                    title: format!("Restore File from {}", hash),
                    prompt: "File path to restore: ".to_string(),
                    value: String::new(),
                    on_submit: crate::app::InputAction::SearchFiles,
                };
            }
        }
        _ => {}
    }

    Ok(())
}
