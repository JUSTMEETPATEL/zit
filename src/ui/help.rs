use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::View;

pub fn render(f: &mut Frame, area: Rect, current_view: View) {
    // Center the popup
    let popup_area = centered_rect(60, 70, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    let keybindings = match current_view {
        View::Dashboard => vec![
            ("s", "Open Staging view"),
            ("c", "Open Commit view"),
            ("b", "Open Branches view"),
            ("l", "Open Timeline (Log) view"),
            ("t", "Open Time Travel view"),
            ("r", "Open Reflog view"),
            ("g", "Open GitHub view"),
            ("?", "Toggle this help"),
            ("q", "Quit"),
            ("Ctrl+C", "Force quit"),
        ],
        View::Staging => vec![
            ("↑/↓ or j/k", "Navigate files"),
            ("Space", "Toggle stage/unstage"),
            ("Ctrl+A", "Stage all files"),
            ("u", "Unstage all files"),
            ("/", "Search files"),
            ("c", "Open Commit view"),
            ("PgDn/PgUp", "Scroll diff"),
            ("q", "Back to Dashboard"),
        ],
        View::Commit => vec![
            ("Type", "Enter commit message"),
            ("Enter", "New line"),
            ("Ctrl+S", "Submit commit"),
            ("Ctrl+A", "Amend previous commit"),
            ("Esc", "Stop editing / Back"),
        ],
        View::Branches => vec![
            ("↑/↓ or j/k", "Navigate branches"),
            ("Enter", "Switch to branch"),
            ("n", "Create new branch"),
            ("d", "Delete branch"),
            ("R", "Rename current branch"),
            ("Tab", "Toggle local/remote"),
            ("q", "Back to Dashboard"),
        ],
        View::Timeline => vec![
            ("↑/↓ or j/k", "Navigate commits"),
            ("Enter", "View commit details & diff"),
            ("/", "Search commits by message"),
            ("y", "Copy commit hash"),
            ("PgDn/PgUp", "Next/prev page"),
            ("q", "Back to Dashboard"),
        ],
        View::TimeTravel => vec![
            ("↑/↓ or j/k", "Navigate commits"),
            ("s", "Soft reset (safe)"),
            ("m", "Mixed reset"),
            ("h", "Hard reset (⚠ destructive)"),
            ("b", "Create branch from commit"),
            ("q", "Back to Dashboard"),
        ],
        View::Reflog => vec![
            ("↑/↓ or j/k", "Navigate entries"),
            ("Enter", "View diff"),
            ("b", "Create branch from entry"),
            ("f", "Cycle operation filter"),
            ("c", "Clear filter"),
            ("q", "Back to Dashboard"),
        ],
        View::GitHub => vec![
            ("↑/↓ or j/k", "Navigate menu"),
            ("Enter", "Select option"),
            ("a", "Add/update PAT"),
            ("q", "Back to Dashboard"),
        ],
    };

    let view_name = match current_view {
        View::Dashboard => "Dashboard",
        View::Staging => "Staging",
        View::Commit => "Commit",
        View::Branches => "Branches",
        View::Timeline => "Timeline",
        View::TimeTravel => "Time Travel",
        View::Reflog => "Reflog",
        View::GitHub => "GitHub",
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  {} — Keybindings", view_name),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
    ];

    for (key, desc) in keybindings {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>14}  ", key), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(desc, Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "  Press ? or Esc to close",
        Style::default().fg(Color::DarkGray),
    )));

    let help = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" ❓ Help ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
