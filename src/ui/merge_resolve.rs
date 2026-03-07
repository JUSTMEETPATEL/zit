//! Merge conflict resolution view — side-by-side diff with AI suggestions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{FollowUpAction, FollowUpItem, Popup, View};
use crate::git;

// ─── State ─────────────────────────────────────────────────────

/// State for the merge conflict resolution view.
#[derive(Default)]
pub struct MergeResolveState {
    /// All conflicted files in the repo.
    pub conflicted_files: Vec<git::FileEntry>,
    /// Currently selected file index.
    pub selected_file: usize,
    /// Parsed conflict regions for the current file.
    pub conflict_regions: Vec<git::merge::ConflictRegion>,
    /// Currently selected conflict region.
    pub selected_region: usize,
    /// Raw file content with conflict markers.
    pub raw_conflict_content: Option<String>,
    /// Total lines in the file.
    pub total_lines: usize,
    /// AI suggestion text (full response).
    pub ai_suggestion: Option<String>,
    /// Parsed recommendation from AI (ACCEPT_CURRENT, ACCEPT_INCOMING, MERGE_BOTH).
    pub ai_recommendation: Option<String>,
    /// Parsed resolved content from AI response.
    pub ai_resolved_content: Option<String>,
    /// Current merge state (merge/rebase/cherry-pick).
    pub merge_state: Option<git::MergeState>,
    /// Scroll positions for the three panels.
    pub scroll_left: u16,
    pub scroll_right: u16,
    pub scroll_center: u16,
    /// Which panel is focused (0=left/current, 1=center/AI, 2=right/incoming).
    pub focused_panel: usize,
    /// Follow-up suggestions after AI response.
    pub follow_ups: Vec<FollowUpItem>,
    /// Selected follow-up index.
    pub follow_up_selected: usize,
}

impl MergeResolveState {
    pub fn refresh(&mut self) {
        // Save AI state so it survives the reload
        let saved_ai_suggestion = self.ai_suggestion.take();
        let saved_ai_recommendation = self.ai_recommendation.take();
        let saved_ai_resolved_content = self.ai_resolved_content.take();
        let saved_follow_ups = std::mem::take(&mut self.follow_ups);
        let saved_scroll_center = self.scroll_center;

        log::debug!(
            "[MergeResolve] refresh: saved ai_suggestion={}",
            saved_ai_suggestion.is_some()
        );

        // Refresh conflict list
        let status = git::status::get_status().unwrap_or_default();
        self.conflicted_files = status.conflicts;
        self.merge_state = git::merge::get_merge_state();

        log::debug!(
            "[MergeResolve] refresh: {} conflicted files",
            self.conflicted_files.len()
        );

        // Clamp selected file
        if self.selected_file >= self.conflicted_files.len() && !self.conflicted_files.is_empty() {
            self.selected_file = self.conflicted_files.len() - 1;
        }

        // Load the selected file
        self.load_selected_file();

        // Restore AI state
        if saved_ai_suggestion.is_some() {
            log::debug!("[MergeResolve] refresh: RESTORING ai_suggestion");
            self.ai_suggestion = saved_ai_suggestion;
            self.ai_recommendation = saved_ai_recommendation;
            self.ai_resolved_content = saved_ai_resolved_content;
            self.follow_ups = saved_follow_ups;
            self.scroll_center = saved_scroll_center;
        }
    }

    pub fn load_selected_file(&mut self) {
        self.conflict_regions.clear();
        self.raw_conflict_content = None;
        self.total_lines = 0;
        self.selected_region = 0;
        self.scroll_left = 0;
        self.scroll_right = 0;
        self.scroll_center = 0;
        self.ai_suggestion = None;
        self.ai_recommendation = None;
        self.ai_resolved_content = None;
        self.follow_ups.clear();

        if let Some(file) = self.conflicted_files.get(self.selected_file) {
            if let Ok(conflict_file) = git::merge::get_conflict_file(&file.path) {
                self.raw_conflict_content = Some(conflict_file.raw_content);
                self.conflict_regions = conflict_file.regions;
                self.total_lines = conflict_file.total_lines;
            }
        }
    }
}

