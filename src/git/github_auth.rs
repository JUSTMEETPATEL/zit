use anyhow::{Context, Result};
use serde::Deserialize;

/// GitHub OAuth App Client ID for zit.
pub const CLIENT_ID: &str = "Ov23liMBOn6cAuIPFslq";

/// Scopes to request — repo access for push/pull/create, read:user for username.
const SCOPES: &str = "repo,read:user";

/// Response from POST /login/device/code
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Successful token response
#[derive(Debug, Deserialize)]
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
        .get(format!("https://api.github.com/repos/{}/{}/collaborators", owner, repo))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "zit-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("Failed to fetch collaborators")?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().context("Failed to parse collaborators response")?;

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
            let role = c["role_name"].as_str().unwrap_or("collaborator").to_string();
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
        .put(format!("https://api.github.com/repos/{}/{}/collaborators/{}", owner, repo, username))
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
        .delete(format!("https://api.github.com/repos/{}/{}/collaborators/{}", owner, repo, username))
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

