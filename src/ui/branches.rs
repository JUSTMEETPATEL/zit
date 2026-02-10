use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::git;

pub struct BranchesState {
    pub branches: Vec<git::BranchEntry>,
    pub selected: usize,
    pub table_state: TableState,
    pub show_remote: bool,
}

impl Default for BranchesState {
    fn default() -> Self {
        Self {
            branches: Vec::new(),
            selected: 0,
            table_state: TableState::default(),
            show_remote: false,
        }
    }
}

impl BranchesState {
    pub fn refresh(&mut self) {
        if let Ok(branches) = git::BranchOps::list() {
            self.branches = if self.show_remote {
                branches
            } else {
                branches.into_iter().filter(|b| !b.is_remote).collect()
            };
        }
        if self.selected >= self.branches.len() && !self.branches.is_empty() {
            self.selected = self.branches.len() - 1;
        }
        self.table_state.select(if self.branches.is_empty() { None } else { Some(self.selected) });
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut BranchesState) {
    let header_cells = ["", "Branch", "Upstream", "Last Commit", "Author", "Date"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .branches
        .iter()
        .map(|b| {
            let current_marker = if b.is_current { "●" } else { " " };
            let marker_color = if b.is_current { Color::Green } else { Color::DarkGray };
            let name_style = if b.is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if b.is_remote {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(current_marker).style(Style::default().fg(marker_color)),
                Cell::from(b.name.as_str()).style(name_style),
                Cell::from(b.upstream.as_str()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(b.last_commit_msg.as_str()).style(Style::default().fg(Color::White)),
                Cell::from(b.last_commit_author.as_str()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(b.last_commit_date.as_str()).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let remote_indicator = if state.show_remote { " (all) " } else { " (local) " };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(
                format!(" Branches{} ", remote_indicator),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .highlight_style(Style::default().bg(Color::DarkGray))
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut state.table_state);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.branches_state.selected > 0 {
                app.branches_state.selected -= 1;
                let sel = app.branches_state.selected;
                app.branches_state.table_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.branches_state.selected + 1 < app.branches_state.branches.len() {
                app.branches_state.selected += 1;
                let sel = app.branches_state.selected;
                app.branches_state.table_state.select(Some(sel));
            }
        }
        KeyCode::Enter => {
            // Switch to selected branch
            let selected = app.branches_state.selected;
            if let Some(branch) = app.branches_state.branches.get(selected) {
                if branch.is_current {
                    app.set_status("Already on this branch");
                    return Ok(());
                }

                let name = branch.name.clone();

                // Check for uncommitted changes
                if git::BranchOps::has_uncommitted_changes().unwrap_or(false) {
                    app.popup = crate::app::Popup::Message {
                        title: "Warning".to_string(),
                        message: format!(
                            "You have uncommitted changes. Commit or stash them before switching to '{}'.",
                            name
                        ),
                    };
                    return Ok(());
                }

                match git::BranchOps::switch(&name) {
                    Ok(()) => {
                        app.set_status(format!("Switched to '{}'", name));
                        app.branches_state.refresh();
                    }
                    Err(e) => app.set_status(format!("Error: {}", e)),
                }
            }
        }
        KeyCode::Char('n') => {
            app.popup = crate::app::Popup::Input {
                title: "New Branch".to_string(),
                prompt: "Branch name: ".to_string(),
                value: String::new(),
                on_submit: crate::app::InputAction::CreateBranch,
            };
        }
        KeyCode::Char('d') => {
            let selected = app.branches_state.selected;
            if let Some(branch) = app.branches_state.branches.get(selected) {
                if branch.is_current {
                    app.set_status("Cannot delete the current branch");
                    return Ok(());
                }
                let name = branch.name.clone();
                app.popup = crate::app::Popup::Confirm {
                    title: "Delete Branch".to_string(),
                    message: format!(
                        "Are you sure you want to delete '{}' branch?\nThis cannot be undone for unmerged branches.\n\n[y] Yes  [n] No",
                        name
                    ),
                    on_confirm: crate::app::ConfirmAction::DeleteBranch(name),
                };
            }
        }
        KeyCode::Char('R') => {
            app.popup = crate::app::Popup::Input {
                title: "Rename Branch".to_string(),
                prompt: "New name: ".to_string(),
                value: String::new(),
                on_submit: crate::app::InputAction::RenameBranch,
            };
        }
        KeyCode::Tab => {
            app.branches_state.show_remote = !app.branches_state.show_remote;
            app.branches_state.refresh();
        }
        _ => {}
    }

    Ok(())
}
