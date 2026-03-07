use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

// ─── Data Model ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorkflowNode {
    pub id: usize,
    pub name: String,
    pub uses: Option<String>, // e.g. "actions/checkout@v4"
    pub run: Option<String>,  // e.g. "npm install"
    pub needs: Vec<usize>,    // IDs of dependency nodes
}

impl WorkflowNode {
    fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            uses: None,
            run: None,
            needs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn display_label(&self) -> String {
        if let Some(ref uses) = self.uses {
            format!("{}\n{}", self.name, uses)
        } else if let Some(ref run) = self.run {
            format!("{}\n{}", self.name, run)
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuilderMode {
    Navigate,
    EditName,
    EditUses,
    EditRun,
    Connect,
    EditWorkflowName,
    SelectTrigger,
}

pub struct WorkflowBuilderState {
    pub nodes: Vec<WorkflowNode>,
    pub selected: usize,
    pub mode: BuilderMode,
    pub workflow_name: String,
    pub trigger_events: Vec<String>,
    pub input_buffer: String,
    pub connect_from: Option<usize>,
    pub next_id: usize,
    pub status: Option<String>,
    pub scroll: u16,
    /// Tracks whether we're adding a new node (true) or editing existing (false)
    pub adding_new: bool,
}

const TRIGGER_OPTIONS: &[&str] = &[
    "push",
    "pull_request",
    "workflow_dispatch",
    "schedule",
    "release",
];

impl WorkflowBuilderState {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            selected: 0,
            mode: BuilderMode::Navigate,
            workflow_name: "CI".to_string(),
            trigger_events: vec!["push".to_string()],
            input_buffer: String::new(),
            connect_from: None,
            next_id: 0,
            status: None,
            scroll: 0,
            adding_new: false,
        }
    }

    fn add_node(&mut self, name: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(WorkflowNode::new(id, name));
        self.selected = self.nodes.len() - 1;
        id
    }

    fn remove_selected(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        let removed_id = self.nodes[self.selected].id;
        self.nodes.remove(self.selected);
        // Remove any references to the deleted node
        for node in &mut self.nodes {
            node.needs.retain(|&id| id != removed_id);
        }
        if self.selected > 0 && self.selected >= self.nodes.len() {
            self.selected = self.nodes.len() - 1;
        }
    }

    fn toggle_trigger(&mut self, trigger: &str) {
        if let Some(pos) = self.trigger_events.iter().position(|t| t == trigger) {
            if self.trigger_events.len() > 1 {
                self.trigger_events.remove(pos);
            }
        } else {
            self.trigger_events.push(trigger.to_string());
        }
    }

    /// Generate the YAML content from the current node graph.
    pub fn generate_yaml(&self) -> String {
        let mut yaml = String::new();

        // Name
        yaml.push_str(&format!("name: {}\n\n", self.workflow_name));

        // Triggers
        yaml.push_str("on:\n");
        for trigger in &self.trigger_events {
            if trigger == "schedule" {
                yaml.push_str("  schedule:\n");
                yaml.push_str("    - cron: '0 0 * * *'\n");
            } else {
                yaml.push_str(&format!("  {}:\n", trigger));
                if trigger == "push" || trigger == "pull_request" {
                    yaml.push_str("    branches: [main]\n");
                }
            }
        }

        yaml.push('\n');

        // Jobs
        yaml.push_str("jobs:\n");

        if self.nodes.is_empty() {
            yaml.push_str("  # Add steps using the workflow builder\n");
            return yaml;
        }

        for node in &self.nodes {
            let job_id = sanitize_job_id(&node.name);
            yaml.push_str(&format!("  {}:\n", job_id));
            yaml.push_str("    runs-on: ubuntu-latest\n");

            // Dependencies
            if !node.needs.is_empty() {
                let needs: Vec<String> = node
                    .needs
                    .iter()
                    .filter_map(|&id| self.nodes.iter().find(|n| n.id == id))
                    .map(|n| sanitize_job_id(&n.name))
                    .collect();
                if needs.len() == 1 {
                    yaml.push_str(&format!("    needs: {}\n", needs[0]));
                } else {
                    yaml.push_str(&format!("    needs: [{}]\n", needs.join(", ")));
                }
            }

            yaml.push_str("    steps:\n");
            yaml.push_str("      - uses: actions/checkout@v4\n");

            if let Some(ref uses) = node.uses {
                yaml.push_str(&format!("      - name: {}\n", node.name));
                yaml.push_str(&format!("        uses: {}\n", uses));
            }

            if let Some(ref run) = node.run {
                yaml.push_str(&format!("      - name: {}\n", node.name));
                yaml.push_str(&format!("        run: {}\n", run));
            }

            // If neither uses nor run, add a placeholder
            if node.uses.is_none() && node.run.is_none() {
                yaml.push_str(&format!("      - name: {}\n", node.name));
                yaml.push_str("        run: echo \"TODO\"\n");
            }

            yaml.push('\n');
        }

        yaml
    }
}

fn sanitize_job_id(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "")
}

