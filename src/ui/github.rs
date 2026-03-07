use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};
use std::sync::{Arc, Mutex};

use crate::git;

#[derive(Debug, Clone, PartialEq)]
pub enum GitHubView {
    Menu,
    DeviceAuth(DeviceAuthState),
    CreateRepo,
    Collaborators,
    PullRequests,
    PullRequestDetail(u64),
    Actions,
    ActionDetail(u64),
}

// ─── Pull Request UI Types ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrFilter {
    Open,
    Closed,
    All,
}

impl PrFilter {
    pub fn label(&self) -> &str {
        match self {
            PrFilter::Open => "Open",
            PrFilter::Closed => "Closed",
            PrFilter::All => "All",
        }
    }

    pub fn api_state(&self) -> &str {
        match self {
            PrFilter::Open => "open",
            PrFilter::Closed => "closed",
            PrFilter::All => "all",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PrFilter::Open => PrFilter::Closed,
            PrFilter::Closed => PrFilter::All,
            PrFilter::All => PrFilter::Open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrDetailTab {
    Overview,
    Files,
    Reviews,
}

impl PrDetailTab {
    pub fn next(&self) -> Self {
        match self {
            PrDetailTab::Overview => PrDetailTab::Files,
            PrDetailTab::Files => PrDetailTab::Reviews,
            PrDetailTab::Reviews => PrDetailTab::Overview,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl MergeMethod {
    pub fn label(&self) -> &str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::Squash => "squash",
            MergeMethod::Rebase => "rebase",
        }
    }

    pub fn display(&self) -> &str {
        match self {
            MergeMethod::Merge => "Create Merge Commit",
            MergeMethod::Squash => "Squash and Merge",
            MergeMethod::Rebase => "Rebase and Merge",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            MergeMethod::Merge => MergeMethod::Squash,
            MergeMethod::Squash => MergeMethod::Rebase,
            MergeMethod::Rebase => MergeMethod::Merge,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PrBgResult {
    PrList(Result<Vec<git::github_auth::PullRequest>, String>),
    PrDetail {
        pr: Result<git::github_auth::PullRequest, String>,
        checks: Result<git::github_auth::CheckRunsResponse, String>,
        files: Result<Vec<git::github_auth::PrFile>, String>,
        reviews: Result<Vec<git::github_auth::PrReview>, String>,
    },
    MergeResult(Result<git::github_auth::MergeResponse, String>),
    CloseResult(Result<git::github_auth::PullRequest, String>),
}

#[derive(Debug, Clone)]
pub enum ActionsBgResult {
    RunsList(Result<Vec<git::github_auth::WorkflowRun>, String>),
    RunDetail(Result<Vec<git::github_auth::WorkflowJob>, String>),
    JobLogs(u64, Result<String, String>), // (job_id, logs)
}

pub struct ActionsState {
    pub runs: Vec<git::github_auth::WorkflowRun>,
    pub selected: usize,
    pub list_state: ListState,
    pub loading: bool,
    pub error: Option<String>,
    // Detail view
    pub jobs: Vec<git::github_auth::WorkflowJob>,
    pub selected_job: usize,
    pub job_list_state: ListState,
    pub job_logs: Option<String>,
    pub log_scroll: u16,
    pub detail_pane: ActionDetailPane,
    pub bg_result: Arc<Mutex<Option<ActionsBgResult>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionDetailPane {
    Jobs,
    Logs,
}

impl ActionsState {
    pub fn new() -> Self {
        Self {
            runs: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            loading: false,
            error: None,
            jobs: Vec::new(),
            selected_job: 0,
            job_list_state: ListState::default(),
            job_logs: None,
            log_scroll: 0,
            detail_pane: ActionDetailPane::Jobs,
            bg_result: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct PullRequestsState {
    pub prs: Vec<git::github_auth::PullRequest>,
    pub selected: usize,
    pub list_state: ListState,
    pub filter: PrFilter,
    pub loading: bool,
    pub error: Option<String>,
    // Detail view
    pub detail_pr: Option<git::github_auth::PullRequest>,
    pub detail_checks: Option<git::github_auth::CheckRunsResponse>,
    pub detail_files: Vec<git::github_auth::PrFile>,
    pub detail_reviews: Vec<git::github_auth::PrReview>,
    pub detail_tab: PrDetailTab,
    pub detail_scroll: u16,
    pub files_selected: usize,
    pub files_list_state: ListState,
    pub merge_method: MergeMethod,
    pub bg_result: Arc<Mutex<Option<PrBgResult>>>,
}

impl PullRequestsState {
    pub fn new() -> Self {
        Self {
            prs: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            filter: PrFilter::Open,
            loading: false,
            error: None,
            detail_pr: None,
            detail_checks: None,
            detail_files: Vec::new(),
            detail_reviews: Vec::new(),
            detail_tab: PrDetailTab::Overview,
            detail_scroll: 0,
            files_selected: 0,
            files_list_state: ListState::default(),
            merge_method: MergeMethod::Merge,
            bg_result: Arc::new(Mutex::new(None)),
        }
    }
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
    // Background operation result
    pub bg_result: Arc<Mutex<Option<String>>>,
    // Pull-request state
    pub pr_state: PullRequestsState,
    // Actions state
    pub actions_state: ActionsState,
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
            bg_result: Arc::new(Mutex::new(None)),
            pr_state: PullRequestsState::new(),
            actions_state: ActionsState::new(),
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
        GitHubView::PullRequests => render_pull_requests(f, area, state),
        GitHubView::PullRequestDetail(_) => render_pr_detail(f, area, state),
        GitHubView::Actions => render_actions_list(f, area, state),
        GitHubView::ActionDetail(_) => render_action_detail(f, area, state),
    }
}

fn render_menu(
    f: &mut Frame,
    area: Rect,
    state: &mut GitHubState,
    has_token: bool,
    config: &crate::config::Config,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Auth status
            Constraint::Min(8),    // Menu
            Constraint::Length(2), // Status
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🐙 ", Style::default()),
        Span::styled(
            "GitHub Integration",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
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
            Span::styled(
                "Not authenticated — press ",
                Style::default().fg(Color::Red),
            ),
            Span::styled("[a]", Style::default().fg(Color::Cyan)),
            Span::styled(" to login with GitHub", Style::default().fg(Color::Red)),
        ]))
    };
    let auth_status = auth_status.block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
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
            Span::styled("  📥  ", Style::default()),
            Span::styled("Pull from Remote", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🔄  ", Style::default()),
            Span::styled("Sync (Pull + Push)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  👥  ", Style::default()),
            Span::styled("Manage Collaborators", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🔀  ", Style::default()),
            Span::styled("Pull Requests", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  ⚡  ", Style::default()),
            Span::styled("Actions", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🚪  ", Style::default()),
            Span::styled(
                "Logout",
                Style::default().fg(if has_token {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ),
        ])),
    ];

    let menu = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(" Menu ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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
            Constraint::Length(3), // Title
            Constraint::Length(5), // Instructions
            Constraint::Length(5), // Code display
            Constraint::Length(3), // URL
            Constraint::Length(3), // Status
            Constraint::Min(1),    // Spacer
            Constraint::Length(2), // Keys
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🔑 ", Style::default()),
        Span::styled(
            "Login with GitHub",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Open the URL below in your browser",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  2. ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Enter the code shown below",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  3. ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Authorize zit — we'll detect it automatically",
                Style::default().fg(Color::White),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
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
            .title(Span::styled(
                " Verification Code ",
                Style::default().fg(Color::Yellow),
            ))
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(url, chunks[3]);

    // Polling status
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = spinner_chars[(auth.ticks_since_poll as usize) % spinner_chars.len()];
    let status = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {} ", spinner), Style::default().fg(Color::Cyan)),
        Span::styled(&auth.status, Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
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
            Constraint::Min(1),    // Spacer
        ])
        .split(area);

    let title = Paragraph::new(Span::styled(
        "  📦 Create GitHub Repository",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(field_style(0)),
    );
    f.render_widget(name, chunks[1]);

    let desc = Paragraph::new(Line::from(vec![
        Span::styled("  Description: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&state.repo_desc, Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(field_style(1)),
    );
    f.render_widget(desc, chunks[2]);

    let vis = Paragraph::new(Line::from(vec![
        Span::styled("  Visibility: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.repo_private {
                "Private 🔒"
            } else {
                "Public 🌍"
            },
            Style::default().fg(if state.repo_private {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
        Span::styled("  (Space to toggle)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(field_style(2)),
    );
    f.render_widget(vis, chunks[3]);

    let submit = Paragraph::new(Span::styled(
        "  [Enter] Create Repository",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(field_style(3)),
    );
    f.render_widget(submit, chunks[4]);
}

fn render_collaborators(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // List
            Constraint::Length(2), // Keys
            Constraint::Length(2), // Status/Error
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  👥 ", Style::default()),
        Span::styled(
            "Collaborators",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ({} total)", state.collaborators.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Collaborator list
    if state.collaborators.is_empty() {
        let empty = Paragraph::new(Line::from(vec![
            Span::styled(
                "  No collaborators found. Press ",
                Style::default().fg(Color::DarkGray),
            ),
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
                    Span::styled(
                        &c.login,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  ({})", c.role),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
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
        GitHubView::PullRequests => handle_pull_requests_key(app, key),
        GitHubView::PullRequestDetail(_) => handle_pr_detail_key(app, key),
        GitHubView::Actions => handle_actions_key(app, key),
        GitHubView::ActionDetail(_) => handle_action_detail_key(app, key),
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
            if app.github_state.menu_selected < 8 {
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
                        app.github_state.status =
                            Some("Login first to create a repository".to_string());
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
                    // Push — run in background thread
                    if let Ok(branch) = git::BranchOps::current() {
                        app.github_state.status = Some("⏳ Pushing...".to_string());
                        let bg = app.github_state.bg_result.clone();
                        let br = branch.clone();
                        std::thread::spawn(move || {
                            let result = match git::RemoteOps::push("origin", &br, true) {
                                Ok(_) => format!("✓ Pushed to origin/{}", br),
                                Err(e) => format!("Push failed: {}", e),
                            };
                            if let Ok(mut r) = bg.lock() {
                                *r = Some(result);
                            }
                        });
                    }
                }
                3 => {
                    // Pull — run in background thread
                    if let Ok(branch) = git::BranchOps::current() {
                        app.github_state.status = Some("⏳ Pulling...".to_string());
                        let bg = app.github_state.bg_result.clone();
                        let br = branch.clone();
                        std::thread::spawn(move || {
                            let result = match git::RemoteOps::pull("origin", &br) {
                                Ok(_) => format!("✓ Pulled from origin/{}", br),
                                Err(e) => format!("Pull failed: {}", e),
                            };
                            if let Ok(mut r) = bg.lock() {
                                *r = Some(result);
                            }
                        });
                    }
                }
                4 => {
                    // Sync — pull then push in background
                    if let Ok(branch) = git::BranchOps::current() {
                        app.github_state.status = Some("⏳ Syncing (pull + push)...".to_string());
                        let bg = app.github_state.bg_result.clone();
                        let br = branch.clone();
                        std::thread::spawn(move || {
                            let pull = git::RemoteOps::pull("origin", &br);
                            let result = match pull {
                                Ok(_) => match git::RemoteOps::push("origin", &br, true) {
                                    Ok(_) => format!("✓ Synced with origin/{}", br),
                                    Err(e) => format!("Push failed after pull: {}", e),
                                },
                                Err(e) => format!("Pull failed: {}", e),
                            };
                            if let Ok(mut r) = bg.lock() {
                                *r = Some(result);
                            }
                        });
                    }
                }
                5 => {
                    // Collaborators — load and switch view
                    if app.config.github.get_token().is_none() {
                        app.github_state.status =
                            Some("Login first to manage collaborators".to_string());
                        return Ok(());
                    }
                    load_collaborators(app);
                    app.github_state.view = GitHubView::Collaborators;
                }
                6 => {
                    // Pull Requests
                    if app.config.github.get_token().is_none() {
                        app.github_state.status =
                            Some("Login first to view pull requests".to_string());
                        return Ok(());
                    }
                    start_load_prs(app);
                    app.github_state.view = GitHubView::PullRequests;
                }
                7 => {
                    // Actions
                    if app.config.github.get_token().is_none() {
                        app.github_state.status = Some("Login first to view Actions".to_string());
                        return Ok(());
                    }
                    start_load_actions(app);
                    app.github_state.view = GitHubView::Actions;
                }
                8 => {
                    // Logout — clear keychain and config
                    if app.config.github.get_token().is_some() {
                        crate::keychain::clear_all();
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
    // Check for background operation results (push/pull/sync)
    if let Ok(mut result) = app.github_state.bg_result.try_lock() {
        if let Some(msg) = result.take() {
            app.github_state.status = Some(msg);
        }
    }

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

            // Store token in OS keychain (fall back to config if keychain fails)
            if crate::keychain::store_github_token(&token.access_token).is_err() {
                log::warn!("Keychain unavailable, storing token in config file");
                app.config.github.oauth_token = Some(token.access_token);
            }
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

            if let Some(token) = app.config.github.get_token() {
                let desc = app.github_state.repo_desc.clone();
                let private = app.github_state.repo_private;
                match git::github_auth::create_repo(&token, &name, &desc, private) {
                    Ok(clone_url) => {
                        // Add remote origin if not already set
                        let _ = git::RemoteOps::add("origin", &clone_url);
                        app.github_state.status =
                            Some(format!("✓ Created '{}' and added as origin", name));
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
        KeyCode::Char(c) => match app.github_state.create_field {
            0 => app.github_state.repo_name.push(c),
            1 => app.github_state.repo_desc.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app.github_state.create_field {
            0 => {
                app.github_state.repo_name.pop();
            }
            1 => {
                app.github_state.repo_desc.pop();
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

fn load_collaborators(app: &mut crate::app::App) {
    if let Some(token) = app.config.github.get_token() {
        match git::github_auth::list_collaborators(&token) {
            Ok(collabs) => {
                app.github_state.collaborators = collabs;
                app.github_state.collab_selected = 0;
                app.github_state.collab_list_state.select(
                    if app.github_state.collaborators.is_empty() {
                        None
                    } else {
                        Some(0)
                    },
                );
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
                    message: format!("Remove @{} from this repository?\n\n[y] Yes  [n] No", login),
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

// ─── Pull Request Loading ────────────────────────────────

fn start_load_prs(app: &mut crate::app::App) {
    app.github_state.pr_state.loading = true;
    app.github_state.pr_state.error = None;
    let token = app.config.github.get_token().unwrap_or_default();
    let filter = app.github_state.pr_state.filter.api_state().to_string();
    let bg = app.github_state.pr_state.bg_result.clone();
    std::thread::spawn(move || {
        let result =
            git::github_auth::list_pull_requests(&token, &filter).map_err(|e| e.to_string());
        if let Ok(mut r) = bg.lock() {
            *r = Some(PrBgResult::PrList(result));
        }
    });
}

fn start_load_pr_detail(app: &mut crate::app::App, number: u64) {
    app.github_state.pr_state.loading = true;
    app.github_state.pr_state.error = None;
    app.github_state.pr_state.detail_tab = PrDetailTab::Overview;
    app.github_state.pr_state.detail_scroll = 0;
    app.github_state.pr_state.files_selected = 0;
    let token = app.config.github.get_token().unwrap_or_default();
    let bg = app.github_state.pr_state.bg_result.clone();
    std::thread::spawn(move || {
        let pr = git::github_auth::get_pull_request(&token, number).map_err(|e| e.to_string());
        let sha = pr.as_ref().map(|p| p.head.sha.clone()).unwrap_or_default();
        let checks = git::github_auth::get_check_runs(&token, &sha).map_err(|e| e.to_string());
        let files = git::github_auth::get_pr_files(&token, number).map_err(|e| e.to_string());
        let reviews = git::github_auth::get_pr_reviews(&token, number).map_err(|e| e.to_string());
        if let Ok(mut r) = bg.lock() {
            *r = Some(PrBgResult::PrDetail {
                pr,
                checks,
                files,
                reviews,
            });
        }
    });
}

/// Called on each tick to poll for PR background results.
pub fn tick_pr_state(app: &mut crate::app::App) {
    let bg_taken = {
        if let Ok(mut result) = app.github_state.pr_state.bg_result.try_lock() {
            result.take()
        } else {
            None
        }
    };

    if let Some(bg) = bg_taken {
        app.github_state.pr_state.loading = false;
        match bg {
            PrBgResult::PrList(Ok(prs)) => {
                app.github_state.pr_state.prs = prs;
                app.github_state.pr_state.selected = 0;
                app.github_state.pr_state.list_state.select(
                    if app.github_state.pr_state.prs.is_empty() {
                        None
                    } else {
                        Some(0)
                    },
                );
                app.github_state.pr_state.error = None;
            }
            PrBgResult::PrList(Err(e)) => {
                app.github_state.pr_state.error = Some(e);
            }
            PrBgResult::PrDetail {
                pr,
                checks,
                files,
                reviews,
            } => {
                match pr {
                    Ok(p) => {
                        app.github_state.pr_state.detail_pr = Some(p);
                        app.github_state.pr_state.error = None;
                    }
                    Err(e) => {
                        app.github_state.pr_state.error = Some(e);
                    }
                }
                if let Ok(c) = checks {
                    app.github_state.pr_state.detail_checks = Some(c);
                }
                if let Ok(f) = files {
                    app.github_state.pr_state.detail_files = f;
                    app.github_state.pr_state.files_list_state.select(
                        if app.github_state.pr_state.detail_files.is_empty() {
                            None
                        } else {
                            Some(0)
                        },
                    );
                }
                if let Ok(r) = reviews {
                    app.github_state.pr_state.detail_reviews = r;
                }
            }
            PrBgResult::MergeResult(Ok(resp)) => {
                if resp.merged {
                    app.github_state.status = Some(format!("✓ PR merged! ({})", resp.sha));
                    if let GitHubView::PullRequestDetail(n) = app.github_state.view {
                        start_load_pr_detail(app, n);
                    }
                } else {
                    app.github_state.pr_state.error =
                        Some(format!("Merge failed: {}", resp.message));
                }
            }
            PrBgResult::MergeResult(Err(e)) => {
                app.github_state.pr_state.error = Some(format!("Merge failed: {}", e));
            }
            PrBgResult::CloseResult(Ok(pr)) => {
                app.github_state.status = Some(format!("✓ PR #{} closed", pr.number));
                if let GitHubView::PullRequestDetail(n) = app.github_state.view {
                    start_load_pr_detail(app, n);
                }
            }
            PrBgResult::CloseResult(Err(e)) => {
                app.github_state.pr_state.error = Some(format!("Close failed: {}", e));
            }
        }
    }
}

// ─── Pull Request List Rendering ────────────────────────────────

fn render_pull_requests(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Filter bar
            Constraint::Min(6),    // PR list
            Constraint::Length(2), // Keys
            Constraint::Length(2), // Status/Error
        ])
        .split(area);

    // Title
    let pr_count = state.pr_state.prs.len();
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  🔀 ", Style::default()),
        Span::styled(
            "Pull Requests",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  ({} {})",
                pr_count,
                state.pr_state.filter.label().to_lowercase()
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Filter bar
    let filters = ["Open", "Closed", "All"];
    let selected_idx = match state.pr_state.filter {
        PrFilter::Open => 0,
        PrFilter::Closed => 1,
        PrFilter::All => 2,
    };
    let tabs = Tabs::new(filters.iter().map(|f| Line::from(*f)).collect::<Vec<_>>())
        .block(
            Block::default()
                .title(Span::styled(
                    " Filter [f] ",
                    Style::default().fg(Color::DarkGray),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(selected_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[1]);

    // PR list
    if state.pr_state.loading {
        let loading = Paragraph::new(Line::from(vec![
            Span::styled("  ⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Loading pull requests...",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(loading, chunks[2]);
    } else if state.pr_state.prs.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            "  No pull requests found.",
            Style::default().fg(Color::DarkGray),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, chunks[2]);
    } else {
        let items: Vec<ListItem> = state
            .pr_state
            .prs
            .iter()
            .map(|pr| {
                let state_icon = if pr.draft {
                    Span::styled("  📝 ", Style::default())
                } else if pr.state == "open" {
                    Span::styled("  🟢 ", Style::default())
                } else if pr.merged_at.is_some() {
                    Span::styled("  🟣 ", Style::default())
                } else {
                    Span::styled("  🔴 ", Style::default())
                };

                let number =
                    Span::styled(format!("#{} ", pr.number), Style::default().fg(Color::Cyan));
                let title_text = Span::styled(
                    pr.title.chars().take(60).collect::<String>(),
                    Style::default().fg(Color::White),
                );
                let author = Span::styled(
                    format!("  @{}", pr.user.login),
                    Style::default().fg(Color::DarkGray),
                );
                let stats = Span::styled(
                    format!(
                        "  +{} -{}",
                        pr.additions.unwrap_or(0),
                        pr.deletions.unwrap_or(0)
                    ),
                    Style::default().fg(Color::DarkGray),
                );

                ListItem::new(Line::from(vec![
                    state_icon, number, title_text, author, stats,
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[2], &mut state.pr_state.list_state);
    }

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Open "),
        Span::styled("[f]", Style::default().fg(Color::Yellow)),
        Span::raw(" Filter "),
        Span::styled("[r]", Style::default().fg(Color::Green)),
        Span::raw(" Refresh "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Back"),
    ]));
    f.render_widget(keys, chunks[3]);

    // Error
    if let Some(ref err) = state.pr_state.error {
        let status = Paragraph::new(Span::styled(
            format!(" {}", err),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(status, chunks[4]);
    }
}

// ─── Pull Request Detail Rendering ────────────────────────────────

fn render_pr_detail(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Title + meta
            Constraint::Length(3), // Tab bar
            Constraint::Min(8),    // Tab content
            Constraint::Length(2), // Keys
            Constraint::Length(2), // Error
        ])
        .split(area);

    let pr = state.pr_state.detail_pr.as_ref();

    // Title + meta
    if state.pr_state.loading && pr.is_none() {
        let loading = Paragraph::new(Line::from(vec![
            Span::styled("  ⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Loading PR details...",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(loading, chunks[0]);
    } else if let Some(pr) = pr {
        let state_color = if pr.state == "open" {
            Color::Green
        } else if pr.merged_at.is_some() {
            Color::Magenta
        } else {
            Color::Red
        };
        let state_label = if pr.merged_at.is_some() {
            "MERGED"
        } else {
            &pr.state.to_uppercase()
        };
        let mergeable_info = match pr.mergeable {
            Some(true) => Span::styled(" ✓ Mergeable", Style::default().fg(Color::Green)),
            Some(false) => Span::styled(" ✗ Conflicts", Style::default().fg(Color::Red)),
            None => Span::styled(" ? Checking...", Style::default().fg(Color::Yellow)),
        };

        let title_block = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("  #{} ", pr.number),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &pr.title,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!(" {} ", state_label),
                    Style::default()
                        .fg(Color::Black)
                        .bg(state_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} → {}", pr.head.ref_name, pr.base.ref_name),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("  by @{}", pr.user.login),
                    Style::default().fg(Color::DarkGray),
                ),
                mergeable_info,
            ]),
            Line::from(vec![Span::styled(
                format!(
                    "  {} files changed  +{} -{}",
                    pr.changed_files.unwrap_or(0),
                    pr.additions.unwrap_or(0),
                    pr.deletions.unwrap_or(0)
                ),
                Style::default().fg(Color::DarkGray),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(title_block, chunks[0]);
    }

    // Tab bar
    let tab_titles = ["Overview", "Files", "Reviews"];
    let selected_tab = match state.pr_state.detail_tab {
        PrDetailTab::Overview => 0,
        PrDetailTab::Files => 1,
        PrDetailTab::Reviews => 2,
    };
    let tabs = Tabs::new(
        tab_titles
            .iter()
            .map(|t| Line::from(*t))
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title(Span::styled(
                " [Tab] Switch ",
                Style::default().fg(Color::DarkGray),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .select(selected_tab)
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(tabs, chunks[1]);

    // Tab content
    match state.pr_state.detail_tab {
        PrDetailTab::Overview => render_pr_overview(f, chunks[2], state),
        PrDetailTab::Files => render_pr_files(f, chunks[2], state),
        PrDetailTab::Reviews => render_pr_reviews(f, chunks[2], state),
    }

    // Keys
    let keys = if let Some(pr) = state.pr_state.detail_pr.as_ref() {
        if pr.state == "open" {
            Line::from(vec![
                Span::styled(" [Tab]", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch Tab "),
                Span::styled("[m]", Style::default().fg(Color::Green)),
                Span::raw(format!(" {} ", state.pr_state.merge_method.display())),
                Span::styled("[M]", Style::default().fg(Color::Yellow)),
                Span::raw(" Cycle Method "),
                Span::styled("[c]", Style::default().fg(Color::Red)),
                Span::raw(" Close "),
                Span::styled("[o]", Style::default().fg(Color::Cyan)),
                Span::raw(" Browser "),
                Span::styled("[r]", Style::default().fg(Color::Green)),
                Span::raw(" Refresh "),
                Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
                Span::raw(" Back"),
            ])
        } else {
            Line::from(vec![
                Span::styled(" [Tab]", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch Tab "),
                Span::styled("[o]", Style::default().fg(Color::Cyan)),
                Span::raw(" Browser "),
                Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
                Span::raw(" Back"),
            ])
        }
    } else {
        Line::from(vec![
            Span::styled(" [Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Back"),
        ])
    };
    f.render_widget(Paragraph::new(keys), chunks[3]);

    // Error
    if let Some(ref err) = state.pr_state.error {
        let status = Paragraph::new(Span::styled(
            format!(" {}", err),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(status, chunks[4]);
    }
}

fn render_pr_overview(f: &mut Frame, area: Rect, state: &GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Checks
            Constraint::Min(4),    // Body / description
        ])
        .split(area);

    // CI Checks
    if let Some(ref checks) = state.pr_state.detail_checks {
        let mut lines = vec![Line::from(vec![Span::styled(
            format!("  Checks ({} total)", checks.total_count),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )])];

        for run in checks.check_runs.iter().take(5) {
            let icon = match run.conclusion.as_deref() {
                Some("success") => Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                Some("failure") => Span::styled("  ✗ ", Style::default().fg(Color::Red)),
                Some("neutral") | Some("skipped") => {
                    Span::styled("  ○ ", Style::default().fg(Color::DarkGray))
                }
                _ => Span::styled("  ◐ ", Style::default().fg(Color::Yellow)),
            };
            let name = Span::styled(&run.name, Style::default().fg(Color::White));
            let status = Span::styled(
                format!("  ({})", run.conclusion.as_deref().unwrap_or(&run.status)),
                Style::default().fg(Color::DarkGray),
            );
            lines.push(Line::from(vec![icon, name, status]));
        }

        if checks.total_count > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", checks.total_count - 5),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let checks_block = Paragraph::new(lines).block(
            Block::default()
                .title(Span::styled(
                    " CI Status ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(checks_block, chunks[0]);
    } else {
        let no_checks = Paragraph::new(Span::styled(
            "  No check data available",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .title(Span::styled(
                    " CI Status ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(no_checks, chunks[0]);
    }

    // Body
    if let Some(ref pr) = state.pr_state.detail_pr {
        let body_text = pr.body.as_deref().unwrap_or("No description provided.");
        let body_lines: Vec<Line> = body_text
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    format!("  {}", l),
                    Style::default().fg(Color::White),
                ))
            })
            .collect();

        let body = Paragraph::new(body_lines)
            .scroll((state.pr_state.detail_scroll, 0))
            .block(
                Block::default()
                    .title(Span::styled(
                        " Description ",
                        Style::default().fg(Color::White),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        f.render_widget(body, chunks[1]);
    }
}

fn render_pr_files(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    if state.pr_state.detail_files.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No files changed",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .title(Span::styled(
                    " Changed Files ",
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = state
        .pr_state
        .detail_files
        .iter()
        .map(|file| {
            let status_icon = match file.status.as_str() {
                "added" => Span::styled("  A ", Style::default().fg(Color::Green)),
                "removed" => Span::styled("  D ", Style::default().fg(Color::Red)),
                "modified" => Span::styled("  M ", Style::default().fg(Color::Yellow)),
                "renamed" => Span::styled("  R ", Style::default().fg(Color::Cyan)),
                _ => Span::styled("  ? ", Style::default().fg(Color::DarkGray)),
            };
            let filename = Span::styled(&file.filename, Style::default().fg(Color::White));
            let changes = Span::styled(
                format!("  +{} -{}", file.additions, file.deletions),
                Style::default().fg(Color::DarkGray),
            );
            ListItem::new(Line::from(vec![status_icon, filename, changes]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Changed Files ({}) ", state.pr_state.detail_files.len()),
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state.pr_state.files_list_state);
}

fn render_pr_reviews(f: &mut Frame, area: Rect, state: &GitHubState) {
    if state.pr_state.detail_reviews.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No reviews yet",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .title(Span::styled(" Reviews ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for review in &state.pr_state.detail_reviews {
        let icon = match review.state.as_str() {
            "APPROVED" => Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            "CHANGES_REQUESTED" => Span::styled("  ✗ ", Style::default().fg(Color::Red)),
            "COMMENTED" => Span::styled("  💬 ", Style::default().fg(Color::Cyan)),
            "DISMISSED" => Span::styled("  ○ ", Style::default().fg(Color::DarkGray)),
            _ => Span::styled("  ? ", Style::default().fg(Color::DarkGray)),
        };
        let user = Span::styled(
            format!("@{}", review.user.login),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
        let state_text = Span::styled(
            format!("  {}", review.state.replace('_', " ").to_lowercase()),
            Style::default().fg(Color::DarkGray),
        );
        lines.push(Line::from(vec![icon, user, state_text]));

        if let Some(ref body) = review.body {
            if !body.is_empty() {
                for body_line in body.lines().take(3) {
                    lines.push(Line::from(Span::styled(
                        format!("      {}", body_line),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
        lines.push(Line::from(""));
    }

    let reviews_widget = Paragraph::new(lines)
        .scroll((state.pr_state.detail_scroll, 0))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Reviews ({}) ", state.pr_state.detail_reviews.len()),
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(reviews_widget, area);
}

// ─── Pull Request Key Handlers ────────────────────────────────

fn handle_pull_requests_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.github_state.view = GitHubView::Menu;
            app.github_state.pr_state.error = None;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.github_state.pr_state.selected > 0 {
                app.github_state.pr_state.selected -= 1;
                let sel = app.github_state.pr_state.selected;
                app.github_state.pr_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.github_state.pr_state.prs.is_empty()
                && app.github_state.pr_state.selected + 1 < app.github_state.pr_state.prs.len()
            {
                app.github_state.pr_state.selected += 1;
                let sel = app.github_state.pr_state.selected;
                app.github_state.pr_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Enter => {
            if let Some(pr) = app
                .github_state
                .pr_state
                .prs
                .get(app.github_state.pr_state.selected)
            {
                let number = pr.number;
                app.github_state.view = GitHubView::PullRequestDetail(number);
                start_load_pr_detail(app, number);
            }
        }
        KeyCode::Char('f') => {
            app.github_state.pr_state.filter = app.github_state.pr_state.filter.next();
            start_load_prs(app);
        }
        KeyCode::Char('r') => {
            start_load_prs(app);
        }
        _ => {}
    }
    Ok(())
}

fn handle_pr_detail_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.github_state.view = GitHubView::PullRequests;
            app.github_state.pr_state.error = None;
            // Refresh list in case merge/close changed things
            start_load_prs(app);
        }
        KeyCode::Tab => {
            app.github_state.pr_state.detail_tab = app.github_state.pr_state.detail_tab.next();
            app.github_state.pr_state.detail_scroll = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => match app.github_state.pr_state.detail_tab {
            PrDetailTab::Files => {
                if app.github_state.pr_state.files_selected > 0 {
                    app.github_state.pr_state.files_selected -= 1;
                    let sel = app.github_state.pr_state.files_selected;
                    app.github_state.pr_state.files_list_state.select(Some(sel));
                }
            }
            _ => {
                if app.github_state.pr_state.detail_scroll > 0 {
                    app.github_state.pr_state.detail_scroll -= 1;
                }
            }
        },
        KeyCode::Down | KeyCode::Char('j') => match app.github_state.pr_state.detail_tab {
            PrDetailTab::Files => {
                if !app.github_state.pr_state.detail_files.is_empty()
                    && app.github_state.pr_state.files_selected + 1
                        < app.github_state.pr_state.detail_files.len()
                {
                    app.github_state.pr_state.files_selected += 1;
                    let sel = app.github_state.pr_state.files_selected;
                    app.github_state.pr_state.files_list_state.select(Some(sel));
                }
            }
            _ => {
                app.github_state.pr_state.detail_scroll += 1;
            }
        },
        KeyCode::Char('m') => {
            // Merge PR
            if let Some(pr) = app.github_state.pr_state.detail_pr.as_ref() {
                if pr.state == "open" {
                    let number = pr.number;
                    let method = app.github_state.pr_state.merge_method.label().to_string();
                    app.popup = crate::app::Popup::Confirm {
                        title: "Merge Pull Request".to_string(),
                        message: format!(
                            "Merge PR #{} using {}?\n\n[y] Yes  [n] No",
                            number,
                            app.github_state.pr_state.merge_method.display()
                        ),
                        on_confirm: crate::app::ConfirmAction::MergePullRequest { number, method },
                    };
                }
            }
        }
        KeyCode::Char('M') => {
            // Cycle merge method
            app.github_state.pr_state.merge_method = app.github_state.pr_state.merge_method.next();
        }
        KeyCode::Char('c') => {
            // Close PR
            if let Some(pr) = app.github_state.pr_state.detail_pr.as_ref() {
                if pr.state == "open" {
                    let number = pr.number;
                    app.popup = crate::app::Popup::Confirm {
                        title: "Close Pull Request".to_string(),
                        message: format!(
                            "Close PR #{} without merging?\n\n[y] Yes  [n] No",
                            number,
                        ),
                        on_confirm: crate::app::ConfirmAction::ClosePullRequest(number),
                    };
                }
            }
        }
        KeyCode::Char('o') => {
            // Open in browser
            if let Some(pr) = app.github_state.pr_state.detail_pr.as_ref() {
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&pr.html_url).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open")
                    .arg(&pr.html_url)
                    .spawn();
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", &pr.html_url])
                    .spawn();
            }
        }
        KeyCode::Char('r') => {
            // Refresh
            if let GitHubView::PullRequestDetail(n) = app.github_state.view {
                start_load_pr_detail(app, n);
            }
        }
        _ => {}
    }
    Ok(())
}

// ─── GitHub Actions Rendering ────────────────────────────────

fn status_color(status: Option<&str>, conclusion: Option<&str>) -> Color {
    match conclusion.unwrap_or("") {
        "success" => Color::Green,
        "failure" | "timed_out" => Color::Red,
        "cancelled" => Color::DarkGray,
        "skipped" => Color::DarkGray,
        _ => match status.unwrap_or("") {
            "in_progress" => Color::Cyan,
            "queued" | "waiting" | "pending" => Color::Yellow,
            "completed" => Color::Green,
            _ => Color::White,
        },
    }
}

fn status_icon(status: Option<&str>, conclusion: Option<&str>) -> &'static str {
    match conclusion.unwrap_or("") {
        "success" => "✅",
        "failure" | "timed_out" => "❌",
        "cancelled" => "⚪",
        "skipped" => "⏭",
        _ => match status.unwrap_or("") {
            "in_progress" => "🔵",
            "queued" | "waiting" | "pending" => "🟡",
            _ => "⚪",
        },
    }
}

fn render_actions_list(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),    // Runs list
            Constraint::Length(2), // Keys
            Constraint::Length(2), // Status/Error
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  ⚡ ", Style::default()),
        Span::styled(
            "GitHub Actions",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ({} runs)", state.actions_state.runs.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Loading state
    if state.actions_state.loading && state.actions_state.runs.is_empty() {
        let loading = Paragraph::new(Line::from(vec![
            Span::styled("  ⏳ ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Loading workflow runs...",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(loading, chunks[1]);
    } else if state.actions_state.runs.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            "  No workflow runs found.",
            Style::default().fg(Color::DarkGray),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = state
            .actions_state
            .runs
            .iter()
            .map(|run| {
                let status_str = run.status.as_deref().unwrap_or("unknown");
                let conclusion_str = run.conclusion.as_deref();
                let icon = status_icon(Some(status_str), conclusion_str);
                let color = status_color(Some(status_str), conclusion_str);
                let name = run
                    .display_title
                    .as_deref()
                    .or(run.name.as_deref())
                    .unwrap_or("Workflow");
                let branch = run.head_branch.as_deref().unwrap_or("");
                let status_label = conclusion_str.unwrap_or(status_str);

                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", icon), Style::default()),
                    Span::styled(
                        format!("#{} ", run.run_number),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        truncate_str(name, 35),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("  {} ", branch), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        status_label.to_string(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Span::styled(
                        " Workflow Runs (newest first) ",
                        Style::default().fg(Color::White),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut state.actions_state.list_state);
    }

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" View "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" Refresh "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Back"),
    ]));
    f.render_widget(keys, chunks[2]);

    // Error
    if let Some(ref err) = state.actions_state.error {
        let status = Paragraph::new(Span::styled(
            format!(" ❌ {}", err),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(status, chunks[3]);
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}

fn render_action_detail(f: &mut Frame, area: Rect, state: &mut GitHubState) {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Content (split: jobs | logs)
            Constraint::Length(2), // Keys
        ])
        .split(area);

    // Title — show run info
    let run_title = if let GitHubView::ActionDetail(run_id) = state.view {
        let run = state.actions_state.runs.iter().find(|r| r.id == run_id);
        if let Some(run) = run {
            let name = run
                .display_title
                .as_deref()
                .or(run.name.as_deref())
                .unwrap_or("Workflow");
            let icon = status_icon(run.status.as_deref(), run.conclusion.as_deref());
            format!("  {} #{} — {}", icon, run.run_number, name)
        } else {
            "  Workflow Run".to_string()
        }
    } else {
        "  Workflow Run".to_string()
    };

    let title = Paragraph::new(Span::styled(
        run_title,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, outer_chunks[0]);

    // Split content area: jobs (left) | logs (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Jobs list
            Constraint::Percentage(65), // Logs
        ])
        .split(outer_chunks[1]);

    // Jobs list
    if state.actions_state.loading && state.actions_state.jobs.is_empty() {
        let loading = Paragraph::new(Span::styled(
            "  ⏳ Loading jobs...",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .title(Span::styled(" Jobs ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if state.actions_state.detail_pane == ActionDetailPane::Jobs {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    },
                )),
        );
        f.render_widget(loading, content_chunks[0]);
    } else {
        let job_items: Vec<ListItem> = state
            .actions_state
            .jobs
            .iter()
            .map(|job| {
                let icon = status_icon(Some(job.status.as_str()), job.conclusion.as_deref());
                let color = status_color(Some(job.status.as_str()), job.conclusion.as_deref());

                let mut spans = vec![
                    Span::styled(format!("  {} ", icon), Style::default()),
                    Span::styled(&job.name, Style::default().fg(color)),
                ];

                // Show step count
                if !job.steps.is_empty() {
                    let completed = job.steps.iter().filter(|s| s.status == "completed").count();
                    spans.push(Span::styled(
                        format!("  ({}/{})", completed, job.steps.len()),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let job_list = List::new(job_items)
            .block(
                Block::default()
                    .title(Span::styled(" Jobs ", Style::default().fg(Color::White)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(
                        if state.actions_state.detail_pane == ActionDetailPane::Jobs {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        },
                    )),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(
            job_list,
            content_chunks[0],
            &mut state.actions_state.job_list_state,
        );
    }

    // Logs pane
    let logs_border_color = if state.actions_state.detail_pane == ActionDetailPane::Logs {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    if let Some(ref logs) = state.actions_state.job_logs {
        let log_lines: Vec<Line> = logs
            .lines()
            .skip(state.actions_state.log_scroll as usize)
            .map(|line| {
                let color =
                    if line.contains("error") || line.contains("Error") || line.contains("FAIL") {
                        Color::Red
                    } else if line.contains("warning") || line.contains("Warning") {
                        Color::Yellow
                    } else if line.contains("##[group]") || line.starts_with("Run ") {
                        Color::Cyan
                    } else {
                        Color::White
                    };
                Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(color),
                ))
            })
            .collect();

        let job_name = state
            .actions_state
            .jobs
            .get(state.actions_state.selected_job)
            .map(|j| j.name.as_str())
            .unwrap_or("Logs");

        let logs_widget = Paragraph::new(log_lines).block(
            Block::default()
                .title(Span::styled(
                    format!(" {} — Logs ", job_name),
                    Style::default().fg(Color::White),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(logs_border_color)),
        );
        f.render_widget(logs_widget, content_chunks[1]);
    } else {
        let placeholder = Paragraph::new(Line::from(vec![
            Span::styled(
                "  Select a job and press ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::styled(" to load logs", Style::default().fg(Color::DarkGray)),
        ]))
        .block(
            Block::default()
                .title(Span::styled(" Logs ", Style::default().fg(Color::White)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(logs_border_color)),
        );
        f.render_widget(placeholder, content_chunks[1]);
    }

    // Keys
    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(Color::Cyan)),
        Span::raw(" Switch pane "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Load logs "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" Refresh "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Back"),
    ]));
    f.render_widget(keys, outer_chunks[2]);
}

// ─── Actions Key Handlers ────────────────────────────────

fn handle_actions_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.github_state.view = GitHubView::Menu;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.github_state.actions_state.selected > 0 {
                app.github_state.actions_state.selected -= 1;
                let sel = app.github_state.actions_state.selected;
                app.github_state.actions_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.github_state.actions_state.runs.is_empty()
                && app.github_state.actions_state.selected + 1
                    < app.github_state.actions_state.runs.len()
            {
                app.github_state.actions_state.selected += 1;
                let sel = app.github_state.actions_state.selected;
                app.github_state.actions_state.list_state.select(Some(sel));
            }
        }
        KeyCode::Enter => {
            if let Some(run) = app
                .github_state
                .actions_state
                .runs
                .get(app.github_state.actions_state.selected)
            {
                let run_id = run.id;
                app.github_state.view = GitHubView::ActionDetail(run_id);
                app.github_state.actions_state.jobs.clear();
                app.github_state.actions_state.job_logs = None;
                app.github_state.actions_state.selected_job = 0;
                app.github_state.actions_state.log_scroll = 0;
                app.github_state.actions_state.detail_pane = ActionDetailPane::Jobs;
                start_load_action_detail(app, run_id);
            }
        }
        KeyCode::Char('r') => {
            start_load_actions(app);
        }
        _ => {}
    }
    Ok(())
}

fn handle_action_detail_key(app: &mut crate::app::App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.github_state.view = GitHubView::Actions;
            app.github_state.actions_state.job_logs = None;
        }
        KeyCode::Tab => {
            app.github_state.actions_state.detail_pane =
                match app.github_state.actions_state.detail_pane {
                    ActionDetailPane::Jobs => ActionDetailPane::Logs,
                    ActionDetailPane::Logs => ActionDetailPane::Jobs,
                };
        }
        KeyCode::Up | KeyCode::Char('k') => match app.github_state.actions_state.detail_pane {
            ActionDetailPane::Jobs => {
                if app.github_state.actions_state.selected_job > 0 {
                    app.github_state.actions_state.selected_job -= 1;
                    let sel = app.github_state.actions_state.selected_job;
                    app.github_state
                        .actions_state
                        .job_list_state
                        .select(Some(sel));
                }
            }
            ActionDetailPane::Logs => {
                if app.github_state.actions_state.log_scroll > 0 {
                    app.github_state.actions_state.log_scroll =
                        app.github_state.actions_state.log_scroll.saturating_sub(3);
                }
            }
        },
        KeyCode::Down | KeyCode::Char('j') => match app.github_state.actions_state.detail_pane {
            ActionDetailPane::Jobs => {
                if !app.github_state.actions_state.jobs.is_empty()
                    && app.github_state.actions_state.selected_job + 1
                        < app.github_state.actions_state.jobs.len()
                {
                    app.github_state.actions_state.selected_job += 1;
                    let sel = app.github_state.actions_state.selected_job;
                    app.github_state
                        .actions_state
                        .job_list_state
                        .select(Some(sel));
                }
            }
            ActionDetailPane::Logs => {
                app.github_state.actions_state.log_scroll += 3;
            }
        },
        KeyCode::Enter => {
            // Load logs for the selected job
            if let Some(job) = app
                .github_state
                .actions_state
                .jobs
                .get(app.github_state.actions_state.selected_job)
            {
                let job_id = job.id;
                app.github_state.actions_state.log_scroll = 0;
                start_load_job_logs(app, job_id);
            }
        }
        KeyCode::Char('r') => {
            // Refresh detail
            if let GitHubView::ActionDetail(run_id) = app.github_state.view {
                start_load_action_detail(app, run_id);
            }
        }
        _ => {}
    }
    Ok(())
}

// ─── Actions Background Loading ────────────────────────────────

fn start_load_actions(app: &mut crate::app::App) {
    app.github_state.actions_state.loading = true;
    app.github_state.actions_state.error = None;
    let token = app.config.github.get_token().unwrap_or_default();
    let bg = app.github_state.actions_state.bg_result.clone();
    std::thread::spawn(move || {
        let result = git::github_auth::list_workflow_runs(&token)
            .map(|r| r.workflow_runs)
            .map_err(|e| e.to_string());
        if let Ok(mut r) = bg.lock() {
            *r = Some(ActionsBgResult::RunsList(result));
        }
    });
}

fn start_load_action_detail(app: &mut crate::app::App, run_id: u64) {
    app.github_state.actions_state.loading = true;
    app.github_state.actions_state.error = None;
    let token = app.config.github.get_token().unwrap_or_default();
    let bg = app.github_state.actions_state.bg_result.clone();
    std::thread::spawn(move || {
        let result = git::github_auth::list_run_jobs(&token, run_id)
            .map(|r| r.jobs)
            .map_err(|e| e.to_string());
        if let Ok(mut r) = bg.lock() {
            *r = Some(ActionsBgResult::RunDetail(result));
        }
    });
}

fn start_load_job_logs(app: &mut crate::app::App, job_id: u64) {
    app.github_state.actions_state.loading = true;
    let token = app.config.github.get_token().unwrap_or_default();
    let bg = app.github_state.actions_state.bg_result.clone();
    std::thread::spawn(move || {
        let result = git::github_auth::get_job_logs(&token, job_id).map_err(|e| e.to_string());
        if let Ok(mut r) = bg.lock() {
            *r = Some(ActionsBgResult::JobLogs(job_id, result));
        }
    });
}

/// Called on each tick to poll for Actions background results.
pub fn tick_actions_state(app: &mut crate::app::App) {
    let bg_taken = {
        if let Ok(mut result) = app.github_state.actions_state.bg_result.try_lock() {
            result.take()
        } else {
            None
        }
    };

    if let Some(bg) = bg_taken {
        app.github_state.actions_state.loading = false;
        match bg {
            ActionsBgResult::RunsList(Ok(runs)) => {
                app.github_state.actions_state.runs = runs;
                app.github_state.actions_state.selected = 0;
                app.github_state.actions_state.list_state.select(
                    if app.github_state.actions_state.runs.is_empty() {
                        None
                    } else {
                        Some(0)
                    },
                );
                app.github_state.actions_state.error = None;
            }
            ActionsBgResult::RunsList(Err(e)) => {
                app.github_state.actions_state.error = Some(e);
            }
            ActionsBgResult::RunDetail(Ok(jobs)) => {
                app.github_state.actions_state.jobs = jobs;
                app.github_state.actions_state.selected_job = 0;
                app.github_state.actions_state.job_list_state.select(
                    if app.github_state.actions_state.jobs.is_empty() {
                        None
                    } else {
                        Some(0)
                    },
                );
            }
            ActionsBgResult::RunDetail(Err(e)) => {
                app.github_state.actions_state.error = Some(format!("Jobs: {}", e));
            }
            ActionsBgResult::JobLogs(_job_id, Ok(logs)) => {
                app.github_state.actions_state.job_logs = Some(logs);
                app.github_state.actions_state.log_scroll = 0;
                // Switch focus to logs pane
                app.github_state.actions_state.detail_pane = ActionDetailPane::Logs;
            }
            ActionsBgResult::JobLogs(_job_id, Err(e)) => {
                app.github_state.actions_state.job_logs =
                    Some(format!("Failed to load logs: {}", e));
            }
        }
    }
}
