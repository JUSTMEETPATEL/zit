use anyhow::{Context, Result};
use serde::Deserialize;

/// GitHub OAuth App Client ID for zit.
pub const CLIENT_ID: &str = "Ov23liMBOn6cAuIPFslq";

/// Scopes to request — repo access for push/pull/create, read:user for username.
const SCOPES: &str = "repo,read:user";

/// Response from POST /login/device/code
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Successful token response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

/// Error response during polling
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(default)]
    pub error_description: String,
    #[serde(default)]
    pub interval: Option<u64>,
}

/// Result of a poll attempt
#[derive(Debug)]
pub enum PollResult {
    /// Still waiting for user to authorize
    Pending,
    /// User authorized — here's the token
    Success(TokenResponse),
    /// Polling too fast — increase interval
    SlowDown(u64),
    /// Token expired — need to restart the flow
    Expired,
    /// User denied access
    AccessDenied,
    /// Other error
    Error(String),
}

/// Step 1: Request device and user verification codes from GitHub.
pub fn request_device_code() -> Result<DeviceCodeResponse> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPES)])
        .send()
        .context("Failed to contact GitHub for device code")?;

    let status = resp.status();
    let body = resp.text().context("Failed to read GitHub response")?;

    if !status.is_success() {
        anyhow::bail!("GitHub returned {}: {}", status, body);
    }

    let response: DeviceCodeResponse =
        serde_json::from_str(&body).context("Failed to parse device code response")?;

    Ok(response)
}

/// Step 3: Poll GitHub to check if the user has authorized the device.
pub fn poll_for_token(device_code: &str) -> PollResult {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", CLIENT_ID),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send();

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return PollResult::Error(format!("Network error: {}", e)),
    };

    let body = match resp.text() {
        Ok(b) => b,
        Err(e) => return PollResult::Error(format!("Read error: {}", e)),
    };

    // Try parsing as success first
    if let Ok(token) = serde_json::from_str::<TokenResponse>(&body) {
        if !token.access_token.is_empty() {
            return PollResult::Success(token);
        }
    }

    // Try parsing as error
    if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
        return match err.error.as_str() {
            "authorization_pending" => PollResult::Pending,
            "slow_down" => PollResult::SlowDown(err.interval.unwrap_or(10)),
            "expired_token" => PollResult::Expired,
            "access_denied" => PollResult::AccessDenied,
            _ => PollResult::Error(err.error_description),
        };
    }

    PollResult::Error(format!("Unexpected response: {}", body))
}

/// Fetch the authenticated user's username.
pub fn get_username(token: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("Failed to fetch user info")?;

    let body: serde_json::Value = resp.json().context("Failed to parse user response")?;
    let login = body["login"]
        .as_str()
        .context("Missing login field")?
        .to_string();

    Ok(login)
}

/// Create a GitHub repository using the API.
pub fn create_repo(token: &str, name: &str, description: &str, private: bool) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "name": name,
        "description": description,
        "private": private,
        "auto_init": false,
    });

    let resp = client
        .post("https://api.github.com/user/repos")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .context("Failed to create repository")?;

    let status = resp.status();
    let resp_body: serde_json::Value = resp.json().context("Failed to parse response")?;

    if status.is_success() {
        let clone_url = resp_body["clone_url"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        Ok(clone_url)
    } else {
        let msg = resp_body["message"]
            .as_str()
            .unwrap_or("Unknown error")
            .to_string();
        anyhow::bail!("{}", msg)
    }
}

/// A GitHub collaborator entry.
#[derive(Debug, Clone)]
pub struct Collaborator {
    pub login: String,
    pub role: String,
}

