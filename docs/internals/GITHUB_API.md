# GitHub API + OAuth Device Flow

## What It Is
GitHub's REST API v3 for managing repositories, pull requests, collaborators, and CI/CD. The OAuth Device Flow allows CLI apps to authenticate without a browser redirect URI.

## Why My Project Uses It
Zit provides full GitHub integration from the terminal — users can create repos, push/pull, manage collaborators, create/merge/close PRs, and monitor GitHub Actions workflows. The OAuth device flow lets users authenticate securely without copying tokens manually.

## Where It Appears in My Project
- `src/git/github_auth.rs` (716 lines) — All GitHub API functions + OAuth device flow
- `src/ui/github.rs` (95,987 bytes) — GitHub integration UI (massive file with tabs for repos, PRs, Actions, collaborators)
- `src/config.rs` lines 25–48: `GithubConfig` with token resolution (keychain → config → env)
- `src/keychain.rs` lines 49–81: Secure token storage for GitHub OAuth tokens and PATs

## How It Works Internally
**OAuth Device Flow:**
1. App calls `POST https://github.com/login/device/code` with the client ID and desired scopes (`repo,read:user`).
2. GitHub returns a `user_code` (e.g., `ABCD-1234`) and a `verification_uri` (`https://github.com/login/device`).
3. The app displays the code and opens the browser. User enters the code on GitHub's website.
4. App polls `POST https://github.com/login/oauth/access_token` every N seconds with the `device_code`.
5. Once the user authorizes, GitHub returns an `access_token`. The app stores it in the OS keychain.

**API Operations:**
- **Repo creation**: `POST /user/repos` with name, description, private flag
- **PR management**: `GET/POST /repos/:owner/:repo/pulls` — list, get, merge, close
- **CI/CD**: `GET /repos/:owner/:repo/actions/runs` — list workflow runs, jobs, download logs
- **Collaborators**: `GET/PUT/DELETE /repos/:owner/:repo/collaborators/:username`

## Key Concepts I Must Know
- **Client ID**: `Ov23liMBOn6cAuIPFslq` — registered OAuth App on GitHub (github_auth.rs:5)
- **Scopes**: `repo` (full repo access) + `read:user` (read username)
- **Token hierarchy**: OS keychain > config file > environment variable
- **Rate limiting**: GitHub API allows 5,000 requests/hour for authenticated requests
- **Polling interval**: Device flow polling respects GitHub's `interval` field; `slow_down` errors increase the interval
- **PollResult enum**: `Pending | Success | SlowDown | Expired | AccessDenied | Error`

## How My Code Uses It (Annotated)
```rust
// src/git/github_auth.rs:58-78 — Step 1: Request device code
pub fn request_device_code() -> Result<DeviceCodeResponse> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")  // GitHub's device flow endpoint
        .header("Accept", "application/json")            // Request JSON response
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPES)]) // OAuth params
        .send()?;
    let response: DeviceCodeResponse = serde_json::from_str(&body)?;
    Ok(response)  // Contains user_code, verification_uri, device_code
}

// src/git/github_auth.rs:81-122 — Step 3: Poll for token
pub fn poll_for_token(device_code: &str) -> PollResult {
    // ... polls GitHub until user authorizes or flow expires
    match err.error.as_str() {
        "authorization_pending" => PollResult::Pending,     // Keep waiting
        "slow_down" => PollResult::SlowDown(interval),      // Increase poll interval
        "expired_token" => PollResult::Expired,              // Start over
        "access_denied" => PollResult::AccessDenied,         // User denied
    }
}
```

## What Could Go Wrong
- **Token expiration**: OAuth tokens don't expire by default, but users can revoke them from GitHub settings
- **Scope insufficient**: If the user wants to access private org repos, additional scopes may be needed
- **Network on polling**: If the network drops during the device flow, polling fails. The UI handles this gracefully.
- **Rate limiting**: Hitting 5,000 req/hour is unlikely for a CLI tool but possible during aggressive polling

## Judge-Ready One-Liner
"We use GitHub's OAuth Device Flow so users can authenticate with a simple code displayed in the terminal — no need to copy-paste tokens — and then manage repos, PRs, and CI/CD directly from the TUI."
