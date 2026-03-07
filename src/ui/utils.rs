use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;

/// Create a centered rectangle within a given area, using percentage-based sizing.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Return the appropriate color for a unified-diff line.
///
/// Handles `+++`/`---` header lines (Yellow), `@@` hunk headers (Cyan),
/// `+` additions (Green), `-` deletions (Red), and context (DarkGray).
pub fn diff_line_color(line: &str) -> Color {
    if line.starts_with("+++") || line.starts_with("---") {
        Color::Yellow
    } else if line.starts_with('+') {
        Color::Green
    } else if line.starts_with('-') {
        Color::Red
    } else if line.starts_with("@@") {
        Color::Cyan
    } else {
        Color::DarkGray
    }
}

/// Navigate a list selection by delta, clamping to bounds.
/// Returns the new selected index.
pub fn navigate_list(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as isize + delta;
    new.clamp(0, (len as isize) - 1) as usize
}
