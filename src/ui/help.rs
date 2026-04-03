use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::utils::centered_rect;
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
            ("a", "Open AI Mentor"),
            ("x", "Open Stash view"),
            ("m", "Open Merge Resolve view"),
            ("w", "Open Workflow Builder"),
            ("B", "Open Bisect view"),
            ("p", "Open Cherry Pick view"),
            ("A", "Open Agent Mode"),
            ("?", "Toggle this help"),
            ("q", "Quit"),
            ("Ctrl+C", "Force quit"),
        ],
        View::Staging => vec![
            ("↑/↓ or j/k", "Navigate files"),
            ("Space", "Toggle stage/unstage"),
            ("h", "Toggle hunk mode"),
            ("A or Ctrl+A", "Stage all files"),
            ("u", "Unstage all files"),
            ("R or Ctrl+R", "AI diff review"),
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
            ("G or Ctrl+G", "Generate AI commit message"),
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
            ("↑/↓ or j/k", "Navigate menu / list"),
            ("Enter", "Select option / Open PR"),
            ("a", "Login with GitHub"),
            ("f", "Cycle PR filter (Open/Closed/All)"),
            ("r", "Refresh"),
            ("Tab", "Switch detail tab"),
            ("m", "Merge PR (in detail)"),
            ("M", "Cycle merge method"),
            ("c", "Close PR (in detail)"),
            ("o", "Open PR in browser"),
            ("Esc", "Back"),
            ("q", "Back to Dashboard"),
        ],
        View::AiMentor => vec![
            ("↑/↓ or j/k", "Navigate menu"),
            ("Enter", "Select / Submit"),
            ("PgDn/PgUp", "Scroll result"),
            ("Esc", "Back to menu"),
            ("q", "Back to Dashboard"),
        ],
        View::Stash => vec![
            ("↑/↓ or j/k", "Navigate stash entries"),
            ("p", "Pop stash (apply & remove)"),
            ("a", "Apply stash (keep in list)"),
            ("d", "Drop stash entry"),
            ("n", "New stash (push)"),
            ("D", "Clear all stashes"),
            ("PgDn/PgUp", "Scroll diff"),
            ("q", "Back to Dashboard"),
        ],
        View::MergeResolve => vec![
            ("a", "Accept current (HEAD) changes"),
            ("i", "Accept incoming changes"),
            ("m", "Apply AI-suggested resolution"),
            ("G or Ctrl+G", "Get AI merge suggestion"),
            ("S or Ctrl+M", "AI merge strategy advice"),
            ("[/]", "Navigate conflict regions"),
            ("n/p", "Next/prev conflicted file"),
            ("Tab", "Cycle panel focus"),
            ("j/k", "Scroll focused panel"),
            ("1-5", "Quick pick follow-up action"),
            ("! or Ctrl+A", "Abort merge"),
            ("F or Ctrl+F", "Continue/finalize merge"),
            ("q", "Back to Dashboard"),
        ],
        View::WorkflowBuilder => vec![
            ("h/l or ←/→", "Navigate steps"),
            ("a", "Add new step"),
            ("Enter", "Edit selected step"),
            ("u", "Edit action (uses)"),
            ("r", "Edit command (run)"),
            ("d", "Delete step"),
            ("c", "Connect steps"),
            ("g", "Generate YAML"),
            ("n", "Edit workflow name"),
            ("t", "Select triggers"),
            ("Tab", "Skip to next field"),
            ("Esc", "Cancel / Go back"),
            ("q", "Back to Dashboard"),
        ],
        View::Bisect => vec![
            ("↑/↓ or j/k", "Navigate commits"),
            ("Enter", "Select commit (bad → good → start)"),
            ("g", "Mark current commit as good"),
            ("b", "Mark current commit as bad"),
            ("s", "Skip current commit"),
            ("R", "Reset / end bisect session"),
            ("PgDn/PgUp", "Scroll log"),
            ("Esc", "Back one step"),
            ("q", "Back to Dashboard"),
        ],
        View::CherryPick => vec![
            ("↑/↓ or j/k", "Navigate branches / commits"),
            ("Enter", "Select branch / Apply cherry-pick"),
            ("Space", "Toggle mark commit for multi-pick"),
            ("c", "Continue after conflict"),
            ("A", "Abort cherry-pick"),
            ("PgDn/PgUp", "Scroll diff"),
            ("Esc", "Back to branch select"),
            ("q", "Back to Dashboard"),
        ],
        View::Agent => vec![
            ("i", "Start typing"),
            ("Enter", "Send message"),
            ("y", "Allow pending command"),
            ("n", "Deny pending command"),
            ("a", "Auto-approve all (session)"),
            ("↑/↓ or j/k", "Scroll conversation"),
            ("Ctrl+L", "Clear conversation"),
            ("Ctrl+C", "Cancel AI request"),
            ("Esc", "Exit input / Back"),
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
        View::AiMentor => "AI Mentor",
        View::Stash => "Stash",
        View::MergeResolve => "Merge Resolve",
        View::WorkflowBuilder => "Workflow Builder",
        View::Bisect => "Bisect",
        View::CherryPick => "Cherry Pick",
        View::Agent => "Agent",
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  {} — Keybindings", view_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
    ];

    for (key, desc) in keybindings {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>14}  ", key),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
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
                .title(Span::styled(
                    " ❓ Help ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(help, popup_area);
}