// ─── Render ────────────────────────────────────────────────────

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &MergeResolveState,
    ai_loading: bool,
    ai_available: bool,
) {
    // No conflicts state
    if state.conflicted_files.is_empty() {
        render_no_conflicts(f, area, &state.merge_state);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Length(3), // File selector
            Constraint::Min(8),    // Main three-panel area
            Constraint::Length(5), // Follow-up suggestions
            Constraint::Length(1), // Key hints
        ])
        .split(area);

    // ── Title bar ──
    render_title_bar(f, chunks[0], state);

    // ── File selector ──
    render_file_selector(f, chunks[1], state);

    // ── Main three-panel layout ──
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Current (HEAD)
            Constraint::Percentage(34), // AI Suggestion
            Constraint::Percentage(33), // Incoming
        ])
        .split(chunks[2]);

    render_current_panel(f, panels[0], state);
    render_ai_panel(f, panels[1], state, ai_loading, ai_available);
    render_incoming_panel(f, panels[2], state);

    // ── Follow-up suggestions ──
    render_follow_ups(f, chunks[3], state);

    // ── Key hints ──
    render_key_hints(f, chunks[4], state, ai_loading, ai_available);
}

fn render_no_conflicts(f: &mut Frame, area: Rect, merge_state: &Option<git::MergeState>) {
    let msg = if merge_state.is_some() {
        "All conflicts have been resolved! Press Ctrl+F to continue merge."
    } else {
        "No merge conflicts detected. Press 'q' to go back."
    };

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ✓ No Merge Conflicts",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(Span::styled(
                " Merge Resolve ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(content, area);
}

