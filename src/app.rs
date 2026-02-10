use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::git;
use crate::ui::{dashboard, staging, commit, branches, timeline, time_travel, reflog, github};


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
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteBranch(String),
    HardReset(String),
    MixedReset(String),
    SoftReset(String),
    RemoveCollaborator(String),
}

#[derive(Debug, Clone)]
pub enum InputAction {
    CreateBranch,
    RenameBranch,
    SearchCommits,
    SearchFiles,
    CommitMessage,
    CreateRepo,
    AddCollaborator,
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub popup: Popup,
    pub config: Config,
    pub status_message: Option<String>,

    // View states
    pub dashboard_state: dashboard::DashboardState,
    pub staging_state: staging::StagingState,
    pub commit_state: commit::CommitState,
    pub branches_state: branches::BranchesState,
    pub timeline_state: timeline::TimelineState,
    pub time_travel_state: time_travel::TimeTravelState,
    pub reflog_state: reflog::ReflogState,
    pub github_state: github::GitHubState,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            running: true,
            view: View::Dashboard,
            popup: Popup::None,
            config,
            status_message: None,
            dashboard_state: dashboard::DashboardState::default(),
            staging_state: staging::StagingState::default(),
            commit_state: commit::CommitState::default(),
            branches_state: branches::BranchesState::default(),
            timeline_state: timeline::TimelineState::default(),
            time_travel_state: time_travel::TimeTravelState::default(),
            reflog_state: reflog::ReflogState::default(),
            github_state: github::GitHubState::new(),
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
            View::GitHub => {} // no auto-refresh for GitHub
        }
    }

    /// Handle a key event. Returns Ok(()) or an error.
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle popup first
        match &self.popup {
            Popup::Help => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') || key.code == KeyCode::Char('q') {
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
            Popup::Input { value, on_submit, .. } => {
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
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter || key.code == KeyCode::Char('q') {
                    self.popup = Popup::None;
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
        }

        Ok(())
    }

    fn execute_confirm(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteBranch(name) => {
                match git::BranchOps::delete(&name, false) {
                    Ok(()) => self.status_message = Some(format!("Deleted branch '{}'", name)),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.branches_state.refresh();
            }
            ConfirmAction::HardReset(hash) => {
                match git::run_git(&["reset", "--hard", &hash]) {
                    Ok(_) => self.status_message = Some(format!("Hard reset to {}", &hash[..7.min(hash.len())])),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::MixedReset(hash) => {
                match git::run_git(&["reset", "--mixed", &hash]) {
                    Ok(_) => self.status_message = Some(format!("Mixed reset to {}", &hash[..7.min(hash.len())])),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::SoftReset(hash) => {
                match git::run_git(&["reset", "--soft", &hash]) {
                    Ok(_) => self.status_message = Some(format!("Soft reset to {}", &hash[..7.min(hash.len())])),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
                self.time_travel_state.refresh();
            }
            ConfirmAction::RemoveCollaborator(username) => {
                if let Some(token) = self.config.github.get_token().map(|t| t.to_string()) {
                    match git::github_auth::remove_collaborator(&token, &username) {
                        Ok(()) => {
                            self.github_state.collab_error = Some(format!("✓ Removed @{}", username));
                            // Refresh collaborator list
                            match git::github_auth::list_collaborators(&token) {
                                Ok(collabs) => {
                                    self.github_state.collaborators = collabs;
                                    self.github_state.collab_selected = 0;
                                    self.github_state.collab_list_state.select(
                                        if self.github_state.collaborators.is_empty() { None } else { Some(0) },
                                    );
                                }
                                Err(_) => {}
                            }
                        }
                        Err(e) => {
                            self.github_state.collab_error = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_input(&mut self, action: InputAction, value: String) -> Result<()> {
        if value.trim().is_empty() {
            return Ok(());
        }

        match action {
            InputAction::CreateBranch => {
                match git::BranchOps::create(value.trim(), None) {
                    Ok(()) => self.status_message = Some(format!("Created branch '{}'", value.trim())),
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
            InputAction::CreateRepo => {
                self.status_message = Some(format!("Create repo '{}' (placeholder)", value));
            }
            InputAction::AddCollaborator => {
                let username = value.trim().to_string();
                if let Some(token) = self.config.github.get_token().map(|t| t.to_string()) {
                    match git::github_auth::add_collaborator(&token, &username) {
                        Ok(msg) => {
                            self.github_state.collab_error = Some(format!("✓ {}", msg));
                            // Refresh collaborator list
                            match git::github_auth::list_collaborators(&token) {
                                Ok(collabs) => {
                                    self.github_state.collaborators = collabs;
                                    self.github_state.collab_selected = 0;
                                    self.github_state.collab_list_state.select(
                                        if self.github_state.collaborators.is_empty() { None } else { Some(0) },
                                    );
                                }
                                Err(_) => {}
                            }
                        }
                        Err(e) => {
                            self.github_state.collab_error = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Set a status message that appears at the bottom.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Clear the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