/// Parse owner/repo from a GitHub remote URL.
/// Supports both HTTPS (https://github.com/owner/repo.git) and SSH (git@github.com:owner/repo.git).
pub fn parse_repo_from_remote() -> Result<(String, String)> {
    let output = super::runner::run_git(&["remote", "get-url", "origin"])
        .context("No 'origin' remote found")?;
    let url = output.trim();

    // SSH: git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let path = rest.trim_end_matches(".git");
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // HTTPS: https://github.com/owner/repo.git
    if url.contains("github.com") {
        let path = url
            .split("github.com/")
            .nth(1)
            .context("Cannot parse GitHub URL")?;
        let path = path.trim_end_matches(".git");
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    anyhow::bail!("Remote 'origin' is not a GitHub URL: {}", url)
}

/// List collaborators for the current repository.
pub fn list_collaborators(token: &str) -> Result<Vec<Collaborator>> {
    let (owner, repo) = parse_repo_from_remote()?;
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!(
            "https://api.github.com/repos/{}/{}/collaborators",
            owner, repo
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("Failed to fetch collaborators")?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .context("Failed to parse collaborators response")?;

    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }

    let collabs = body
        .as_array()
        .context("Expected array")?
        .iter()
        .filter_map(|c| {
            let login = c["login"].as_str()?.to_string();
            let role = c["role_name"]
                .as_str()
                .unwrap_or("collaborator")
                .to_string();
            Some(Collaborator { login, role })
        })
        .collect();

    Ok(collabs)
}

/// Add a collaborator to the current repository.
pub fn add_collaborator(token: &str, username: &str) -> Result<String> {
    let (owner, repo) = parse_repo_from_remote()?;
    let client = reqwest::blocking::Client::new();
    let resp = client
        .put(format!(
            "https://api.github.com/repos/{}/{}/collaborators/{}",
            owner, repo, username
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .json(&serde_json::json!({"permission": "push"}))
        .send()
        .context("Failed to add collaborator")?;

    let status = resp.status();
    if status.is_success() || status.as_u16() == 201 || status.as_u16() == 204 {
        Ok(format!("Invited '{}' as collaborator", username))
    } else {
        let body: serde_json::Value = resp.json().unwrap_or_default();
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg)
    }
}

/// Remove a collaborator from the current repository.
pub fn remove_collaborator(token: &str, username: &str) -> Result<()> {
    let (owner, repo) = parse_repo_from_remote()?;
    let client = reqwest::blocking::Client::new();
    let resp = client
        .delete(format!(
            "https://api.github.com/repos/{}/{}/collaborators/{}",
            owner, repo, username
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("Failed to remove collaborator")?;

    let status = resp.status();
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body: serde_json::Value = resp.json().unwrap_or_default();
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg)
    }
}

// ─── Pull Request Types ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub body: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    #[serde(default)]
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub mergeable_state: Option<String>,
    pub head: PrBranch,
    pub base: PrBranch,
    pub user: GhUser,
    #[serde(default)]
    pub labels: Vec<GhLabel>,
    #[serde(default)]
    pub requested_reviewers: Vec<GhUser>,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub changed_files: Option<u64>,
    pub comments: Option<u64>,
    pub review_comments: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PrBranch {
    #[serde(default)]
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhUser {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GhLabel {
    pub name: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRunsResponse {
    pub total_count: u64,
    pub check_runs: Vec<CheckRun>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    #[serde(default)]
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PrFile {
    pub filename: String,
    pub status: String,
    pub additions: u64,
    pub deletions: u64,
    pub changes: u64,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PrReview {
    pub user: GhUser,
    pub state: String,
    pub body: Option<String>,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeResponse {
    pub sha: String,
    pub merged: bool,
    pub message: String,
}

// ─── Pull Request API Functions ────────────────────────────────

fn gh_get(token: &str, url: &str) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("GitHub API request failed")
}

fn gh_put_json(
    token: &str,
    url: &str,
    body: &serde_json::Value,
) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    client
        .put(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .json(body)
        .send()
        .context("GitHub API request failed")
}

fn gh_patch_json(
    token: &str,
    url: &str,
    body: &serde_json::Value,
) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    client
        .patch(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .json(body)
        .send()
        .context("GitHub API request failed")
}

/// List pull requests. `state` is "open", "closed", or "all".
pub fn list_pull_requests(token: &str, state: &str) -> Result<Vec<PullRequest>> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state={}&per_page=50&sort=updated&direction=desc",
        owner, repo, state
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse PR list")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let prs: Vec<PullRequest> =
        serde_json::from_value(body).context("Failed to deserialize PR list")?;
    Ok(prs)
}

/// Get a single pull request with full detail (includes mergeable, additions/deletions).
pub fn get_pull_request(token: &str, number: u64) -> Result<PullRequest> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, number
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse PR detail")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let pr: PullRequest = serde_json::from_value(body).context("Failed to deserialize PR")?;
    Ok(pr)
}

/// Get CI check runs for a commit SHA.
pub fn get_check_runs(token: &str, sha: &str) -> Result<CheckRunsResponse> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
        owner, repo, sha
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse check runs")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let runs: CheckRunsResponse =
        serde_json::from_value(body).context("Failed to deserialize check runs")?;
    Ok(runs)
}

/// Get files changed in a PR.
pub fn get_pr_files(token: &str, number: u64) -> Result<Vec<PrFile>> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/files?per_page=100",
        owner, repo, number
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse PR files")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let files: Vec<PrFile> =
        serde_json::from_value(body).context("Failed to deserialize PR files")?;
    Ok(files)
}

/// Get reviews on a PR.
pub fn get_pr_reviews(token: &str, number: u64) -> Result<Vec<PrReview>> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, number
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse PR reviews")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let reviews: Vec<PrReview> =
        serde_json::from_value(body).context("Failed to deserialize PR reviews")?;
    Ok(reviews)
}

/// Merge a pull request. `merge_method` is "merge", "squash", or "rebase".
pub fn merge_pull_request(token: &str, number: u64, merge_method: &str) -> Result<MergeResponse> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/merge",
        owner, repo, number
    );
    let body = serde_json::json!({ "merge_method": merge_method });
    let resp = gh_put_json(token, &url, &body)?;
    let status = resp.status();
    let resp_body: serde_json::Value = resp.json().context("Failed to parse merge response")?;
    if !status.is_success() {
        let msg = resp_body["message"].as_str().unwrap_or("Merge failed");
        anyhow::bail!("{}", msg);
    }
    let merge: MergeResponse =
        serde_json::from_value(resp_body).context("Failed to deserialize merge response")?;
    Ok(merge)
}