fn render_title_bar(f: &mut Frame, area: Rect, state: &MergeResolveState) {
    let merge_type_str = state
        .merge_state
        .as_ref()
        .map(|s| format!(" [{}]", s.merge_type))
        .unwrap_or_default();

    let conflict_count = state.conflicted_files.len();

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "⚔ Merge Conflict Resolution",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            merge_type_str,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("File {}/{} ", state.selected_file + 1, conflict_count),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("│ {} conflict region(s)", state.conflict_regions.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(title, area);
}

fn render_file_selector(f: &mut Frame, area: Rect, state: &MergeResolveState) {
    let items: Vec<Span> = state
        .conflicted_files
        .iter()
        .enumerate()
        .flat_map(|(i, file)| {
            let style = if i == state.selected_file {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red)
            };
            vec![
                Span::styled(format!(" {} ", file.path), style),
                Span::raw(" "),
            ]
        })
        .collect();

    let selector = Paragraph::new(Line::from(items)).block(
        Block::default()
            .title(Span::styled(
                " Conflicted Files ",
                Style::default().fg(Color::Red),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(selector, area);
}

fn render_current_panel(f: &mut Frame, area: Rect, state: &MergeResolveState) {
    let border_color = if state.focused_panel == 0 {
        Color::Green
    } else {
        Color::DarkGray
    };

    let lines: Vec<Line> = if state.conflict_regions.is_empty() {
        vec![Line::from(Span::styled(
            "  No conflict markers found",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state
            .conflict_regions
            .iter()
            .enumerate()
            .flat_map(|(i, region)| {
                let is_selected = i == state.selected_region;
                let header_style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                let mut lines = vec![Line::from(Span::styled(
                    format!(
                        "{}── Region {} (lines {}-{}) ──",
                        if is_selected { "▶ " } else { "  " },
                        i + 1,
                        region.start_line,
                        region.end_line,
                    ),
                    header_style,
                ))];

                for line in &region.current {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Green),
                    )));
                }
                lines.push(Line::from(""));
                lines
            })
            .collect()
    };

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Current (HEAD) ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .scroll((state.scroll_left, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(panel, area);
}

fn render_incoming_panel(f: &mut Frame, area: Rect, state: &MergeResolveState) {
    let border_color = if state.focused_panel == 2 {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let lines: Vec<Line> = if state.conflict_regions.is_empty() {
        vec![Line::from(Span::styled(
            "  No conflict markers found",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state
            .conflict_regions
            .iter()
            .enumerate()
            .flat_map(|(i, region)| {
                let is_selected = i == state.selected_region;
                let header_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                let mut lines = vec![Line::from(Span::styled(
                    format!(
                        "{}── Region {} (lines {}-{}) ──",
                        if is_selected { "▶ " } else { "  " },
                        i + 1,
                        region.start_line,
                        region.end_line,
                    ),
                    header_style,
                ))];

                for line in &region.incoming {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Cyan),
                    )));
                }
                lines.push(Line::from(""));
                lines
            })
            .collect()
    };

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Incoming ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .scroll((state.scroll_right, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(panel, area);
}

fn render_ai_panel(
    f: &mut Frame,
    area: Rect,
    state: &MergeResolveState,
    ai_loading: bool,
    ai_available: bool,
) {
    let border_color = if state.focused_panel == 1 {
        Color::Magenta
    } else {
        Color::DarkGray
    };

    let lines: Vec<Line> = if ai_loading {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  ⏳ AI analyzing conflict...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Checking both sides and suggesting",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  the best resolution...",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else if let Some(ref suggestion) = state.ai_suggestion {
        // Parse and display the AI suggestion
        let mut display_lines = Vec::new();

        // Show recommendation badge
        if let Some(ref rec) = state.ai_recommendation {
            let (badge_text, badge_color) = match rec.as_str() {
                s if s.contains("CURRENT") => ("✓ ACCEPT CURRENT", Color::Green),
                s if s.contains("INCOMING") => ("→ ACCEPT INCOMING", Color::Cyan),
                s if s.contains("BOTH") || s.contains("MERGE") => ("⚡ MERGE BOTH", Color::Yellow),
                _ => ("💡 SUGGESTION", Color::Magenta),
            };
            display_lines.push(Line::from(Span::styled(
                format!("  {}", badge_text),
                Style::default()
                    .fg(badge_color)
                    .add_modifier(Modifier::BOLD),
            )));
            display_lines.push(Line::from(""));
        }

        // Show the explanation
        for line in suggestion.lines().take(20) {
            let style = if line.starts_with("RECOMMENDATION:")
                || line.starts_with("RECOMMENDED:")
                || line.starts_with("EXPLANATION:")
                || line.starts_with("FOLLOW-UP:")
                || line.starts_with("COMMANDS:")
                || line.starts_with("RESOLVED CONTENT:")
                || line.starts_with("CURRENT CHANGES")
                || line.starts_with("INCOMING CHANGES")
                || line.starts_with("WARNINGS:")
            {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if line.starts_with("```") {
                Style::default().fg(Color::DarkGray)
            } else if line.starts_with("- ") {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            display_lines.push(Line::from(Span::styled(format!("  {}", line), style)));
        }

        display_lines
    } else if !ai_available {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  AI not configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Press 'a' to set up AI Mentor",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  🤖 AI Ready",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press Ctrl+G to get AI suggestion",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  for resolving this conflict",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " 🤖 AI Suggestion ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .scroll((state.scroll_center, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(panel, area);
}

fn render_follow_ups(f: &mut Frame, area: Rect, state: &MergeResolveState) {
    if state.follow_ups.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  Resolve conflicts or press Ctrl+G for AI help",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .title(Span::styled(
                    " Follow-up Actions ",
                    Style::default().fg(Color::DarkGray),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = state
        .follow_ups
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.follow_up_selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}{}", prefix, i + 1),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(". "),
                Span::styled(&item.label, style),
                Span::styled(
                    format!(" — {}", item.description),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(
                " Follow-up Actions (Enter to select, 1-5 for quick pick) ",
                Style::default().fg(Color::Yellow),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(list, area);
}

fn render_key_hints(
    f: &mut Frame,
    area: Rect,
    state: &MergeResolveState,
    ai_loading: bool,
    _ai_available: bool,
) {
    let mut hints = vec![
        Span::styled(" [a]", Style::default().fg(Color::Green)),
        Span::raw(" Accept Current "),
        Span::styled("[i]", Style::default().fg(Color::Cyan)),
        Span::raw(" Accept Incoming "),
    ];

    if state.ai_resolved_content.is_some() {
        hints.push(Span::styled("[m]", Style::default().fg(Color::Magenta)));
        hints.push(Span::raw(" Apply AI "));
    }

    if ai_loading {
        hints.push(Span::styled(
            "⏳ AI working... ",
            Style::default().fg(Color::Yellow),
        ));
    } else {
        hints.push(Span::styled("[G]", Style::default().fg(Color::Magenta)));
        hints.push(Span::raw(" AI Suggest "));
    }

    hints.extend([
        Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
        Span::raw(" Panel "),
        Span::styled("[n/p]", Style::default().fg(Color::Cyan)),
        Span::raw(" File "),
        Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
        Span::raw(" Region "),
        Span::styled("[!]", Style::default().fg(Color::Red)),
        Span::raw(" Abort "),
        Span::styled("[F]", Style::default().fg(Color::Green)),
        Span::raw(" Continue "),
        Span::styled("[q]", Style::default().fg(Color::Red)),
        Span::raw(" Back"),
    ]);

    let hint_line = Paragraph::new(Line::from(hints));
    f.render_widget(hint_line, area);
}

// ─── Key Handling ──────────────────────────────────────────────

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.merge_resolve_state;

    match key.code {
        // Accept current changes for the selected region
        KeyCode::Char('a')
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !state.conflict_regions.is_empty() =>
        {
            resolve_current_region(app, "current")?;
        }

        // Accept incoming changes for the selected region
        KeyCode::Char('i') if !state.conflict_regions.is_empty() => {
            resolve_current_region(app, "incoming")?;
        }

        // Apply AI-suggested resolution
        KeyCode::Char('m') if app.merge_resolve_state.ai_resolved_content.is_some() => {
            let path = app
                .merge_resolve_state
                .conflicted_files
                .get(app.merge_resolve_state.selected_file)
                .map(|f| f.path.clone());
            if let Some(path) = path {
                if let Some(ref content) = app.merge_resolve_state.ai_resolved_content.clone() {
                    match git::merge::resolve_file(&path, content) {
                        Ok(()) => {
                            app.set_status(format!("✓ Applied AI resolution to {}", path));
                            app.merge_resolve_state.refresh();
                            // Show follow-up
                            if app.merge_resolve_state.conflicted_files.is_empty() {
                                app.popup = Popup::FollowUp {
                                    title: "🎉 All Conflicts Resolved!".to_string(),
                                    context: "All merge conflicts have been resolved.".to_string(),
                                    suggestions: vec![
                                        FollowUpItem {
                                            label: "Continue merge".to_string(),
                                            description: "Finalize the merge operation".to_string(),
                                            action: FollowUpAction::ContinueMerge,
                                        },
                                        FollowUpItem {
                                            label: "Review changes".to_string(),
                                            description: "Go to staging view to review".to_string(),
                                            action: FollowUpAction::SwitchToView(View::Staging),
                                        },
                                        FollowUpItem {
                                            label: "Commit now".to_string(),
                                            description: "Go to commit view".to_string(),
                                            action: FollowUpAction::CommitNow,
                                        },
                                    ],
                                    selected: 0,
                                };
                            }
                        }
                        Err(e) => {
                            app.set_status(format!("Error: {}", e));
                        }
                    }
                }
            }
        }

        // AI suggest for current file (Ctrl+G or G)
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let state = &app.merge_resolve_state;
            if let Some(ref content) = state.raw_conflict_content.clone() {
                let path = state
                    .conflicted_files
                    .get(state.selected_file)
                    .map(|f| f.path.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                app.start_ai_merge_resolve(path, content.to_string());
            } else {
                app.set_status("No conflict content to analyze");
            }
        }
        KeyCode::Char('G') => {
            // Mac-friendly alternative for Ctrl+G
            let state = &app.merge_resolve_state;
            if let Some(ref content) = state.raw_conflict_content.clone() {
                let path = state
                    .conflicted_files
                    .get(state.selected_file)
                    .map(|f| f.path.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                app.start_ai_merge_resolve(path, content.to_string());
            } else {
                app.set_status("No conflict content to analyze");
            }
        }

        // Abort merge (Ctrl+A or !)
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.popup = Popup::Confirm {
                title: "⚠ Abort Merge".to_string(),
                message: "This will discard ALL merge progress. Are you sure? (y/n)".to_string(),
                on_confirm: crate::app::ConfirmAction::AbortMerge,
            };
        }
        KeyCode::Char('!') => {
            // Mac-friendly alternative for Ctrl+A (abort)
            app.popup = Popup::Confirm {
                title: "⚠ Abort Merge".to_string(),
                message: "This will discard ALL merge progress. Are you sure? (y/n)".to_string(),
                on_confirm: crate::app::ConfirmAction::AbortMerge,
            };
        }

        // Continue merge (Ctrl+F or F)
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let state = &app.merge_resolve_state;
            if state.conflicted_files.is_empty() || state.conflict_regions.is_empty() {
                app.popup = Popup::Confirm {
                    title: "Continue Merge".to_string(),
                    message: "Finalize the merge? (y/n)".to_string(),
                    on_confirm: crate::app::ConfirmAction::ContinueMerge,
                };
            } else {
                app.set_status(format!(
                    "Cannot continue — {} conflicts remaining",
                    state.conflicted_files.len()
                ));
            }
        }
        KeyCode::Char('F') => {
            // Mac-friendly alternative for Ctrl+F (continue/finalize)
            let state = &app.merge_resolve_state;
            if state.conflicted_files.is_empty() || state.conflict_regions.is_empty() {
                app.popup = Popup::Confirm {
                    title: "Continue Merge".to_string(),
                    message: "Finalize the merge? (y/n)".to_string(),
                    on_confirm: crate::app::ConfirmAction::ContinueMerge,
                };
            } else {
                app.set_status(format!(
                    "Cannot continue — {} conflicts remaining",
                    state.conflicted_files.len()
                ));
            }
        }

        // Merge strategy (Ctrl+M or S)
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.start_ai_merge_strategy(None);
        }
        KeyCode::Char('S') => {
            // Mac-friendly alternative for Ctrl+M (strategy)
            app.start_ai_merge_strategy(None);
        }

        // Navigate conflict regions
        KeyCode::Char('j') | KeyCode::Down => {
            let state = &mut app.merge_resolve_state;
            if state.focused_panel == 0 || state.focused_panel == 2 {
                // Scroll the focused panel
                if state.focused_panel == 0 {
                    state.scroll_left = state.scroll_left.saturating_add(1);
                } else {
                    state.scroll_right = state.scroll_right.saturating_add(1);
                }
            } else if state.focused_panel == 1 {
                state.scroll_center = state.scroll_center.saturating_add(1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let state = &mut app.merge_resolve_state;
            if state.focused_panel == 0 {
                state.scroll_left = state.scroll_left.saturating_sub(1);
            } else if state.focused_panel == 1 {
                state.scroll_center = state.scroll_center.saturating_sub(1);
            } else {
                state.scroll_right = state.scroll_right.saturating_sub(1);
            }
        }

        // Navigate between conflict regions ([ and ])
        KeyCode::Char('[') => {
            let state = &mut app.merge_resolve_state;
            if state.selected_region > 0 {
                state.selected_region -= 1;
            }
        }
        KeyCode::Char(']') => {
            let state = &mut app.merge_resolve_state;
            if state.selected_region + 1 < state.conflict_regions.len() {
                state.selected_region += 1;
            }
        }

        // Navigate files
        KeyCode::Char('n') => {
            let state = &mut app.merge_resolve_state;
            if state.selected_file + 1 < state.conflicted_files.len() {
                state.selected_file += 1;
                state.load_selected_file();
            }
        }
        KeyCode::Char('p') => {
            let state = &mut app.merge_resolve_state;
            if state.selected_file > 0 {
                state.selected_file -= 1;
                state.load_selected_file();
            }
        }

        // Tab to cycle panels
        KeyCode::Tab => {
            let state = &mut app.merge_resolve_state;
            state.focused_panel = (state.focused_panel + 1) % 3;
        }

        // Follow-up selection with number keys
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let idx = (c as usize) - ('1' as usize);
            let state = &app.merge_resolve_state;
            if idx < state.follow_ups.len() {
                let action = state.follow_ups[idx].action.clone();
                app.execute_follow_up(action);
            }
        }

        // Enter to execute selected follow-up
        KeyCode::Enter => {
            let state = &app.merge_resolve_state;
            if !state.follow_ups.is_empty() {
                let idx = state.follow_up_selected;
                if idx < state.follow_ups.len() {
                    let action = state.follow_ups[idx].action.clone();
                    app.execute_follow_up(action);
                }
            }
        }

        _ => {}
    }

    Ok(())
}

/// Resolve the currently selected conflict region with the given choice.
fn resolve_current_region(app: &mut crate::app::App, choice: &str) -> anyhow::Result<()> {
    let state = &app.merge_resolve_state;
    let file_path = state
        .conflicted_files
        .get(state.selected_file)
        .map(|f| f.path.clone());

    if let Some(path) = file_path {
        if let Some(region) = state.conflict_regions.get(state.selected_region) {
            let region = region.clone();
            match git::merge::resolve_region(&path, &region, choice) {
                Ok(new_content) => {
                    // Write the resolved content
                    match std::fs::write(&path, &new_content) {
                        Ok(()) => {
                            let label = if choice == "current" {
                                "current"
                            } else {
                                "incoming"
                            };
                            app.set_status(format!(
                                "✓ Accepted {} changes in region {} of {}",
                                label,
                                app.merge_resolve_state.selected_region + 1,
                                path
                            ));

                            // Reload the file to check for remaining conflicts
                            app.merge_resolve_state.load_selected_file();

                            // If no more conflict regions in this file, stage it
                            if app.merge_resolve_state.conflict_regions.is_empty() {
                                let _ = git::run_git(&["add", &path]);
                                app.set_status(format!(
                                    "✓ All conflicts resolved in {} — file staged",
                                    path
                                ));
                                app.merge_resolve_state.refresh();

                                // If all conflicts resolved, show follow-up
                                if app.merge_resolve_state.conflicted_files.is_empty() {
                                    app.popup = Popup::FollowUp {
                                        title: "🎉 All Conflicts Resolved!".to_string(),
                                        context: "All merge conflicts have been resolved."
                                            .to_string(),
                                        suggestions: vec![
                                            FollowUpItem {
                                                label: "Continue merge".to_string(),
                                                description: "Finalize the merge operation"
                                                    .to_string(),
                                                action: FollowUpAction::ContinueMerge,
                                            },
                                            FollowUpItem {
                                                label: "Review changes".to_string(),
                                                description: "Go to staging view to review"
                                                    .to_string(),
                                                action: FollowUpAction::SwitchToView(View::Staging),
                                            },
                                            FollowUpItem {
                                                label: "Commit now".to_string(),
                                                description: "Go to commit view".to_string(),
                                                action: FollowUpAction::CommitNow,
                                            },
                                        ],
                                        selected: 0,
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            app.set_status(format!("Error writing file: {}", e));
                        }
                    }
                }
                Err(e) => {
                    app.set_status(format!("Error resolving region: {}", e));
                }
            }
        }
    }

    Ok(())
}
