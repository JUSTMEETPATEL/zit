use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::git;

pub struct CommitState {
    pub message: String,
    pub cursor_pos: usize,
    pub staged_files: Vec<git::FileEntry>,
    pub stat_output: String,
    pub editing: bool,
    pub validation_warnings: Vec<String>,
}

impl Default for CommitState {
    fn default() -> Self {
        Self {
            message: String::new(),
            cursor_pos: 0,
            staged_files: Vec::new(),
            stat_output: String::new(),
            editing: true,
            validation_warnings: Vec::new(),
        }
    }
}

impl CommitState {
    pub fn refresh(&mut self) {
        if let Ok(status) = git::status::get_status() {
            self.staged_files = status.staged;
        }
        if let Ok(stat) = git::diff::get_staged_stat() {
            self.stat_output = stat;
        }
        self.validate();
    }

    fn validate(&mut self) {
        self.validation_warnings.clear();

        if self.message.is_empty() {
            return;
        }

        let lines: Vec<&str> = self.message.lines().collect();

        // Subject line checks
        if let Some(subject) = lines.first() {
            if subject.len() > 72 {
                self.validation_warnings.push(format!(
                    "Subject line is {} chars (recommended: ≤72)",
                    subject.len()
                ));
            }
            if subject.ends_with('.') {
                self.validation_warnings.push("Subject should not end with a period".to_string());
            }
        }

        // Body line checks
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.len() > 80 {
                self.validation_warnings.push(format!(
                    "Line {} is {} chars (recommended: ≤80)",
                    i + 1,
                    line.len()
                ));
            }
        }

        // Second line should be blank
        if lines.len() > 1 && !lines[1].is_empty() {
            self.validation_warnings
                .push("Line 2 should be blank (separates subject from body)".to_string());
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &CommitState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(6),  // Staged files summary
            Constraint::Min(8),    // Message editor
            Constraint::Length(4), // Validation + hints
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  ✏ ", Style::default().fg(Color::Green)),
        Span::styled("Commit", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  ({} files staged)", state.staged_files.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(title, chunks[0]);

    // Staged files
    let stat_paragraph = Paragraph::new(state.stat_output.as_str())
        .block(
            Block::default()
                .title(Span::styled(" Changes to commit ", Style::default().fg(Color::Green)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: false });
    f.render_widget(stat_paragraph, chunks[1]);

    // Message editor
    let editor_border_color = if state.editing { Color::Cyan } else { Color::DarkGray };
    let lines: Vec<Line> = state
        .message
        .lines()
        .enumerate()
        .map(|(i, l)| {
            let color = if i == 0 && l.len() > 72 {
                Color::Yellow
            } else if l.len() > 80 {
                Color::Yellow
            } else {
                Color::White
            };
            Line::from(Span::styled(l, Style::default().fg(color)))
        })
        .collect();

    let lines = if lines.is_empty() {
        vec![Line::from(Span::styled(
            "Type your commit message...",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        lines
    };

    let editor = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Commit Message ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(editor_border_color)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(editor, chunks[2]);

    // Show cursor position if editing
    if state.editing {
        let msg_lines: Vec<&str> = state.message.lines().collect();
        let cursor_y = msg_lines.len().max(1) - 1;
        let cursor_x = msg_lines.last().map(|l| l.len()).unwrap_or(0);
        f.set_cursor_position((
            chunks[2].x + 1 + cursor_x as u16,
            chunks[2].y + 1 + cursor_y as u16,
        ));
    }

    // Validation & hints
    let mut hint_lines = Vec::new();

    for w in &state.validation_warnings {
        hint_lines.push(Line::from(Span::styled(
            format!("  ⚠ {}", w),
            Style::default().fg(Color::Yellow),
        )));
    }

    hint_lines.push(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" Commit  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" New line  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" Cancel  "),
        Span::styled("Ctrl+A", Style::default().fg(Color::Cyan)),
        Span::raw(" Amend"),
    ]));

    let hints = Paragraph::new(hint_lines)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(hints, chunks[3]);
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.commit_state;

    if !state.editing {
        match key.code {
            KeyCode::Char('i') | KeyCode::Enter => {
                state.editing = true;
            }
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            if state.message.is_empty() {
                // Go back to dashboard
                app.view = crate::app::View::Dashboard;
                app.dashboard_state.refresh();
            } else {
                state.editing = false;
            }
        }
        KeyCode::Enter => {
            // Enter commits if message is non-empty
            if !state.message.trim().is_empty() {
                do_commit(app)?;
            }
        }
        KeyCode::Tab => {
            // Tab adds a newline for multi-line commit messages
            state.message.push('\n');
        }
        KeyCode::Char('a') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            // Amend
            if let Ok(prev_msg) = git::run_git(&["log", "-1", "--format=%B"]) {
                state.message = prev_msg.trim().to_string();
                state.validate();
                app.set_status("Loaded previous commit message (amend mode)");
            }
        }
        KeyCode::Char(c) => {
            state.message.push(c);
            state.validate();
        }
        KeyCode::Backspace => {
            state.message.pop();
            state.validate();
        }
        _ => {}
    }

    Ok(())
}

fn do_commit(app: &mut crate::app::App) -> anyhow::Result<()> {
    if app.commit_state.message.trim().is_empty() {
        app.set_status("Commit message cannot be empty");
        return Ok(());
    }

    if app.commit_state.staged_files.is_empty() {
        app.set_status("No files staged for commit");
        return Ok(());
    }

    let msg = app.commit_state.message.trim().to_string();
    match git::run_git(&["commit", "-m", &msg]) {
        Ok(output) => {
            app.set_status(format!("✓ {}", output.lines().next().unwrap_or("Committed")));
            app.commit_state.message.clear();
            app.commit_state.editing = true;
            app.view = crate::app::View::Dashboard;
            app.dashboard_state.refresh();
        }
        Err(e) => {
            app.set_status(format!("Commit failed: {}", e));
        }
    }

    Ok(())
}
