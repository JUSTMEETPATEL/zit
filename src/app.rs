use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::sync::mpsc;

use crate::ai::client::AiClient;
use crate::config::Config;
use crate::git;
use crate::ui::{
    ai_mentor, branches, commit, dashboard, github, merge_resolve, reflog, staging, stash,
    time_travel, timeline,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Dashboard,
    Staging,
    Commit,
    Branches,
    Timeline,
    TimeTravel,
    Reflog,
    GitHub,
    AiMentor,
    Stash,
    MergeResolve,
}

/// Popup dialog state.
#[derive(Debug, Clone)]
pub enum Popup {
    None,
    Help,
    Confirm {
        title: String,
        message: String,
        on_confirm: ConfirmAction,
    },
    Input {
        title: String,
        prompt: String,
        value: String,
        on_submit: InputAction,
    },
    Message {
        title: String,
        message: String,
    },
    FollowUp {
        title: String,
        #[allow(dead_code)]
        context: String,
        suggestions: Vec<FollowUpItem>,
        selected: usize,
    },
}

/// A follow-up suggestion item shown after AI responses.
#[derive(Debug, Clone)]
pub struct FollowUpItem {
    pub label: String,
    pub description: String,
    pub action: FollowUpAction,
}

/// Actions that can be triggered from follow-up suggestions.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FollowUpAction {
    ApplyResolution(String), // file path
    StageFile(String),       // file path
    CommitNow,
    AbortMerge,
    ContinueMerge,
    ViewNextConflict,
    AskAiMore(String), // context/question
    SwitchToView(View),
    RunGitCommand(Vec<String>), // args for git
    EditCommitMessage,
    RegenerateAiSuggestion,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteBranch(String),
    HardReset(String),
    MixedReset(String),
    SoftReset(String),
    RemoveCollaborator(String),
    ClearStash,
    AbortMerge,
    ContinueMerge,
    MergePullRequest { number: u64, method: String },
    ClosePullRequest(u64),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputAction {
    CreateBranch,
    RenameBranch,
    SearchCommits,
    SearchFiles,
    CommitMessage,
    AddCollaborator,
    AiSetupEndpoint,
    AiSetupApiKey,
    StashPush,
}

/// Describes which AI action is in flight.
#[derive(Debug, Clone)]
pub enum AiAction {
    CommitSuggest,
    ExplainRepo,
    ExplainError(String),
    Recommend,
    HealthCheck,
    ReviewDiff(String), // file path being reviewed
    AskQuestion,
    Learn,
    MergeResolve(String), // file path being resolved
    MergeStrategy,
    ResetSuggest,
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub popup: Popup,
    pub config: Config,
    pub status_message: Option<String>,
    pub ai_client: Option<AiClient>,
    pub ai_loading: bool,
    ai_receiver: Option<mpsc::Receiver<Result<String, String>>>,
    ai_action: Option<AiAction>,
    /// Temporary storage for AI setup wizard (endpoint from step 1).
    ai_setup_endpoint: Option<String>,

    // View states
    pub dashboard_state: dashboard::DashboardState,
    pub staging_state: staging::StagingState,
    pub commit_state: commit::CommitState,
    pub branches_state: branches::BranchesState,
    pub timeline_state: timeline::TimelineState,
    pub time_travel_state: time_travel::TimeTravelState,
    pub reflog_state: reflog::ReflogState,
    pub github_state: github::GitHubState,
    pub ai_mentor_state: ai_mentor::AiMentorState,
    pub stash_state: stash::StashState,
    pub merge_resolve_state: merge_resolve::MergeResolveState,
}

impl App {
    pub fn new(config: Config) -> Self {
        // Validate AI config and warn about issues
        let ai_issues = config.ai.validate();
        let ai_client = AiClient::from_config(&config.ai);
        let status_message = if !ai_issues.is_empty() {
            Some(format!("⚠ AI config: {}", ai_issues[0]))
        } else if config.ai.is_ready() && ai_client.is_some() {
            Some("✓ AI Mentor ready".to_string())
        } else {
            None
        };
        Self {
            running: true,
            view: View::Dashboard,
            popup: Popup::None,
            config,
            status_message,
            ai_client,
            ai_loading: false,
            ai_receiver: None,
            ai_action: None,
            ai_setup_endpoint: None,
            dashboard_state: dashboard::DashboardState::default(),
            staging_state: staging::StagingState::default(),
            commit_state: commit::CommitState::default(),
            branches_state: branches::BranchesState::default(),
            timeline_state: timeline::TimelineState::default(),
            time_travel_state: time_travel::TimeTravelState::default(),
            reflog_state: reflog::ReflogState::default(),
            github_state: github::GitHubState::new(),
            ai_mentor_state: ai_mentor::AiMentorState::default(),
            stash_state: stash::StashState::default(),
            merge_resolve_state: merge_resolve::MergeResolveState::default(),
        }
    }

