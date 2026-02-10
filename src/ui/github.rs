use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::git;

#[derive(Debug, Clone, PartialEq)]
pub enum GitHubView {
    Menu,
    DeviceAuth(DeviceAuthState),
    CreateRepo,
    Collaborators,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceAuthState {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub interval: u64,
    pub ticks_since_poll: u64,
    pub status: String,
}

pub struct GitHubState {
    pub view: GitHubView,
    pub menu_selected: usize,
    pub menu_state: ListState,
    // Create repo fields
    pub repo_name: String,
    pub repo_desc: String,
    pub repo_private: bool,
    pub create_field: usize,
    pub editing_field: bool,
    // Collaborator fields
    pub collaborators: Vec<git::github_auth::Collaborator>,
    pub collab_selected: usize,
    pub collab_list_state: ListState,
    pub collab_error: Option<String>,
    // Status
    pub status: Option<String>,
}

impl GitHubState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            view: GitHubView::Menu,
            menu_selected: 0,
            menu_state: state,
            repo_name: String::new(),
            repo_desc: String::new(),
            repo_private: true,
            create_field: 0,
            editing_field: false,
            collaborators: Vec::new(),
            collab_selected: 0,
            collab_list_state: ListState::default(),
            collab_error: None,
            status: None,
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &mut GitHubState, config: &crate::config::Config) {
    let has_token = config.github.get_token().is_some();

    match &state.view {
        GitHubView::Menu => render_menu(f, area, state, has_token, config),
        GitHubView::DeviceAuth(auth_state) => render_device_auth(f, area, auth_state),
        GitHubView::CreateRepo => render_create_repo(f, area, state),
        GitHubView::Collaborators => render_collaborators(f, area, state),
    }
}

fn render_menu(f: &mut Frame, area: Rect, state: &mut GitHubState, has_token: bool, config: &crate::config::Config) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Auth status
            Constraint::Min(8),   // Menu
            Constraint::Length(2), // Status
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🐙 ", Style::default()),
        Span::styled("GitHub Integration", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);

    // Auth status
    let auth_status = if has_token {
        let user_info = if let Some(ref username) = config.github.username {
            format!("Authenticated as @{}", username)
        } else {
            "Authenticated".to_string()
        };
        Paragraph::new(Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::styled(user_info, Style::default().fg(Color::Green)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("  ✗ ", Style::default().fg(Color::Red)),
            Span::styled("Not authenticated — press ", Style::default().fg(Color::Red)),
            Span::styled("[a]", Style::default().fg(Color::Cyan)),
            Span::styled(" to login with GitHub", Style::default().fg(Color::Red)),
        ]))
    };
    let auth_status = auth_status.block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(auth_status, chunks[1]);

    // Menu
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  🔑  ", Style::default()),
            Span::styled("Login with GitHub", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  📦  ", Style::default()),
            Span::styled("Create Repository", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  📤  ", Style::default()),
            Span::styled("Push to Remote", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  👥  ", Style::default()),
            Span::styled("Manage Collaborators", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🚪  ", Style::default()),
            Span::styled("Logout", Style::default().fg(if has_token { Color::Red } else { Color::DarkGray })),
        ])),
    ];

    let menu = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(" Menu ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(menu, chunks[2], &mut state.menu_state);

    // Status
    if let Some(ref msg) = state.status {
        let status = Paragraph::new(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ));
        f.render_widget(status, chunks[3]);
    }
}