// ─── Rendering ─────────────────────────────────────────────

pub fn render(f: &mut Frame, area: Rect, state: &WorkflowBuilderState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Workflow info
            Constraint::Min(10),   // Pipeline view
            Constraint::Length(3), // Keys
            Constraint::Length(2), // Status
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🔧 ", Style::default()),
        Span::styled(
            "Workflow Builder",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ({} steps)", state.nodes.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Workflow info bar
    let triggers_str = state.trigger_events.join(", ");
    let info = Paragraph::new(Line::from(vec![
        Span::styled("  Name: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &state.workflow_name,
            Style::default()
                .fg(if state.mode == BuilderMode::EditWorkflowName {
                    Color::Yellow
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("    Triggers: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&triggers_str, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(info, chunks[1]);

    // Pipeline view — render the boxes
    render_pipeline(f, chunks[2], state);

    // Keys based on mode
    let keys = match state.mode {
        BuilderMode::Navigate => Line::from(vec![
            Span::styled(" [a]", Style::default().fg(Color::Green)),
            Span::raw(" Add "),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" Edit "),
            Span::styled("[d]", Style::default().fg(Color::Red)),
            Span::raw(" Delete "),
            Span::styled("[c]", Style::default().fg(Color::Yellow)),
            Span::raw(" Connect "),
            Span::styled("[g]", Style::default().fg(Color::Magenta)),
            Span::raw(" Generate "),
            Span::styled("[n]", Style::default().fg(Color::White)),
            Span::raw(" Name "),
            Span::styled("[t]", Style::default().fg(Color::White)),
            Span::raw(" Triggers"),
        ]),
        BuilderMode::EditName
        | BuilderMode::EditUses
        | BuilderMode::EditRun
        | BuilderMode::EditWorkflowName => {
            let field = match state.mode {
                BuilderMode::EditName => "Step Name",
                BuilderMode::EditUses => "Action (uses)",
                BuilderMode::EditRun => "Command (run)",
                BuilderMode::EditWorkflowName => "Workflow Name",
                _ => "",
            };
            Line::from(vec![
                Span::styled(
                    format!("  Editing: {} ", field),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("│ {} │ ", state.input_buffer),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Confirm "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ])
        }
        BuilderMode::Connect => {
            if state.connect_from.is_some() {
                Line::from(vec![
                    Span::styled("  ↳ Select target node", Style::default().fg(Color::Yellow)),
                    Span::raw(" then press "),
                    Span::styled("[c]", Style::default().fg(Color::Yellow)),
                    Span::raw(" to connect  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  Press ", Style::default().fg(Color::Yellow)),
                    Span::styled("[c]", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        " on source node to start connecting",
                        Style::default().fg(Color::Yellow),
                    ),
                ])
            }
        }
        BuilderMode::SelectTrigger => {
            let triggers: Vec<Span> = TRIGGER_OPTIONS
                .iter()
                .enumerate()
                .flat_map(|(i, &t)| {
                    let active = state.trigger_events.contains(&t.to_string());
                    let style = if active {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let marker = if active { "✓" } else { "○" };
                    vec![
                        Span::styled(format!(" [{}]", i + 1), Style::default().fg(Color::Cyan)),
                        Span::styled(format!(" {} {} ", marker, t), style),
                    ]
                })
                .collect();
            let mut spans = triggers;
            spans.push(Span::styled(" [Esc]", Style::default().fg(Color::Red)));
            spans.push(Span::raw(" Done"));
            Line::from(spans)
        }
    };

    let keys_widget = Paragraph::new(keys).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys_widget, chunks[3]);

    // Status
    if let Some(ref msg) = state.status {
        let status = Paragraph::new(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ));
        f.render_widget(status, chunks[4]);
    }
}

fn render_pipeline(f: &mut Frame, area: Rect, state: &WorkflowBuilderState) {
    if state.nodes.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No steps yet. Press [a] to add your first step.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Build a GitHub Actions pipeline visually:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  ┌──────────┐     ┌──────────┐     ┌──────────┐",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  │  Build   │────▶│  Test    │────▶│  Deploy  │",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  └──────────┘     └──────────┘     └──────────┘",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(
                    " Pipeline ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, area);
        return;
    }

    // Render nodes as boxes with connectors
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Topological layers for layout
    let layers = build_layers(&state.nodes);

    for (layer_idx, layer) in layers.iter().enumerate() {
        // Top border row
        let mut top_spans: Vec<Span> = vec![Span::raw("  ")];
        let mut mid_spans: Vec<Span> = vec![Span::raw("  ")];
        let mut detail_spans: Vec<Span> = vec![Span::raw("  ")];
        let mut bot_spans: Vec<Span> = vec![Span::raw("  ")];

        for (i, &node_idx) in layer.iter().enumerate() {
            let node = &state.nodes[node_idx];
            let is_selected = node_idx == state.selected;
            let is_connect_source = state.connect_from == Some(node.id);

            let border_color = if is_connect_source {
                Color::Yellow
            } else if is_selected {
                Color::Cyan
            } else {
                Color::DarkGray
            };
            let name_color = if is_selected {
                Color::White
            } else {
                Color::Gray
            };

            let box_width = 18;
            let name = if node.name.len() > box_width - 4 {
                format!("{}…", &node.name[..box_width - 5])
            } else {
                node.name.clone()
            };
            let detail = if let Some(ref uses) = node.uses {
                if uses.len() > box_width - 4 {
                    format!("{}…", &uses[..box_width - 5])
                } else {
                    uses.clone()
                }
            } else if let Some(ref run) = node.run {
                if run.len() > box_width - 4 {
                    format!("{}…", &run[..box_width - 5])
                } else {
                    run.clone()
                }
            } else {
                "(empty)".to_string()
            };

            let name_padded = format!("{:^width$}", name, width = box_width - 2);
            let detail_padded = format!("{:^width$}", detail, width = box_width - 2);
            let border_top = "─".repeat(box_width - 2);
            let border_bot = "─".repeat(box_width - 2);

            let selector = if is_selected { "▶" } else { " " };

            top_spans.push(Span::styled(selector, Style::default().fg(Color::Cyan)));
            top_spans.push(Span::styled(
                format!("┌{}┐", border_top),
                Style::default().fg(border_color),
            ));

            mid_spans.push(Span::raw(" "));
            mid_spans.push(Span::styled("│", Style::default().fg(border_color)));
            mid_spans.push(Span::styled(
                name_padded,
                Style::default().fg(name_color).add_modifier(Modifier::BOLD),
            ));
            mid_spans.push(Span::styled("│", Style::default().fg(border_color)));

            detail_spans.push(Span::raw(" "));
            detail_spans.push(Span::styled("│", Style::default().fg(border_color)));
            detail_spans.push(Span::styled(
                detail_padded,
                Style::default().fg(Color::DarkGray),
            ));
            detail_spans.push(Span::styled("│", Style::default().fg(border_color)));

            bot_spans.push(Span::raw(" "));
            bot_spans.push(Span::styled(
                format!("└{}┘", border_bot),
                Style::default().fg(border_color),
            ));

            // Connector to next node in same layer
            if i + 1 < layer.len() {
                top_spans.push(Span::raw("  "));
                mid_spans.push(Span::raw("  "));
                detail_spans.push(Span::raw("  "));
                bot_spans.push(Span::raw("  "));
            }
        }

        lines.push(Line::from(top_spans));
        lines.push(Line::from(mid_spans));
        lines.push(Line::from(detail_spans));
        lines.push(Line::from(bot_spans));

        // Arrow between layers
        if layer_idx + 1 < layers.len() {
            let mut arrow_spans: Vec<Span> = vec![Span::raw("  ")];
            let first_node = &state.nodes[layer[0]];
            // Check if any node in next layer needs this node
            let next_layer = &layers[layer_idx + 1];
            let has_connection = next_layer
                .iter()
                .any(|&ni| state.nodes[ni].needs.contains(&first_node.id))
                || !next_layer.is_empty();

            if has_connection {
                arrow_spans.push(Span::styled(
                    "         │         ",
                    Style::default().fg(Color::DarkGray),
                ));
            }
            lines.push(Line::from(arrow_spans.clone()));

            let mut arrow2_spans: Vec<Span> = vec![Span::raw("  ")];
            if has_connection {
                arrow2_spans.push(Span::styled(
                    "         ▼         ",
                    Style::default().fg(Color::DarkGray),
                ));
            }
            lines.push(Line::from(arrow2_spans));
        }
    }

    lines.push(Line::from(""));

    let pipeline = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Pipeline ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));
    f.render_widget(pipeline, area);
}

/// Build topological layers for rendering. Nodes with no dependencies go
/// in layer 0, then nodes that depend on layer 0 go in layer 1, etc.
fn build_layers(nodes: &[WorkflowNode]) -> Vec<Vec<usize>> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut layers: Vec<Vec<usize>> = Vec::new();
    let mut placed: Vec<bool> = vec![false; nodes.len()];
    let mut remaining = nodes.len();

    while remaining > 0 {
        let mut current_layer: Vec<usize> = Vec::new();

        for (idx, node) in nodes.iter().enumerate() {
            if placed[idx] {
                continue;
            }
            // Check if all dependencies are already placed
            let deps_met = node.needs.iter().all(|&dep_id| {
                nodes
                    .iter()
                    .enumerate()
                    .any(|(di, dn)| dn.id == dep_id && placed[di])
            });
            if deps_met || node.needs.is_empty() {
                current_layer.push(idx);
            }
        }

        // Prevent infinite loop if there are circular deps
        if current_layer.is_empty() {
            // Place remaining nodes
            for (idx, _) in nodes.iter().enumerate() {
                if !placed[idx] {
                    current_layer.push(idx);
                }
            }
        }

        for &idx in &current_layer {
            placed[idx] = true;
            remaining -= 1;
        }

        layers.push(current_layer);
    }

    layers
}

// ─── Key Handling ──────────────────────────────────────────

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let state = &mut app.workflow_builder_state;

    match state.mode {
        BuilderMode::Navigate => handle_navigate_key(state, key),
        BuilderMode::EditName
        | BuilderMode::EditUses
        | BuilderMode::EditRun
        | BuilderMode::EditWorkflowName => handle_edit_key(state, key),
        BuilderMode::Connect => handle_connect_key(state, key),
        BuilderMode::SelectTrigger => handle_trigger_key(state, key),
    }

    Ok(())
}

fn handle_navigate_key(state: &mut WorkflowBuilderState, key: KeyEvent) {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if !state.nodes.is_empty() && state.selected + 1 < state.nodes.len() {
                state.selected += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.scroll > 0 {
                state.scroll -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll += 1;
        }
        KeyCode::Char('a') => {
            // Add new node
            state.adding_new = true;
            state.mode = BuilderMode::EditName;
            state.input_buffer.clear();
        }
        KeyCode::Enter => {
            // Edit selected node — cycle name → uses → run
            if !state.nodes.is_empty() {
                state.adding_new = false;
                state.mode = BuilderMode::EditName;
                state.input_buffer = state.nodes[state.selected].name.clone();
            }
        }
        KeyCode::Char('u') => {
            // Edit uses directly
            if !state.nodes.is_empty() {
                state.mode = BuilderMode::EditUses;
                state.input_buffer = state.nodes[state.selected].uses.clone().unwrap_or_default();
            }
        }
        KeyCode::Char('r') => {
            // Edit run directly
            if !state.nodes.is_empty() {
                state.mode = BuilderMode::EditRun;
                state.input_buffer = state.nodes[state.selected].run.clone().unwrap_or_default();
            }
        }
        KeyCode::Char('d') => {
            // Delete selected node
            if !state.nodes.is_empty() {
                let name = state.nodes[state.selected].name.clone();
                state.remove_selected();
                state.status = Some(format!("Removed '{}'", name));
            }
        }
        KeyCode::Char('c') => {
            // Start connecting
            if !state.nodes.is_empty() {
                state.mode = BuilderMode::Connect;
                state.connect_from = Some(state.nodes[state.selected].id);
                state.status = Some("Select target node and press [c] to connect".to_string());
            }
        }
        KeyCode::Char('g') => {
            // Generate YAML
            let yaml = state.generate_yaml();
            let dir = ".github/workflows";
            let filename = format!("{}/{}.yml", dir, sanitize_job_id(&state.workflow_name));
            if let Err(e) = std::fs::create_dir_all(dir) {
                state.status = Some(format!("Error creating dir: {}", e));
                return;
            }
            match std::fs::write(&filename, &yaml) {
                Ok(()) => {
                    state.status = Some(format!("✓ Generated {}", filename));
                }
                Err(e) => {
                    state.status = Some(format!("Error: {}", e));
                }
            }
        }
        KeyCode::Char('n') => {
            // Edit workflow name
            state.mode = BuilderMode::EditWorkflowName;
            state.input_buffer = state.workflow_name.clone();
        }
        KeyCode::Char('t') => {
            // Trigger selection
            state.mode = BuilderMode::SelectTrigger;
        }
        _ => {}
    }
}

fn handle_edit_key(state: &mut WorkflowBuilderState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.mode = BuilderMode::Navigate;
            state.input_buffer.clear();
        }
        KeyCode::Enter => {
            let value = state.input_buffer.trim().to_string();
            if value.is_empty() {
                state.mode = BuilderMode::Navigate;
                return;
            }

            match state.mode {
                BuilderMode::EditWorkflowName => {
                    state.workflow_name = value;
                    state.mode = BuilderMode::Navigate;
                }
                BuilderMode::EditName => {
                    if state.adding_new {
                        // Creating a new node
                        state.add_node(value.clone());
                        state.adding_new = false;
                        state.status = Some(format!("Added step: {}", value));
                        // Continue to edit uses
                        state.mode = BuilderMode::EditUses;
                        state.input_buffer.clear();
                    } else if !state.nodes.is_empty() && state.selected < state.nodes.len() {
                        // Editing existing node name
                        state.nodes[state.selected].name = value;
                        state.mode = BuilderMode::EditUses;
                        state.input_buffer =
                            state.nodes[state.selected].uses.clone().unwrap_or_default();
                    }
                }
                BuilderMode::EditUses => {
                    if !state.nodes.is_empty() && state.selected < state.nodes.len() {
                        state.nodes[state.selected].uses =
                            if value.is_empty() { None } else { Some(value) };
                        state.mode = BuilderMode::EditRun;
                        state.input_buffer =
                            state.nodes[state.selected].run.clone().unwrap_or_default();
                    }
                }
                BuilderMode::EditRun => {
                    if !state.nodes.is_empty() && state.selected < state.nodes.len() {
                        state.nodes[state.selected].run =
                            if value.is_empty() { None } else { Some(value) };
                        state.mode = BuilderMode::Navigate;
                        state.status = Some("✓ Step updated".to_string());
                    }
                }
                _ => {}
            }
            state.input_buffer.clear();
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
        }
        KeyCode::Tab => {
            // Skip to next field
            match state.mode {
                BuilderMode::EditName => {
                    let value = state.input_buffer.trim().to_string();
                    if !value.is_empty() {
                        if state.nodes.is_empty() || state.selected >= state.nodes.len() {
                            state.add_node(value);
                        } else {
                            state.nodes[state.selected].name = value;
                        }
                    }
                    state.mode = BuilderMode::EditUses;
                    state.input_buffer = state
                        .nodes
                        .get(state.selected)
                        .and_then(|n| n.uses.clone())
                        .unwrap_or_default();
                }
                BuilderMode::EditUses => {
                    let value = state.input_buffer.trim().to_string();
                    if !state.nodes.is_empty() && state.selected < state.nodes.len() {
                        state.nodes[state.selected].uses =
                            if value.is_empty() { None } else { Some(value) };
                    }
                    state.mode = BuilderMode::EditRun;
                    state.input_buffer = state
                        .nodes
                        .get(state.selected)
                        .and_then(|n| n.run.clone())
                        .unwrap_or_default();
                }
                BuilderMode::EditRun => {
                    let value = state.input_buffer.trim().to_string();
                    if !state.nodes.is_empty() && state.selected < state.nodes.len() {
                        state.nodes[state.selected].run =
                            if value.is_empty() { None } else { Some(value) };
                    }
                    state.mode = BuilderMode::Navigate;
                    state.input_buffer.clear();
                    state.status = Some("✓ Step updated".to_string());
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_connect_key(state: &mut WorkflowBuilderState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.mode = BuilderMode::Navigate;
            state.connect_from = None;
            state.status = None;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if !state.nodes.is_empty() && state.selected + 1 < state.nodes.len() {
                state.selected += 1;
            }
        }
        KeyCode::Char('c') | KeyCode::Enter => {
            if let Some(from_id) = state.connect_from {
                if !state.nodes.is_empty() {
                    let target_id = state.nodes[state.selected].id;
                    if target_id != from_id {
                        // Add dependency: target needs from
                        if !state.nodes[state.selected].needs.contains(&from_id) {
                            state.nodes[state.selected].needs.push(from_id);
                            let from_name = state
                                .nodes
                                .iter()
                                .find(|n| n.id == from_id)
                                .map(|n| n.name.as_str())
                                .unwrap_or("?");
                            let to_name = &state.nodes[state.selected].name;
                            state.status =
                                Some(format!("✓ Connected: {} → {}", from_name, to_name));
                        } else {
                            state.status = Some("Already connected".to_string());
                        }
                    }
                }
                state.connect_from = None;
                state.mode = BuilderMode::Navigate;
            }
        }
        _ => {}
    }
}

fn handle_trigger_key(state: &mut WorkflowBuilderState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            state.mode = BuilderMode::Navigate;
        }
        KeyCode::Char('1') => state.toggle_trigger("push"),
        KeyCode::Char('2') => state.toggle_trigger("pull_request"),
        KeyCode::Char('3') => state.toggle_trigger("workflow_dispatch"),
        KeyCode::Char('4') => state.toggle_trigger("schedule"),
        KeyCode::Char('5') => state.toggle_trigger("release"),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_job_id() {
        assert_eq!(sanitize_job_id("Build Project"), "build-project");
        assert_eq!(sanitize_job_id("Test & Lint"), "test--lint");
        assert_eq!(sanitize_job_id("deploy_prod"), "deploy_prod");
    }

    #[test]
    fn test_generate_yaml_empty() {
        let state = WorkflowBuilderState::new();
        let yaml = state.generate_yaml();
        assert!(yaml.contains("name: CI"));
        assert!(yaml.contains("on:"));
        assert!(yaml.contains("push:"));
    }

    #[test]
    fn test_generate_yaml_with_nodes() {
        let mut state = WorkflowBuilderState::new();
        let id1 = state.add_node("Build".to_string());
        state.nodes[0].run = Some("npm run build".to_string());
        let _id2 = state.add_node("Test".to_string());
        state.nodes[1].run = Some("npm test".to_string());
        state.nodes[1].needs.push(id1);

        let yaml = state.generate_yaml();
        assert!(yaml.contains("build:"));
        assert!(yaml.contains("test:"));
        assert!(yaml.contains("needs: build"));
        assert!(yaml.contains("run: npm test"));
    }

    #[test]
    fn test_add_and_remove_node() {
        let mut state = WorkflowBuilderState::new();
        state.add_node("Build".to_string());
        state.add_node("Test".to_string());
        assert_eq!(state.nodes.len(), 2);

        state.selected = 0;
        state.remove_selected();
        assert_eq!(state.nodes.len(), 1);
        assert_eq!(state.nodes[0].name, "Test");
    }

    #[test]
    fn test_connect_nodes() {
        let mut state = WorkflowBuilderState::new();
        let id1 = state.add_node("Build".to_string());
        state.add_node("Test".to_string());
        state.nodes[1].needs.push(id1);

        assert_eq!(state.nodes[1].needs, vec![id1]);
    }

    #[test]
    fn test_build_layers_linear() {
        let mut state = WorkflowBuilderState::new();
        let id1 = state.add_node("Build".to_string());
        let id2 = state.add_node("Test".to_string());
        state.nodes[1].needs.push(id1);
        state.add_node("Deploy".to_string());
        state.nodes[2].needs.push(id2);

        let layers = build_layers(&state.nodes);
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec![0]);
        assert_eq!(layers[1], vec![1]);
        assert_eq!(layers[2], vec![2]);
    }

    #[test]
    fn test_build_layers_parallel() {
        let mut state = WorkflowBuilderState::new();
        let id1 = state.add_node("Setup".to_string());
        state.add_node("Build".to_string());
        state.nodes[1].needs.push(id1);
        state.add_node("Lint".to_string());
        state.nodes[2].needs.push(id1);

        let layers = build_layers(&state.nodes);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec![0]); // Setup
        assert!(layers[1].contains(&1) && layers[1].contains(&2)); // Build + Lint parallel
    }

    #[test]
    fn test_toggle_trigger() {
        let mut state = WorkflowBuilderState::new();
        assert_eq!(state.trigger_events, vec!["push"]);

        state.toggle_trigger("pull_request");
        assert_eq!(state.trigger_events.len(), 2);

        state.toggle_trigger("push");
        assert_eq!(state.trigger_events, vec!["pull_request"]);

        // Can't remove last trigger
        state.toggle_trigger("pull_request");
        assert_eq!(state.trigger_events, vec!["pull_request"]);
    }
}