    /// Refresh data for the current view.
    pub fn refresh(&mut self) {
        match self.view {
            View::Dashboard => self.dashboard_state.refresh(),
            View::Staging => self.staging_state.refresh(),
            View::Commit => self.commit_state.refresh(),
            View::Branches => self.branches_state.refresh(),
            View::Timeline => self.timeline_state.refresh(),
            View::TimeTravel => self.time_travel_state.refresh(),
            View::Reflog => self.reflog_state.refresh(),
            View::GitHub => {}   // no auto-refresh for GitHub
            View::AiMentor => {} // no auto-refresh
            View::Stash => self.stash_state.refresh(),
            View::MergeResolve => self.merge_resolve_state.refresh(),
        }
    }

    /// Handle a key event. Returns Ok(()) or an error.
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle popup first
        match &self.popup {
            Popup::Help => {
                if key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('?')
                    || key.code == KeyCode::Char('q')
                {
                    self.popup = Popup::None;
                }
                return Ok(());
            }
            Popup::Confirm { on_confirm, .. } => {
                let action = on_confirm.clone();
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.popup = Popup::None;
                        self.execute_confirm(action)?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.popup = Popup::None;
                    }
                    _ => {}
                }
                return Ok(());
            }
            Popup::Input {
                value, on_submit, ..
            } => {
                let mut val = value.clone();
                let action = on_submit.clone();
                match key.code {
                    KeyCode::Char(c) => {
                        val.push(c);
                        if let Popup::Input { ref mut value, .. } = self.popup {
                            *value = val;
                        }
                    }
                    KeyCode::Backspace => {
                        val.pop();
                        if let Popup::Input { ref mut value, .. } = self.popup {
                            *value = val;
                        }
                    }
                    KeyCode::Enter => {
                        self.popup = Popup::None;
                        self.execute_input(action, val)?;
                    }
                    KeyCode::Esc => {
                        self.popup = Popup::None;
                    }
                    _ => {}
                }
                return Ok(());
            }
            Popup::Message { .. } => {
                if key.code == KeyCode::Esc
                    || key.code == KeyCode::Enter
                    || key.code == KeyCode::Char('q')
                {
                    self.popup = Popup::None;
                }
                return Ok(());
            }
            Popup::FollowUp {
                suggestions,
                selected,
                ..
            } => {
                let suggestions = suggestions.clone();
                let sel = *selected;
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.popup = Popup::None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Popup::FollowUp {
                            ref mut selected, ..
                        } = self.popup
                        {
                            if *selected > 0 {
                                *selected -= 1;
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Popup::FollowUp {
                            ref mut selected,
                            ref suggestions,
                            ..
                        } = self.popup
                        {
                            if *selected + 1 < suggestions.len() {
                                *selected += 1;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(item) = suggestions.get(sel) {
                            let action = item.action.clone();
                            self.popup = Popup::None;
                            self.execute_follow_up(action);
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let idx = (c as usize) - ('1' as usize);
                        if idx < suggestions.len() {
                            let action = suggestions[idx].action.clone();
                            self.popup = Popup::None;
                            self.execute_follow_up(action);
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }
            Popup::None => {}
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                if self.view == View::Dashboard {
                    self.running = false;
                } else {
                    self.view = View::Dashboard;
                    self.dashboard_state.refresh();
                    self.clear_status();
                }
                return Ok(());
            }
            KeyCode::Char('?') => {
                self.popup = Popup::Help;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
                return Ok(());
            }
            _ => {}
        }

        // Navigation from Dashboard
        if self.view == View::Dashboard {
            match key.code {
                KeyCode::Char('s') => {
                    self.view = View::Staging;
                    self.staging_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    self.view = View::Commit;
                    self.commit_state.refresh();
                    self.auto_suggest_if_ready();
                    return Ok(());
                }
                KeyCode::Char('b') => {
                    self.view = View::Branches;
                    self.branches_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('l') => {
                    self.view = View::Timeline;
                    self.timeline_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('t') => {
                    self.view = View::TimeTravel;
                    self.time_travel_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('r') => {
                    self.view = View::Reflog;
                    self.reflog_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('g') => {
                    self.view = View::GitHub;
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    self.view = View::AiMentor;
                    return Ok(());
                }
                KeyCode::Char('x') => {
                    self.view = View::Stash;
                    self.stash_state.refresh();
                    return Ok(());
                }
                KeyCode::Char('m') => {
                    // Open merge resolve view (only useful when conflicts exist)
                    self.view = View::MergeResolve;
                    self.merge_resolve_state.refresh();
                    return Ok(());
                }
                _ => {}
            }
        }

        // Delegate to view-specific handler
        match self.view {
            View::Dashboard => dashboard::handle_key(self, key)?,
            View::Staging => staging::handle_key(self, key)?,
            View::Commit => commit::handle_key(self, key)?,
            View::Branches => branches::handle_key(self, key)?,
            View::Timeline => timeline::handle_key(self, key)?,
            View::TimeTravel => time_travel::handle_key(self, key)?,
            View::Reflog => reflog::handle_key(self, key)?,
            View::GitHub => github::handle_key(self, key)?,
            View::AiMentor => ai_mentor::handle_key(self, key)?,
            View::Stash => stash::handle_key(self, key)?,
            View::MergeResolve => merge_resolve::handle_key(self, key)?,
        }

        Ok(())
    }

    fn execute_confirm(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteBranch(name) => {
                match git::BranchOps::delete(&name, false) {
                    Ok(()) => self.status_message = Some(format!("Deleted branch '{}'", name)),
                    Err(e) => {
                        let err_str = e.to_string();
                        self.status_message = Some(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.branches_state.refresh();
            }
            ConfirmAction::HardReset(hash) => {
                match git::run_git(&["reset", "--hard", &hash]) {
                    Ok(_) => {
                        self.status_message =
                            Some(format!("Hard reset to {}", &hash[..7.min(hash.len())]))
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        self.status_message = Some(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::MixedReset(hash) => {
                match git::run_git(&["reset", "--mixed", &hash]) {
                    Ok(_) => {
                        self.status_message =
                            Some(format!("Mixed reset to {}", &hash[..7.min(hash.len())]))
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        self.status_message = Some(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::SoftReset(hash) => {
                match git::run_git(&["reset", "--soft", &hash]) {
                    Ok(_) => {
                        self.status_message =
                            Some(format!("Soft reset to {}", &hash[..7.min(hash.len())]))
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        self.status_message = Some(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::RemoveCollaborator(username) => {
                if let Some(token) = self.config.github.get_token() {
                    match git::github_auth::remove_collaborator(&token, &username) {
                        Ok(()) => {
                            self.github_state.collab_error =
                                Some(format!("✓ Removed @{}", username));
                            // Refresh collaborator list
                            if let Ok(collabs) = git::github_auth::list_collaborators(&token) {
                                self.github_state.collaborators = collabs;
                                self.github_state.collab_selected = 0;
                                self.github_state.collab_list_state.select(
                                    if self.github_state.collaborators.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            self.github_state.collab_error = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            ConfirmAction::ClearStash => {
                match git::stash::stash_clear() {
                    Ok(_) => self.status_message = Some("Cleared all stash entries".to_string()),
                    Err(e) => {
                        let err_str = e.to_string();
                        self.status_message = Some(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.stash_state.refresh();
            }
            ConfirmAction::AbortMerge => match git::merge::abort_merge() {
                Ok(()) => {
                    self.set_status("Merge aborted successfully");
                    self.view = View::Dashboard;
                    self.dashboard_state.refresh();
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.set_status(format!("Error aborting merge: {}", err_str));
                    self.start_ai_error_explain(err_str);
                }
            },
            ConfirmAction::ContinueMerge => match git::merge::continue_merge() {
                Ok(()) => {
                    self.set_status("Merge completed successfully!");
                    self.view = View::Dashboard;
                    self.dashboard_state.refresh();
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.set_status(format!("Error continuing merge: {}", err_str));
                    self.start_ai_error_explain(err_str);
                }
            },
            ConfirmAction::MergePullRequest { number, method } => {
                if let Some(token) = self.config.github.get_token() {
                    self.github_state.pr_state.loading = true;
                    let bg = self.github_state.pr_state.bg_result.clone();
                    std::thread::spawn(move || {
                        let result = git::github_auth::merge_pull_request(&token, number, &method)
                            .map_err(|e| e.to_string());
                        if let Ok(mut r) = bg.lock() {
                            *r = Some(github::PrBgResult::MergeResult(result));
                        }
                    });
                }
            }
            ConfirmAction::ClosePullRequest(number) => {
                if let Some(token) = self.config.github.get_token() {
                    self.github_state.pr_state.loading = true;
                    let bg = self.github_state.pr_state.bg_result.clone();
                    std::thread::spawn(move || {
                        let result = git::github_auth::close_pull_request(&token, number)
                            .map_err(|e| e.to_string());
                        if let Ok(mut r) = bg.lock() {
                            *r = Some(github::PrBgResult::CloseResult(result));
                        }
                    });
                }
            }
        }
        Ok(())
    }

    fn execute_input(&mut self, action: InputAction, value: String) -> Result<()> {
        // AI setup steps handle empty values themselves
        if value.trim().is_empty()
            && !matches!(
                action,
                InputAction::AiSetupEndpoint | InputAction::AiSetupApiKey | InputAction::StashPush
            )
        {
            return Ok(());
        }

        match action {
            InputAction::CreateBranch => {
                match git::BranchOps::create(value.trim(), None) {
                    Ok(()) => {
                        self.status_message = Some(format!("Created branch '{}'", value.trim()))
                    }
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.branches_state.refresh();
            }
            InputAction::RenameBranch => {
                match git::BranchOps::rename(value.trim()) {
                    Ok(()) => self.status_message = Some(format!("Renamed to '{}'", value.trim())),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.branches_state.refresh();
            }
            InputAction::SearchCommits => {
                self.timeline_state.search_query = value;
                self.timeline_state.do_search();
            }
            InputAction::SearchFiles => {
                self.staging_state.filter = value;
            }
            InputAction::CommitMessage => {
                // Handled in commit view
            }
            InputAction::AddCollaborator => {
                let username = value.trim().to_string();
                if let Some(token) = self.config.github.get_token() {
                    match git::github_auth::add_collaborator(&token, &username) {
                        Ok(msg) => {
                            self.github_state.collab_error = Some(format!("✓ {}", msg));
                            // Refresh collaborator list
                            if let Ok(collabs) = git::github_auth::list_collaborators(&token) {
                                self.github_state.collaborators = collabs;
                                self.github_state.collab_selected = 0;
                                self.github_state.collab_list_state.select(
                                    if self.github_state.collaborators.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            self.github_state.collab_error = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            InputAction::AiSetupEndpoint => {
                // Step 1 complete — store endpoint, ask for API key
                let endpoint = value.trim().to_string();
                if endpoint.is_empty() {
                    self.set_status("AI setup cancelled — endpoint is required");
                    return Ok(());
                }
                self.ai_setup_endpoint = Some(endpoint);
                self.popup = Popup::Input {
                    title: "🤖 AI Setup (2/2)".to_string(),
                    prompt: "API Key: ".to_string(),
                    value: self.config.ai.resolved_api_key().unwrap_or_default(),
                    on_submit: InputAction::AiSetupApiKey,
                };
            }
            InputAction::AiSetupApiKey => {
                // Step 2 complete — test + save
                let api_key = value.trim().to_string();
                let endpoint = self.ai_setup_endpoint.take().unwrap_or_default();
                if api_key.is_empty() || endpoint.is_empty() {
                    self.set_status("AI setup cancelled — endpoint and API key are required");
                    return Ok(());
                }
                // Update config in memory
                self.config.ai.enabled = true;
                self.config.ai.endpoint = Some(endpoint);
                self.config.ai.api_key = Some(api_key);
                // Save to disk
                match self.config.save() {
                    Ok(()) => {}
                    Err(e) => self.set_status(format!("⚠ Config in memory but save failed: {}", e)),
                }
                // Recreate AI client
                self.ai_client = AiClient::from_config(&self.config.ai);
                if self.ai_client.is_some() {
                    self.set_status("✓ AI configured! Testing connection...");
                    self.start_ai_query("health_check".to_string(), None);
                } else {
                    self.set_status("AI setup failed — could not create client");
                }
            }
            InputAction::StashPush => {
                let msg = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim())
                };
                match git::stash::stash_push(msg) {
                    Ok(_) => {
                        let label = msg.unwrap_or("(default)");
                        self.set_status(format!("Stashed changes: {}", label));
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        self.set_status(format!("Stash failed: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
                self.stash_state.refresh();
            }
        }
        Ok(())
    }

    /// Launch the interactive AI setup wizard (2-step: endpoint → API key).
    pub fn start_ai_setup(&mut self) {
        self.popup = Popup::Input {
            title: "🤖 AI Setup (1/2)".to_string(),
            prompt: "API Endpoint URL: ".to_string(),
            value: self.config.ai.resolved_endpoint().unwrap_or_default(),
            on_submit: InputAction::AiSetupEndpoint,
        };
    }

    /// Auto-suggest commit message if AI is available, message is empty, and files are staged.
    pub fn auto_suggest_if_ready(&mut self) {
        if self.ai_client.is_some()
            && self.commit_state.message.is_empty()
            && !self.commit_state.staged_files.is_empty()
            && !self.ai_loading
        {
            self.start_ai_suggest();
        }
    }

    /// Start an async AI commit message suggestion (non-blocking).
    pub fn start_ai_suggest(&mut self) {
        if self.ai_loading {
            self.set_status("⏳ AI is already generating...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured. Set [ai] in ~/.config/zit/config.toml or export ZIT_AI_API_KEY + ZIT_AI_ENDPOINT");
                return;
            }
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::CommitSuggest);
        self.set_status("⏳ Generating AI commit message...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client.suggest_commit_message().map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI query (explain_repo, recommend, health_check) — non-blocking.
    pub fn start_ai_query(&mut self, action_type: String, query: Option<String>) {
        if self.ai_loading {
            self.set_status("⏳ AI is already generating...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured. Set [ai] in ~/.config/zit/config.toml or export ZIT_AI_API_KEY + ZIT_AI_ENDPOINT");
                return;
            }
        };

        let action = match action_type.as_str() {
            "explain_repo" => AiAction::ExplainRepo,
            "recommend" => AiAction::Recommend,
            "health_check" => AiAction::HealthCheck,
            _ => AiAction::ExplainRepo,
        };

        self.ai_loading = true;
        self.ai_action = Some(action.clone());
        self.set_status("⏳ Asking AI mentor...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);
        let query_clone = query;

        std::thread::spawn(move || {
            let result = match action {
                AiAction::ExplainRepo => client
                    .explain_repo(query_clone.as_deref())
                    .map_err(|e| e.to_string()),
                AiAction::Recommend => {
                    let q = query_clone.unwrap_or_else(|| "What should I do next?".to_string());
                    client.recommend(&q).map_err(|e| e.to_string())
                }
                AiAction::HealthCheck => client.health_check().map_err(|e| e.to_string()),
                _ => Err("Unknown action".to_string()),
            };
            let _ = tx.send(result);
        });
    }

    /// Start an async AI error explanation — non-blocking.
    pub fn start_ai_error_explain(&mut self, error_msg: String) {
        if self.ai_loading {
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => return,
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::ExplainError(error_msg.clone()));
        self.set_status("⏳ AI is analyzing the error...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client.explain_error(&error_msg).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI diff review for a specific file — non-blocking.
    pub fn start_ai_diff_review(&mut self, file_path: String, diff_content: String) {
        if self.ai_loading {
            self.set_status("⏳ AI is already working...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured — press 'a' to open AI Mentor and set up");
                return;
            }
        };

        if diff_content.trim().is_empty() {
            self.set_status("No diff content to review");
            return;
        }

        self.ai_loading = true;
        self.ai_action = Some(AiAction::ReviewDiff(file_path.clone()));
        self.set_status(format!("⏳ AI reviewing diff for {}...", file_path));

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client
                .review_diff(&file_path, &diff_content)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI free-form question — non-blocking.
    pub fn start_ai_ask(&mut self, question: String) {
        if self.ai_loading {
            self.set_status("⏳ AI is already working...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured");
                return;
            }
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::AskQuestion);
        self.set_status("⏳ Asking AI mentor...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client.ask(&question).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI learn query — non-blocking.
    pub fn start_ai_learn(&mut self, topic: String) {
        if self.ai_loading {
            self.set_status("⏳ AI is already working...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured");
                return;
            }
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::Learn);
        self.set_status("⏳ AI is teaching...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client.learn(&topic).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI merge conflict resolution — non-blocking.
    pub fn start_ai_merge_resolve(&mut self, file_path: String, conflict_content: String) {
        if self.ai_loading {
            self.set_status("⏳ AI is already working...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured — press 'a' to open AI Mentor and set up");
                return;
            }
        };

        if conflict_content.trim().is_empty() {
            self.set_status("No conflict content to analyze");
            return;
        }

        self.ai_loading = true;
        self.ai_action = Some(AiAction::MergeResolve(file_path.clone()));
        self.set_status(format!("⏳ AI analyzing conflict in {}...", file_path));

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client
                .suggest_merge_resolution(&file_path, &conflict_content)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI merge strategy recommendation — non-blocking.
    pub fn start_ai_merge_strategy(&mut self, query: Option<String>) {
        if self.ai_loading {
            self.set_status("⏳ AI is already working...");
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.set_status("AI not configured — press 'a' to open AI Mentor and set up");
                return;
            }
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::MergeStrategy);
        self.set_status("⏳ AI analyzing merge strategy...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client
                .suggest_merge_strategy(query.as_deref())
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Start an async AI reset suggestion — non-blocking.
    pub fn start_ai_reset_suggest(
        &mut self,
        current_hash: String,
        target_hash: String,
        target_msg: String,
        commits_back: usize,
    ) {
        if self.ai_loading {
            self.time_travel_state.ai_suggestion = Some(
                "⏳ AI is already working on another request...\n\nPlease wait for it to finish, then try again."
                    .to_string(),
            );
            return;
        }
        let client = match self.ai_client {
            Some(ref c) => c.clone(),
            None => {
                self.time_travel_state.ai_suggestion = Some(
                    "⚠ AI is not configured.\n\n\
                     To set up AI Mentor:\n\
                     1. Press [Esc] to dismiss this panel\n\
                     2. Press [q] to go back to Dashboard\n\
                     3. Press [a] to open AI Mentor\n\
                     4. Follow the setup wizard to add your API endpoint and key\n\n\
                     Once configured, come back here and press [i] again!"
                        .to_string(),
                );
                self.time_travel_state.ai_loading = false;
                return;
            }
        };

        self.ai_loading = true;
        self.ai_action = Some(AiAction::ResetSuggest);
        self.time_travel_state.ai_loading = true;
        self.time_travel_state.ai_suggestion = None;
        self.set_status("⏳ AI analyzing reset options...");

        let (tx, rx) = mpsc::channel();
        self.ai_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = client
                .suggest_reset(&current_hash, &target_hash, &target_msg, commits_back)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    /// Execute a follow-up action from the suggestion list.
    pub fn execute_follow_up(&mut self, action: FollowUpAction) {
        match action {
            FollowUpAction::ApplyResolution(path) => {
                if let Some(ref content) = self.merge_resolve_state.ai_resolved_content.clone() {
                    match git::merge::resolve_file(&path, content) {
                        Ok(()) => {
                            self.set_status(format!("✓ Resolved and staged: {}", path));
                            self.merge_resolve_state.refresh();
                        }
                        Err(e) => {
                            self.set_status(format!("Error resolving: {}", e));
                        }
                    }
                }
            }
            FollowUpAction::StageFile(path) => match git::run_git(&["add", &path]) {
                Ok(_) => self.set_status(format!("✓ Staged: {}", path)),
                Err(e) => self.set_status(format!("Error staging: {}", e)),
            },
            FollowUpAction::CommitNow => {
                self.view = View::Commit;
                self.commit_state.refresh();
                self.auto_suggest_if_ready();
            }
            FollowUpAction::AbortMerge => {
                self.popup = Popup::Confirm {
                    title: "⚠ Abort Merge".to_string(),
                    message: "This will discard all merge progress. Continue? (y/n)".to_string(),
                    on_confirm: ConfirmAction::AbortMerge,
                };
            }
            FollowUpAction::ContinueMerge => {
                self.popup = Popup::Confirm {
                    title: "Continue Merge".to_string(),
                    message: "All conflicts resolved. Continue merge? (y/n)".to_string(),
                    on_confirm: ConfirmAction::ContinueMerge,
                };
            }
            FollowUpAction::ViewNextConflict => {
                if self.merge_resolve_state.selected_file + 1
                    < self.merge_resolve_state.conflicted_files.len()
                {
                    self.merge_resolve_state.selected_file += 1;
                    self.merge_resolve_state.load_selected_file();
                }
            }
            FollowUpAction::AskAiMore(question) => {
                self.start_ai_ask(question);
            }
            FollowUpAction::SwitchToView(view) => {
                self.view = view;
                self.refresh();
            }
            FollowUpAction::RunGitCommand(args) => {
                let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                match git::run_git(&args_str) {
                    Ok(output) => self.set_status(format!("✓ {}", output.trim())),
                    Err(e) => {
                        let err_str = e.to_string();
                        self.set_status(format!("Error: {}", err_str));
                        self.start_ai_error_explain(err_str);
                    }
                }
            }
            FollowUpAction::EditCommitMessage => {
                self.view = View::Commit;
                self.commit_state.refresh();
            }
            FollowUpAction::RegenerateAiSuggestion => {
                if self.view == View::Commit {
                    self.start_ai_suggest();
                } else if self.view == View::MergeResolve {
                    let state = &self.merge_resolve_state;
                    if let Some(content) = state.raw_conflict_content.clone() {
                        let path = state
                            .conflicted_files
                            .get(state.selected_file)
                            .map(|f| f.path.clone())
                            .unwrap_or_default();
                        self.start_ai_merge_resolve(path, content);
                    }
                }
            }
        }
    }

    /// Poll for an AI result (non-blocking). Call on every tick/key event.
    pub fn poll_ai_result(&mut self) {
        if let Some(ref rx) = self.ai_receiver {
            match rx.try_recv() {
                Ok(Ok(response)) => {
                    let action = self.ai_action.take();
                    self.ai_loading = false;
                    self.ai_receiver = None;

                    match action {
                        Some(AiAction::CommitSuggest) => {
                            self.commit_state.message = response.trim().to_string();
                            self.commit_state.validate();
                            self.set_status(
                                "✓ AI suggestion loaded — edit or press Enter to commit",
                            );
                            // Store in history
                            self.ai_mentor_state.add_history(
                                "Commit Suggestion".to_string(),
                                response.trim().to_string(),
                            );
                        }
                        Some(AiAction::ExplainError(original_err)) => {
                            let msg = format!(
                                "Error: {}\n\n── AI Explanation ──\n\n{}",
                                original_err, response
                            );
                            self.popup = Popup::Message {
                                title: "🤖 AI Error Explanation".to_string(),
                                message: msg,
                            };
                            self.set_status("✓ AI explanation ready");
                            // Store in history
                            self.ai_mentor_state.add_history(
                                format!("Error: {}", &original_err[..original_err.len().min(50)]),
                                response.clone(),
                            );
                        }
                        Some(AiAction::ReviewDiff(file_path)) => {
                            let msg =
                                format!("── AI Diff Review: {} ──\n\n{}", file_path, response);
                            self.popup = Popup::Message {
                                title: "🤖 AI Diff Review".to_string(),
                                message: msg,
                            };
                            self.set_status("✓ AI diff review ready");
                            // Store in history
                            self.ai_mentor_state
                                .add_history(format!("Review: {}", file_path), response.clone());
                        }
                        Some(AiAction::AskQuestion) => {
                            self.ai_mentor_state.result_text = response.clone();
                            self.ai_mentor_state.result_scroll = 0;
                            self.ai_mentor_state.mode = ai_mentor::AiMode::Result;
                            self.set_status("✓ AI response ready");
                            // Store in history
                            let query = self.ai_mentor_state.input.clone();
                            self.ai_mentor_state.add_history(
                                if query.is_empty() {
                                    "Question".to_string()
                                } else {
                                    query
                                },
                                response,
                            );
                        }
                        Some(AiAction::ExplainRepo)
                        | Some(AiAction::Recommend)
                        | Some(AiAction::HealthCheck)
                        | Some(AiAction::Learn) => {
                            let label = match &action {
                                Some(AiAction::ExplainRepo) => "Explain Repo",
                                Some(AiAction::Recommend) => "Recommend",
                                Some(AiAction::HealthCheck) => "Health Check",
                                Some(AiAction::Learn) => "Learn",
                                _ => "AI Response",
                            };
                            self.ai_mentor_state.result_text = response.clone();
                            self.ai_mentor_state.result_scroll = 0;
                            self.ai_mentor_state.mode = ai_mentor::AiMode::Result;
                            self.set_status("✓ AI response ready");
                            // Store in history
                            self.ai_mentor_state
                                .add_history(label.to_string(), response);
                        }
                        Some(AiAction::MergeResolve(file_path)) => {
                            // Parse AI response and populate merge resolve state
                            self.merge_resolve_state.ai_suggestion = Some(response.clone());
                            self.merge_resolve_state.ai_resolved_content =
                                parse_ai_resolved_content(&response);
                            self.merge_resolve_state.ai_recommendation =
                                parse_ai_recommendation(&response);
                            self.set_status(format!("✓ AI resolution ready for {}", file_path));
                            // Generate follow-up suggestions
                            let follow_ups =
                                generate_merge_follow_ups(&file_path, &self.merge_resolve_state);
                            if !follow_ups.is_empty() {
                                self.merge_resolve_state.follow_ups = follow_ups;
                            }
                            // Store in history
                            self.ai_mentor_state
                                .add_history(format!("Merge Resolve: {}", file_path), response);
                        }
                        Some(AiAction::MergeStrategy) => {
                            // Show strategy recommendation as popup with follow-ups
                            let follow_ups = generate_strategy_follow_ups(&response);
                            if follow_ups.is_empty() {
                                self.popup = Popup::Message {
                                    title: "🤖 AI Merge Strategy".to_string(),
                                    message: response.clone(),
                                };
                            } else {
                                self.popup = Popup::FollowUp {
                                    title: "🤖 AI Merge Strategy".to_string(),
                                    context: response.clone(),
                                    suggestions: follow_ups,
                                    selected: 0,
                                };
                            }
                            self.set_status("✓ AI strategy recommendation ready");
                            // Store in history
                            self.ai_mentor_state
                                .add_history("Merge Strategy".to_string(), response);
                        }
                        Some(AiAction::ResetSuggest) => {
                            self.time_travel_state.ai_suggestion = Some(response.clone());
                            self.time_travel_state.ai_loading = false;
                            self.time_travel_state.ai_scroll = 0;
                            self.set_status("✓ AI reset insight ready — press Esc to dismiss");
                            // Store in history
                            self.ai_mentor_state
                                .add_history("Reset Insight".to_string(), response);
                        }
                        None => {
                            self.set_status(format!("AI: {}", response));
                        }
                    }
                }
                Ok(Err(e)) => {
                    self.set_status(format!("AI error: {}", e));
                    self.ai_loading = false;
                    self.ai_receiver = None;
                    self.ai_action = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still waiting — nothing to do
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.set_status("AI request was interrupted");
                    self.ai_loading = false;
                    self.ai_receiver = None;
                    self.ai_action = None;
                }
            }
        }
    }

    /// Set a status message that appears at the bottom.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Clear the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Handle mouse events (scroll wheel for list navigation).
    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => match self.view {
                View::Staging => {
                    let len = self.staging_state.files.len();
                    if len > 0 && self.staging_state.selected < len - 1 {
                        self.staging_state.selected += 1;
                        self.staging_state
                            .list_state
                            .select(Some(self.staging_state.selected));
                    }
                }
                View::Timeline => {
                    let len = self.timeline_state.commits.len();
                    if len > 0 && self.timeline_state.selected < len - 1 {
                        self.timeline_state.selected += 1;
                    }
                }
                View::Branches => {
                    let len = self.branches_state.branches.len();
                    if len > 0 && self.branches_state.selected < len - 1 {
                        self.branches_state.selected += 1;
                    }
                }
                View::Reflog => {
                    let len = self.reflog_state.entries.len();
                    if len > 0 && self.reflog_state.selected < len - 1 {
                        self.reflog_state.selected += 1;
                    }
                }
                View::Stash => {
                    let len = self.stash_state.entries.len();
                    if len > 0 && self.stash_state.selected < len - 1 {
                        self.stash_state.selected += 1;
                        self.stash_state
                            .list_state
                            .select(Some(self.stash_state.selected));
                    }
                }
                _ => {}
            },
            MouseEventKind::ScrollUp => match self.view {
                View::Staging => {
                    if self.staging_state.selected > 0 {
                        self.staging_state.selected -= 1;
                        self.staging_state
                            .list_state
                            .select(Some(self.staging_state.selected));
                    }
                }
                View::Timeline => {
                    if self.timeline_state.selected > 0 {
                        self.timeline_state.selected -= 1;
                    }
                }
                View::Branches => {
                    if self.branches_state.selected > 0 {
                        self.branches_state.selected -= 1;
                    }
                }
                View::Reflog => {
                    if self.reflog_state.selected > 0 {
                        self.reflog_state.selected -= 1;
                    }
                }
                View::Stash => {
                    if self.stash_state.selected > 0 {
                        self.stash_state.selected -= 1;
                        self.stash_state
                            .list_state
                            .select(Some(self.stash_state.selected));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

// ─── AI Response Parsing Helpers ───────────────────────────────

/// Parse the RECOMMENDATION line from an AI merge resolution response.
fn parse_ai_recommendation(response: &str) -> Option<String> {
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("RECOMMENDATION:") {
            return Some(
                trimmed
                    .strip_prefix("RECOMMENDATION:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }
        if trimmed.starts_with("RECOMMENDED:") {
            return Some(
                trimmed
                    .strip_prefix("RECOMMENDED:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }
    }
    None
}

/// Extract resolved content from code blocks in the AI response.
fn parse_ai_resolved_content(response: &str) -> Option<String> {
    // Look for RESOLVED CONTENT section followed by a code block
    let mut in_resolved_section = false;
    let mut in_code_block = false;
    let mut content = Vec::new();

    for line in response.lines() {
        if line.trim().starts_with("RESOLVED CONTENT") {
            in_resolved_section = true;
            continue;
        }
        if in_resolved_section && line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                break;
            } else {
                in_code_block = true;
                continue;
            }
        }
        if in_code_block {
            content.push(line);
        }
    }

    if content.is_empty() {
        None
    } else {
        Some(content.join("\n"))
    }
}

/// Generate follow-up suggestions after a merge conflict resolution AI response.
fn generate_merge_follow_ups(
    file_path: &str,
    state: &merge_resolve::MergeResolveState,
) -> Vec<FollowUpItem> {
    let mut items = Vec::new();

    // If AI provided resolved content, offer to apply it
    if state.ai_resolved_content.is_some() {
        items.push(FollowUpItem {
            label: "Apply AI resolution".to_string(),
            description: format!("Write resolved content and stage {}", file_path),
            action: FollowUpAction::ApplyResolution(file_path.to_string()),
        });
    }

    // Navigate to next conflict
    if state.selected_file + 1 < state.conflicted_files.len() {
        items.push(FollowUpItem {
            label: "Next conflicted file".to_string(),
            description: "Move to the next file with conflicts".to_string(),
            action: FollowUpAction::ViewNextConflict,
        });
    }

    // All conflicts resolved? Offer to continue merge
    if state.conflicted_files.len() <= 1 {
        items.push(FollowUpItem {
            label: "Continue merge".to_string(),
            description: "All conflicts resolved — finalize the merge".to_string(),
            action: FollowUpAction::ContinueMerge,
        });
    }

    // Always offer abort
    items.push(FollowUpItem {
        label: "Abort merge".to_string(),
        description: "Discard all merge progress and return to previous state".to_string(),
        action: FollowUpAction::AbortMerge,
    });

    // Ask AI for more explanation
    items.push(FollowUpItem {
        label: "Ask AI for more detail".to_string(),
        description: "Get a deeper explanation of why this resolution was suggested".to_string(),
        action: FollowUpAction::AskAiMore(format!(
            "Explain in more detail why {} should be resolved this way",
            file_path
        )),
    });

    items
}

/// Generate follow-up suggestions after a merge strategy AI response.
fn generate_strategy_follow_ups(response: &str) -> Vec<FollowUpItem> {
    let mut items = Vec::new();

    // Parse recommended strategy
    let rec = parse_ai_recommendation(response);
    let strategy = rec.as_deref().unwrap_or("");

    if strategy.contains("MERGE") || strategy.contains("merge") {
        items.push(FollowUpItem {
            label: "Run merge --no-ff".to_string(),
            description: "Execute the recommended merge strategy".to_string(),
            action: FollowUpAction::AskAiMore(
                "I chose to merge. What exact commands should I run and what should I watch for?"
                    .to_string(),
            ),
        });
    }
    if strategy.contains("REBASE") || strategy.contains("rebase") {
        items.push(FollowUpItem {
            label: "Run rebase".to_string(),
            description: "Execute the recommended rebase strategy".to_string(),
            action: FollowUpAction::AskAiMore(
                "I chose to rebase. Walk me through the exact steps and what to watch for."
                    .to_string(),
            ),
        });
    }
    if strategy.contains("FAST") || strategy.contains("fast") {
        items.push(FollowUpItem {
            label: "Fast-forward merge".to_string(),
            description: "Execute fast-forward merge".to_string(),
            action: FollowUpAction::AskAiMore(
                "I chose fast-forward. What exact command should I use?".to_string(),
            ),
        });
    }

    // Always offer conflict resolution view
    items.push(FollowUpItem {
        label: "View conflicts".to_string(),
        description: "Open the merge conflict resolution view".to_string(),
        action: FollowUpAction::SwitchToView(View::MergeResolve),
    });

    // Ask for alternatives
    items.push(FollowUpItem {
        label: "Ask for alternatives".to_string(),
        description: "Get more detail on alternative strategies".to_string(),
        action: FollowUpAction::AskAiMore(
            "What are the trade-offs between merge and rebase for my situation?".to_string(),
        ),
    });

    items
}