/// Close a pull request.
pub fn close_pull_request(token: &str, number: u64) -> Result<PullRequest> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, number
    );
    let body = serde_json::json!({ "state": "closed" });
    let resp = gh_patch_json(token, &url, &body)?;
    let status = resp.status();
    let resp_body: serde_json::Value = resp.json().context("Failed to parse close response")?;
    if !status.is_success() {
        let msg = resp_body["message"].as_str().unwrap_or("Close failed");
        anyhow::bail!("{}", msg);
    }
    let pr: PullRequest = serde_json::from_value(resp_body).context("Failed to deserialize PR")?;
    Ok(pr)
}

// ─── GitHub Actions Types ────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub head_branch: Option<String>,
    pub event: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub run_number: u64,
    pub display_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRunsResponse {
    #[allow(dead_code)]
    pub total_count: u64,
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkflowJob {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub steps: Vec<JobStep>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct JobStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub number: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowJobsResponse {
    #[allow(dead_code)]
    pub total_count: u64,
    pub jobs: Vec<WorkflowJob>,
}

// ─── GitHub Actions API Functions ────────────────────────────────

/// List workflow runs for the repository, sorted newest first.
pub fn list_workflow_runs(token: &str) -> Result<WorkflowRunsResponse> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs?per_page=30&sort=created&direction=desc",
        owner, repo
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse workflow runs")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let runs: WorkflowRunsResponse =
        serde_json::from_value(body).context("Failed to deserialize workflow runs")?;
    Ok(runs)
}

/// List jobs for a specific workflow run.
pub fn list_run_jobs(token: &str, run_id: u64) -> Result<WorkflowJobsResponse> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
        owner, repo, run_id
    );
    let resp = gh_get(token, &url)?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse workflow jobs")?;
    if !status.is_success() {
        let msg = body["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("{}", msg);
    }
    let jobs: WorkflowJobsResponse =
        serde_json::from_value(body).context("Failed to deserialize workflow jobs")?;
    Ok(jobs)
}

/// Download logs for a specific job. Returns the log text.
pub fn get_job_logs(token: &str, job_id: u64) -> Result<String> {
    let (owner, repo) = parse_repo_from_remote()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/jobs/{}/logs",
        owner, repo, job_id
    );
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("Failed to fetch job logs")?;

    let status = resp.status();
    if status.is_redirection() {
        // GitHub redirects to a temporary URL for log downloads
        if let Some(location) = resp.headers().get("location") {
            let redirect_url = location.to_str().unwrap_or("");
            let log_resp =
                reqwest::blocking::get(redirect_url).context("Failed to follow log redirect")?;
            return log_resp.text().context("Failed to read log text");
        }
    }

    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("Failed to fetch logs ({}): {}", status, body);
    }

    resp.text().context("Failed to read log text")
}
