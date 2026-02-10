use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::git;

pub struct DashboardState {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub conflict_count: usize,
    pub stash_count: u32,
    pub is_clean: bool,
    pub recent_commits: Vec<git::CommitEntry>,
    pub error: Option<String>,
}

impl Default for DashboardState {
    fn default() -> Self {
        let mut state = Self {
            branch: String::new(),
            upstream: None,
            ahead: 0,
            behind: 0,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            conflict_count: 0,
            stash_count: 0,
            is_clean: true,
            recent_commits: Vec::new(),
            error: None,
        };
        state.refresh();
        state
    }
}

impl DashboardState {
    pub fn refresh(&mut self) {
        match git::status::get_status() {
            Ok(status) => {
                self.branch = status.branch.clone();
                self.upstream = status.upstream.clone();
                self.ahead = status.ahead;
                self.behind = status.behind;
                self.staged_count = status.staged.len();
                self.unstaged_count = status.unstaged.len();
                self.untracked_count = status.untracked.len();
                self.conflict_count = status.conflicts.len();
                self.stash_count = status.stash_count;
                self.is_clean = status.is_clean();
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }

        match git::log::get_recent_commits(5) {
            Ok(commits) => self.recent_commits = commits,
            Err(_) => self.recent_commits = Vec::new(),
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &DashboardState, status_msg: &Option<String>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Branch info
            Constraint::Length(3),  // File counts
            Constraint::Min(5),    // Recent commits
            Constraint::Length(3), // Keybindings
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("⚡ zit", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" — Repository Dashboard"),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    // Branch info
    let status_icon = if state.is_clean { "✓" } else { "✗" };
    let status_color = if state.is_clean { Color::Green } else { Color::Yellow };

    let mut branch_spans = vec![
        Span::styled("  Branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&state.branch, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];

    if state.ahead > 0 || state.behind > 0 {
        branch_spans.push(Span::raw("  "));
        if state.ahead > 0 {
            branch_spans.push(Span::styled(format!("⬆{}", state.ahead), Style::default().fg(Color::Green)));
            branch_spans.push(Span::raw(" "));
        }
        if state.behind > 0 {
            branch_spans.push(Span::styled(format!("⬇{}", state.behind), Style::default().fg(Color::Red)));
        }
    }

    branch_spans.push(Span::raw("  │  "));
    branch_spans.push(Span::styled(format!("{} {}", status_icon, if state.is_clean { "Clean" } else { "Dirty" }), Style::default().fg(status_color)));

    if state.conflict_count > 0 {
        branch_spans.push(Span::raw("  "));
        branch_spans.push(Span::styled(format!("⚠ {} conflicts", state.conflict_count), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
    }

    let branch_info = Paragraph::new(Line::from(branch_spans))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(branch_info, chunks[1]);

    // File counts
    let counts = Paragraph::new(Line::from(vec![
        Span::styled("  Staged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.staged_count), Style::default().fg(Color::Green)),
        Span::raw("  │  "),
        Span::styled("Unstaged: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.unstaged_count), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled("Untracked: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.untracked_count), Style::default().fg(Color::Gray)),
        Span::raw("  │  "),
        Span::styled("Stash: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.stash_count), Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(counts, chunks[2]);

    // Recent commits
    let commit_items: Vec<ListItem> = state
        .recent_commits
        .iter()
        .map(|c| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", c.short_hash), Style::default().fg(Color::Yellow)),
                Span::styled(&c.message, Style::default().fg(Color::White)),
                Span::styled(format!(" ({})", c.date), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let commits = if commit_items.is_empty() {
        List::new(vec![ListItem::new(Span::styled(
            "  No commits yet",
            Style::default().fg(Color::DarkGray),
        ))])
    } else {
        List::new(commit_items)
    };

    let commits = commits.block(
        Block::default()
            .title(Span::styled(" Recent Commits ", Style::default().fg(Color::White)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(commits, chunks[3]);

    // Keybindings
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [s]", Style::default().fg(Color::Cyan)),
        Span::raw(" Stage "),
        Span::styled("[c]", Style::default().fg(Color::Cyan)),
        Span::raw(" Commit "),
        Span::styled("[b]", Style::default().fg(Color::Cyan)),
        Span::raw(" Branches "),
        Span::styled("[l]", Style::default().fg(Color::Cyan)),
        Span::raw(" Log "),
        Span::styled("[t]", Style::default().fg(Color::Cyan)),
        Span::raw(" Time Travel "),
        Span::styled("[r]", Style::default().fg(Color::Cyan)),
        Span::raw(" Reflog "),
        Span::styled("[g]", Style::default().fg(Color::Cyan)),
        Span::raw(" GitHub "),
        Span::styled("[?]", Style::default().fg(Color::Cyan)),
        Span::raw(" Help "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(keys, chunks[4]);

    // Status bar
    if let Some(msg) = status_msg {
        let status = Paragraph::new(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ));
        f.render_widget(status, chunks[5]);
    } else if let Some(err) = &state.error {
        let status = Paragraph::new(Span::styled(
            format!(" Error: {}", err),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(status, chunks[5]);
    }
}

pub fn handle_key(_app: &mut crate::app::App, _key: KeyEvent) -> anyhow::Result<()> {
    // Dashboard-specific keys are handled in app.rs global handler
    Ok(())
}