fn render_device_auth(f: &mut Frame, area: Rect, auth: &DeviceAuthState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Instructions
            Constraint::Length(5),  // Code display
            Constraint::Length(3),  // URL
            Constraint::Length(3),  // Status
            Constraint::Min(1),    // Spacer
            Constraint::Length(2),  // Keys
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🔑 ", Style::default()),
        Span::styled("Login with GitHub", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. ", Style::default().fg(Color::Cyan)),
            Span::styled("Open the URL below in your browser", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  2. ", Style::default().fg(Color::Cyan)),
            Span::styled("Enter the code shown below", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  3. ", Style::default().fg(Color::Cyan)),
            Span::styled("Authorize zit — we'll detect it automatically", Style::default().fg(Color::White)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(instructions, chunks[1]);

    // User code — big and prominent
    let code_display = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("    Your code:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &auth.user_code,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ])
    .block(
        Block::default()
            .title(Span::styled(" Verification Code ", Style::default().fg(Color::Yellow)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(code_display, chunks[2]);

    // Verification URL
    let url = Paragraph::new(Line::from(vec![
        Span::styled("  Open: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &auth.verification_uri,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(url, chunks[3]);

    // Polling status
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = spinner_chars[(auth.ticks_since_poll as usize) % spinner_chars.len()];
    let status = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {} ", spinner), Style::default().fg(Color::Cyan)),
        Span::styled(&auth.status, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(status, chunks[4]);

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]));
    f.render_widget(keys, chunks[6]);
}

fn render_create_repo(f: &mut Frame, area: Rect, state: &GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name
            Constraint::Length(3), // Description
            Constraint::Length(3), // Visibility
            Constraint::Length(3), // Submit
            Constraint::Min(1),   // Spacer
        ])
        .split(area);

    let title = Paragraph::new(Span::styled(
        "  📦 Create GitHub Repository",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);

    let field_style = |idx: usize| {
        if state.create_field == idx {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let name = Paragraph::new(Line::from(vec![
        Span::styled("  Name: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&state.repo_name, Style::default().fg(Color::White)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(field_style(0)));
    f.render_widget(name, chunks[1]);

    let desc = Paragraph::new(Line::from(vec![
        Span::styled("  Description: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&state.repo_desc, Style::default().fg(Color::White)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(field_style(1)));
    f.render_widget(desc, chunks[2]);

    let vis = Paragraph::new(Line::from(vec![
        Span::styled("  Visibility: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.repo_private { "Private 🔒" } else { "Public 🌍" },
            Style::default().fg(if state.repo_private { Color::Yellow } else { Color::Green }),
        ),
        Span::styled("  (Space to toggle)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(field_style(2)));
    f.render_widget(vis, chunks[3]);

    let submit = Paragraph::new(Span::styled(
        "  [Enter] Create Repository",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).border_style(field_style(3)));
    f.render_widget(submit, chunks[4]);
}

fn render_collaborators(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),   // List
            Constraint::Length(2), // Keys
            Constraint::Length(2), // Status/Error
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  👥 ", Style::default()),
        Span::styled("Collaborators", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  ({} total)", state.collaborators.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);

    // Collaborator list
    if state.collaborators.is_empty() {
        let empty = Paragraph::new(Line::from(vec![
            Span::styled("  No collaborators found. Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("[a]", Style::default().fg(Color::Cyan)),
            Span::styled(" to add one.", Style::default().fg(Color::DarkGray)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = state
            .collaborators
            .iter()
            .map(|c| {
                ListItem::new(Line::from(vec![
                    Span::styled("  @", Style::default().fg(Color::DarkGray)),
                    Span::styled(&c.login, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  ({})", c.role), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut state.collab_list_state);
    }

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [a]", Style::default().fg(Color::Cyan)),
        Span::raw(" Add "),
        Span::styled("[d]", Style::default().fg(Color::Red)),
        Span::raw(" Remove "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" Refresh "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Back"),
    ]));
    f.render_widget(keys, chunks[2]);

    // Status / Error
    if let Some(ref err) = state.collab_error {
        let status = Paragraph::new(Span::styled(
            format!(" {}", err),
            Style::default().fg(Color::Yellow),
        ));
        f.render_widget(status, chunks[3]);
    }
}

pub fn handle_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    let view = app.github_state.view.clone();
    match view {
        GitHubView::Menu => handle_menu_key(app, key),
        GitHubView::DeviceAuth(_) => handle_device_auth_key(app, key),
        GitHubView::CreateRepo => handle_create_repo_key(app, key),
        GitHubView::Collaborators => handle_collaborators_key(app, key),
    }
}

fn handle_menu_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.github_state.menu_selected > 0 {
                app.github_state.menu_selected -= 1;
                let sel = app.github_state.menu_selected;
                app.github_state.menu_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.github_state.menu_selected < 4 {
                app.github_state.menu_selected += 1;
                let sel = app.github_state.menu_selected;
                app.github_state.menu_state.select(Some(sel));
            }
        }
        KeyCode::Enter => {
            match app.github_state.menu_selected {
                0 => {
                    // Login with GitHub — start device flow
                    start_device_flow(app);
                }
                1 => {
                    // Create repo
                    if app.config.github.get_token().is_none() {
                        app.github_state.status = Some("Login first to create a repository".to_string());
                        return Ok(());
                    }
                    app.github_state.view = GitHubView::CreateRepo;
                    app.github_state.repo_name.clear();
                    app.github_state.repo_desc.clear();
                    app.github_state.repo_private = true;
                    app.github_state.create_field = 0;
                    app.github_state.editing_field = true;
                }
                2 => {
                    // Push
                    if let Ok(branch) = git::BranchOps::current() {
                        match git::RemoteOps::push("origin", &branch, true) {
                            Ok(_) => app.github_state.status = Some(format!("✓ Pushed to origin/{}", branch)),
                            Err(e) => app.github_state.status = Some(format!("Push failed: {}", e)),
                        }
                    }
                }
                3 => {
                    // Collaborators — load and switch view
                    if app.config.github.get_token().is_none() {
                        app.github_state.status = Some("Login first to manage collaborators".to_string());
                        return Ok(());
                    }
                    load_collaborators(app);
                    app.github_state.view = GitHubView::Collaborators;
                }
                4 => {
                    // Logout
                    if app.config.github.get_token().is_some() {
                        app.config.github.oauth_token = None;
                        app.config.github.pat = None;
                        app.config.github.username = None;
                        let _ = app.config.save();
                        app.github_state.status = Some("Logged out".to_string());
                    } else {
                        app.github_state.status = Some("Not logged in".to_string());
                    }
                }
                _ => {}
            }
        }
        KeyCode::Char('a') => {
            // Quick auth shortcut
            start_device_flow(app);
        }
        _ => {}
    }

    Ok(())
}

fn start_device_flow(app: &mut crate::app::App) {
    match git::github_auth::request_device_code() {
        Ok(response) => {
            app.github_state.view = GitHubView::DeviceAuth(DeviceAuthState {
                user_code: response.user_code,
                device_code: response.device_code,
                verification_uri: response.verification_uri,
                interval: response.interval,
                ticks_since_poll: 0,
                status: "Waiting for you to authorize in browser...".to_string(),
            });
        }
        Err(e) => {
            app.github_state.status = Some(format!("Auth failed: {}", e));
        }
    }
}

fn handle_device_auth_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    if key.code == KeyCode::Esc {
        app.github_state.view = GitHubView::Menu;
        app.github_state.status = Some("Login cancelled".to_string());
    }
    Ok(())
}

/// Called on every tick event to poll GitHub for authorization status.
pub fn tick_device_auth(app: &mut crate::app::App) {
    let (device_code, _interval) = {
        if let GitHubView::DeviceAuth(ref mut auth) = app.github_state.view {
            auth.ticks_since_poll += 1;
            // interval is in seconds, tick_rate is in ms (typically 2000ms = 2s)
            // We need to wait `interval` seconds between polls
            let ticks_needed = (auth.interval * 1000) / app.config.general.tick_rate_ms;
            let ticks_needed = ticks_needed.max(1);
            if auth.ticks_since_poll < ticks_needed {
                return; // Not time to poll yet
            }
            auth.ticks_since_poll = 0;
            (auth.device_code.clone(), auth.interval)
        } else {
            return; // Not in device auth view
        }
    };

    match git::github_auth::poll_for_token(&device_code) {
        git::github_auth::PollResult::Pending => {
            if let GitHubView::DeviceAuth(ref mut auth) = app.github_state.view {
                auth.status = "Waiting for you to authorize in browser...".to_string();
            }
        }
        git::github_auth::PollResult::Success(token) => {
            // Fetch username
            let username = git::github_auth::get_username(&token.access_token).ok();

            // Save token to config
            app.config.github.oauth_token = Some(token.access_token);
            app.config.github.username = username.clone();
            let _ = app.config.save();

            let msg = if let Some(user) = username {
                format!("✓ Authenticated as @{}", user)
            } else {
                "✓ Authenticated with GitHub".to_string()
            };

            app.github_state.view = GitHubView::Menu;
            app.github_state.status = Some(msg);
        }
        git::github_auth::PollResult::SlowDown(new_interval) => {
            if let GitHubView::DeviceAuth(ref mut auth) = app.github_state.view {
                auth.interval = new_interval;
                auth.status = "Slowing down polling rate...".to_string();
            }
        }
        git::github_auth::PollResult::Expired => {
            app.github_state.view = GitHubView::Menu;
            app.github_state.status = Some("Code expired. Please try again.".to_string());
        }
        git::github_auth::PollResult::AccessDenied => {
            app.github_state.view = GitHubView::Menu;
            app.github_state.status = Some("Access denied. Please try again.".to_string());
        }
        git::github_auth::PollResult::Error(e) => {
            if let GitHubView::DeviceAuth(ref mut auth) = app.github_state.view {
                auth.status = format!("Error: {}", e);
            }
        }
    }
}

fn handle_create_repo_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.github_state.view = GitHubView::Menu;
        }
        KeyCode::Tab | KeyCode::Down => {
            app.github_state.create_field = (app.github_state.create_field + 1).min(3);
        }
        KeyCode::BackTab | KeyCode::Up => {
            if app.github_state.create_field > 0 {
                app.github_state.create_field -= 1;
            }
        }
        KeyCode::Char(' ') if app.github_state.create_field == 2 => {
            app.github_state.repo_private = !app.github_state.repo_private;
        }
        KeyCode::Enter if app.github_state.create_field == 3 => {
            let name = app.github_state.repo_name.trim().to_string();
            if name.is_empty() {
                app.github_state.status = Some("Repository name cannot be empty".to_string());
                return Ok(());
            }

            if let Some(token) = app.config.github.get_token().map(|t| t.to_string()) {
                let desc = app.github_state.repo_desc.clone();
                let private = app.github_state.repo_private;
                match git::github_auth::create_repo(&token, &name, &desc, private) {
                    Ok(clone_url) => {
                        // Add remote origin if not already set
                        let _ = git::RemoteOps::add("origin", &clone_url);
                        app.github_state.status = Some(format!("✓ Created '{}' and added as origin", name));
                        app.github_state.view = GitHubView::Menu;
                    }
                    Err(e) => {
                        app.github_state.status = Some(format!("Error: {}", e));
                    }
                }
            } else {
                app.github_state.status = Some("Login first to create a repository".to_string());
            }
        }
        KeyCode::Char(c) => {
            match app.github_state.create_field {
                0 => app.github_state.repo_name.push(c),
                1 => app.github_state.repo_desc.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match app.github_state.create_field {
                0 => { app.github_state.repo_name.pop(); }
                1 => { app.github_state.repo_desc.pop(); }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

fn load_collaborators(app: &mut crate::app::App) {
    if let Some(token) = app.config.github.get_token().map(|t| t.to_string()) {
        match git::github_auth::list_collaborators(&token) {
            Ok(collabs) => {
                app.github_state.collaborators = collabs;
                app.github_state.collab_selected = 0;
                app.github_state.collab_list_state.select(if app.github_state.collaborators.is_empty() { None } else { Some(0) });
                app.github_state.collab_error = None;
            }
            Err(e) => {
                app.github_state.collab_error = Some(format!("Error: {}", e));
                app.github_state.collaborators.clear();
            }
        }
    }
}

fn handle_collaborators_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.github_state.view = GitHubView::Menu;
            app.github_state.collab_error = None;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.github_state.collab_selected > 0 {
                app.github_state.collab_selected -= 1;
                let sel = app.github_state.collab_selected;
                app.github_state.collab_list_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.github_state.collaborators.is_empty()
                && app.github_state.collab_selected + 1 < app.github_state.collaborators.len()
            {
                app.github_state.collab_selected += 1;
                let sel = app.github_state.collab_selected;
                app.github_state.collab_list_state.select(Some(sel));
            }
        }
        KeyCode::Char('a') => {
            // Add collaborator via input popup
            app.popup = crate::app::Popup::Input {
                title: "Add Collaborator".to_string(),
                prompt: "GitHub username: ".to_string(),
                value: String::new(),
                on_submit: crate::app::InputAction::AddCollaborator,
            };
        }
        KeyCode::Char('d') => {
            // Remove selected collaborator
            let selected = app.github_state.collab_selected;
            if let Some(collab) = app.github_state.collaborators.get(selected) {
                let login = collab.login.clone();
                app.popup = crate::app::Popup::Confirm {
                    title: "Remove Collaborator".to_string(),
                    message: format!(
                        "Remove @{} from this repository?\n\n[y] Yes  [n] No",
                        login
                    ),
                    on_confirm: crate::app::ConfirmAction::RemoveCollaborator(login),
                };
            }
        }
        KeyCode::Char('r') => {
            // Refresh collaborator list
            load_collaborators(app);
        }
        _ => {}
    }

    Ok(())
}
