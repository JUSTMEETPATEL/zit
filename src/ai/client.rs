//! AI Mentor client — supports multiple AI providers.
//!
//! Providers: Amazon Bedrock (Lambda), OpenAI, Anthropic, OpenRouter, Ollama.
//! See `ai::provider` for the trait and implementations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::AiConfig;
use crate::git;

// ─── Constants ─────────────────────────────────────────────────

/// Maximum number of retry attempts for transient failures.
const MAX_RETRIES: u32 = 2;

/// Re-export from parent module for local use.
use super::DIFF_TRUNCATE_AT;

/// Cache TTL — responses are cached for 5 minutes.
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Maximum cached entries before eviction.
const CACHE_MAX_ENTRIES: usize = 50;

// ─── Request / Response Types ──────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MentorRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<RepoContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RepoContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub staged_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unstaged_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_stats: Option<DiffStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflict_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_diff: Option<String>,
    #[serde(default)]
    pub has_conflicts: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_type: Option<String>,
    #[serde(default)]
    pub detached_head: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Deserialize)]
pub struct MentorApiResponse {
    pub success: bool,
    #[serde(default)]
    pub response: Option<MentorResponseData>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MentorResponseData {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub response_type: Option<String>,
    pub content: Option<String>,
}

// ─── Cache ─────────────────────────────────────────────────────

#[derive(Clone)]
struct CacheEntry {
    response: String,
    created: Instant,
}

type ResponseCache = Arc<Mutex<HashMap<String, CacheEntry>>>;

/// Compute a simple hash key for a request (type + query + error + branch).
fn cache_key(request: &MentorRequest) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    request.request_type.hash(&mut hasher);
    request.query.hash(&mut hasher);
    request.error.hash(&mut hasher);
    if let Some(ref ctx) = request.context {
        ctx.branch.hash(&mut hasher);
        ctx.staged_files.hash(&mut hasher);
        ctx.diff.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

use crate::ai::prompts;
use crate::ai::provider::{self, AiProvider};

// ─── Client ────────────────────────────────────────────────────

/// AI Mentor client that dispatches requests through a provider.
pub struct AiClient {
    provider: Box<dyn AiProvider>,
    /// Keep raw endpoint/api_key for Bedrock's call_raw() path.
    endpoint: String,
    api_key: String,
    provider_kind: String,
    client: reqwest::blocking::Client,
    cache: ResponseCache,
}

impl AiClient {
    /// Create a new AI client from config. Returns None if AI is not configured.
    pub fn from_config(config: &AiConfig) -> Option<Self> {
        if !config.enabled {
            return None;
        }

        let prov = provider::create_provider(config)?;
        let provider_kind = config.effective_provider().to_string();
        let endpoint = config.effective_endpoint().unwrap_or_default();
        let api_key = config.resolved_api_key().unwrap_or_default();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(
                config.timeout_secs.unwrap_or(30),
            ))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        Some(Self {
            provider: prov,
            endpoint,
            api_key,
            provider_kind,
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// The active provider's display name (e.g. "Amazon Bedrock", "Ollama").
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// The model being used.
    #[allow(dead_code)]
    pub fn model_name(&self) -> &str {
        self.provider.model_name()
    }

    /// Generate a unique request ID for tracing.
    fn request_id() -> String {
        use std::time::SystemTime;
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("zit-{:x}", ts)
    }

    /// Look up a cached response. Returns None if not found or expired.
    fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().ok()?;
        if let Some(entry) = cache.get(key) {
            if entry.created.elapsed() < CACHE_TTL {
                return Some(entry.response.clone());
            }
        }
        None
    }

    /// Store a response in cache, evicting old entries if needed.
    fn set_cached(&self, key: String, response: String) {
        if let Ok(mut cache) = self.cache.lock() {
            // Evict expired entries
            cache.retain(|_, v| v.created.elapsed() < CACHE_TTL);
            // Evict oldest if at capacity
            if cache.len() >= CACHE_MAX_ENTRIES {
                if let Some(oldest_key) = cache
                    .iter()
                    .min_by_key(|(_, v)| v.created)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
            cache.insert(
                key,
                CacheEntry {
                    response,
                    created: Instant::now(),
                },
            );
        }
    }

    /// Send a request to the AI mentor with caching, retry, and error classification.
    /// For Bedrock, sends the full MentorRequest JSON to Lambda.
    /// For other providers, builds prompts client-side and calls provider.chat().
    fn call(&self, request: &MentorRequest) -> Result<String> {
        // Check cache first
        let ckey = cache_key(request);
        if let Some(cached) = self.get_cached(&ckey) {
            log::debug!("AI cache hit for type={}", request.request_type);
            return Ok(cached);
        }

        let request_id = Self::request_id();
        log::info!(
            "AI request: type={} provider={} id={}",
            request.request_type,
            self.provider_kind,
            request_id
        );

        let result = if self.provider_kind == "bedrock" {
            // Bedrock: send full JSON to Lambda (Lambda constructs prompts)
            self.call_bedrock(request)
        } else {
            // Direct providers: build prompts client-side
            self.call_direct(request)
        };

        // Cache on success
        if let Ok(ref response) = result {
            self.set_cached(ckey, response.clone());
        }

        result
    }

    /// Bedrock path: send full MentorRequest as JSON to Lambda.
    fn call_bedrock(&self, request: &MentorRequest) -> Result<String> {
        let body = serde_json::to_value(request).context("Failed to serialize request")?;

        // Downcast provider to BedrockProvider to use call_raw
        // This is safe because we only reach here when provider_kind == "bedrock"
        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(
                    500 * 2u64.pow(attempt - 1),
                ));
            }

            let send_result = self
                .client
                .post(&self.endpoint)
                .header("Content-Type", "application/json")
                .header("x-api-key", &self.api_key)
                .header("x-request-id", Self::request_id())
                .json(&body)
                .send();

            match send_result {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_client_error() {
                        return self.parse_error_response(resp, status.as_u16());
                    }

                    if status.is_server_error() {
                        last_error = Some(anyhow::anyhow!(classify_http_error(status.as_u16())));
                        continue;
                    }

                    return self.parse_success_response(resp);
                }
                Err(e) => {
                    if attempt == 0 && e.is_connect() {
                        return Err(anyhow::anyhow!(
                            "You appear to be offline — cannot reach AI service."
                        ));
                    }
                    last_error = Some(classify_request_error(e));
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("AI request failed after retries")))
    }

    /// Parse a successful Bedrock/Lambda response.
    fn parse_success_response(&self, resp: reqwest::blocking::Response) -> Result<String> {
        let body: MentorApiResponse = resp.json().context("Failed to parse AI mentor response")?;

        if !body.success {
            let err_msg = body
                .error
                .unwrap_or_else(|| "Unknown API error".to_string());
            anyhow::bail!("{}", err_msg);
        }

        body.response
            .and_then(|r| r.content)
            .ok_or_else(|| anyhow::anyhow!("Empty response from AI mentor"))
    }

    /// Parse an error response from Bedrock/Lambda.
    fn parse_error_response(
        &self,
        resp: reqwest::blocking::Response,
        status: u16,
    ) -> Result<String> {
        if let Ok(body) = resp.json::<MentorApiResponse>() {
            if let Some(err) = body.error {
                anyhow::bail!("{}", err);
            }
        }
        anyhow::bail!("{}", classify_http_error(status));
    }

    /// Direct provider path: build prompts client-side, call provider.chat().
    fn call_direct(&self, request: &MentorRequest) -> Result<String> {
        let ctx = request
            .context
            .as_ref()
            .cloned()
            .unwrap_or_else(|| RepoContext {
                branch: None,
                staged_files: vec![],
                unstaged_files: vec![],
                diff_stats: None,
                diff: None,
                conflict_files: vec![],
                conflict_diff: None,
                has_conflicts: false,
                merge_type: None,
                detached_head: false,
            });

        let system_prompt = prompts::system_prompt_for(&request.request_type);
        let user_message = prompts::build_user_message(
            &request.request_type,
            &ctx,
            request.query.as_deref(),
            request.error.as_deref(),
        );

        log::debug!(
            "[AI] call_direct: type={} provider={} prompt_len={} user_msg_len={}",
            request.request_type,
            self.provider_kind,
            system_prompt.len(),
            user_message.len()
        );

        // Retry with backoff
        let mut last_error = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(
                    500 * 2u64.pow(attempt - 1),
                ));
            }

            match self.provider.chat(system_prompt, &user_message) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("AI request failed after retries")))
    }

    /// Suggest a commit message based on staged changes.
    pub fn suggest_commit_message(&self) -> Result<String> {
        let ctx = build_repo_context(true)?;
        let request = MentorRequest {
            request_type: "commit_suggestion".to_string(),
            context: Some(ctx),
            query: None,
            error: None,
        };
        self.call(&request)
    }

    /// Explain the current repository state.
    pub fn explain_repo(&self, query: Option<&str>) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "explain".to_string(),
            context: Some(ctx),
            query: query.map(|s| s.to_string()),
            error: None,
        };
        self.call(&request)
    }

    /// Explain a git error and suggest fixes.
    pub fn explain_error(&self, error_message: &str) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "error".to_string(),
            context: Some(ctx),
            query: None,
            error: Some(error_message.to_string()),
        };
        self.call(&request)
    }

    /// Get a recommendation for a git operation.
    pub fn recommend(&self, query: &str) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "recommend".to_string(),
            context: Some(ctx),
            query: Some(query.to_string()),
            error: None,
        };
        self.call(&request)
    }

    /// Check if the AI service is reachable.
    pub fn health_check(&self) -> Result<String> {
        self.provider.health_check()
    }

    /// Review a specific file's diff using AI. Returns review comments/suggestions.
    pub fn review_diff(&self, file_path: &str, diff_content: &str) -> Result<String> {
        let branch = git::branch::BranchOps::current().ok();

        // Truncate diff if too long
        let diff_text = if diff_content.len() > DIFF_TRUNCATE_AT {
            format!("{}...(truncated)", &diff_content[..DIFF_TRUNCATE_AT])
        } else {
            diff_content.to_string()
        };

        let context = RepoContext {
            branch,
            staged_files: vec![file_path.to_string()],
            unstaged_files: vec![],
            diff_stats: None,
            diff: Some(diff_text),
            conflict_files: vec![],
            conflict_diff: None,
            has_conflicts: false,
            merge_type: None,
            detached_head: false,
        };

        let request = MentorRequest {
            request_type: "review".to_string(),
            context: Some(context),
            query: Some(format!(
                "Review the following diff for file '{}'. Identify potential issues, \
                 suggest improvements, and highlight anything noteworthy. \
                 Be concise and actionable.",
                file_path
            )),
            error: None,
        };

        self.call(&request)
    }

    /// Learn about a git topic with beginner-friendly explanations.
    pub fn learn(&self, topic: &str) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "learn".to_string(),
            context: Some(ctx),
            query: Some(topic.to_string()),
            error: None,
        };
        self.call(&request)
    }

    /// Ask a free-form question with full repo context.
    pub fn ask(&self, question: &str) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "explain".to_string(),
            context: Some(ctx),
            query: Some(question.to_string()),
            error: None,
        };
        self.call(&request)
    }

    /// Suggest a merge conflict resolution for a specific file.
    pub fn suggest_merge_resolution(
        &self,
        file_path: &str,
        conflict_content: &str,
    ) -> Result<String> {
        let branch = git::branch::BranchOps::current().ok();
        let status = git::status::get_status().unwrap_or_default();
        let conflict_files: Vec<String> = status.conflicts.iter().map(|f| f.path.clone()).collect();
        let merge_state = git::merge::get_merge_state();

        // Truncate if too long
        let conflict_text = if conflict_content.len() > DIFF_TRUNCATE_AT {
            format!("{}...(truncated)", &conflict_content[..DIFF_TRUNCATE_AT])
        } else {
            conflict_content.to_string()
        };

        let context = RepoContext {
            branch,
            staged_files: vec![],
            unstaged_files: vec![],
            diff_stats: None,
            diff: None,
            conflict_files,
            conflict_diff: Some(conflict_text),
            has_conflicts: true,
            merge_type: merge_state.map(|s| s.merge_type.to_string()),
            detached_head: false,
        };

        let request = MentorRequest {
            request_type: "merge_resolve".to_string(),
            context: Some(context),
            query: Some(format!(
                "Resolve the merge conflict in file '{}'. Analyze both sides and recommend the best resolution.",
                file_path
            )),
            error: None,
        };

        self.call(&request)
    }

    /// Get AI recommendation for the best merge strategy.
    pub fn suggest_merge_strategy(&self, query: Option<&str>) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "merge_strategy".to_string(),
            context: Some(ctx),
            query: Some(
                query
                    .unwrap_or("What is the safest strategy to integrate these branches?")
                    .to_string(),
            ),
            error: None,
        };
        self.call(&request)
    }

    /// Generate a .gitignore file based on the project structure using AI.
    pub fn generate_gitignore(&self) -> Result<String> {
        // Collect file listing: tracked + untracked (excluding ignored)
        let tracked = git::run_git(&["ls-files"]).unwrap_or_default();
        let untracked =
            git::run_git(&["ls-files", "--others", "--exclude-standard"]).unwrap_or_default();

        let mut file_listing = String::new();
        for line in tracked.lines().chain(untracked.lines()) {
            if !line.is_empty() {
                file_listing.push_str(line);
                file_listing.push('\n');
            }
        }

        // Truncate if the listing is very large
        if file_listing.len() > 8000 {
            file_listing.truncate(8000);
            file_listing.push_str("\n...(truncated)");
        }

        // Read existing .gitignore if present
        let existing_gitignore = std::fs::read_to_string(".gitignore").ok();

        let request = MentorRequest {
            request_type: "generate_gitignore".to_string(),
            context: None,
            query: Some(file_listing),
            error: existing_gitignore, // reuse error field for existing .gitignore context
        };
        self.call(&request)
    }

    /// Get AI recommendation for resetting to a specific commit.
    pub fn suggest_reset(
        &self,
        current_hash: &str,
        target_hash: &str,
        target_msg: &str,
        commits_back: usize,
    ) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let query = format!(
            "I want to reset from current HEAD ({}) to commit {} (\"{}\"), \
             which is {} commit(s) back. \
             Explain what will happen with: --soft, --mixed, and --hard reset. \
             Which type do you recommend and why? Be concise.",
            current_hash, target_hash, target_msg, commits_back
        );
        let request = MentorRequest {
            request_type: "recommend".to_string(),
            context: Some(ctx),
            query: Some(query),
            error: None,
        };
        self.call(&request)
    }

    /// Check if the client is configured and reachable (quick check, no API call).
    #[allow(dead_code)]
    pub fn is_configured(&self) -> bool {
        !self.provider_kind.is_empty()
    }

    /// Agent mode: send a user message with full conversation context.
    /// The `user_message` should contain the formatted conversation history
    /// and the latest user intent.
    pub fn agent_chat(&self, user_message: &str) -> Result<String> {
        let ctx = build_repo_context(false)?;
        let request = MentorRequest {
            request_type: "agent".to_string(),
            context: Some(ctx),
            query: Some(user_message.to_string()),
            error: None,
        };
        // Don't use the cached call — agent responses should never be cached
        if self.provider_kind == "bedrock" {
            self.call_bedrock(&request)
        } else {
            self.call_direct(&request)
        }
    }
}

// ─── Helpers ───────────────────────────────────────────────────

/// Build repository context from the current git state.
fn build_repo_context(include_diff: bool) -> Result<RepoContext> {
    let branch = git::branch::BranchOps::current().ok();
    let detached_head = branch.as_deref() == Some("HEAD");

    let status = git::status::get_status().unwrap_or_default();
    let staged_files: Vec<String> = status.staged.iter().map(|f| f.path.clone()).collect();
    let unstaged_files: Vec<String> = status
        .unstaged
        .iter()
        .chain(status.untracked.iter())
        .map(|f| f.path.clone())
        .collect();
    let conflict_files: Vec<String> = status.conflicts.iter().map(|f| f.path.clone()).collect();
    let has_conflicts = !conflict_files.is_empty();
    let merge_state = git::merge::get_merge_state();
    let merge_type = merge_state.map(|s| s.merge_type.to_string());

    let mut diff_text = None;
    let mut diff_stats = None;

    if include_diff && !staged_files.is_empty() {
        // Get staged diff for commit suggestions
        if let Ok(stat) = git::diff::get_staged_stat() {
            let parts = parse_stat_line(&stat);
            diff_stats = Some(DiffStats {
                files_changed: parts.0,
                insertions: parts.1,
                deletions: parts.2,
            });
        }

        if let Ok(diffs) = git::diff::get_staged_diff() {
            let mut combined = String::new();
            for file_diff in &diffs {
                for hunk in &file_diff.hunks {
                    for line in &hunk.lines {
                        combined.push_str(&line.content);
                        combined.push('\n');
                        if combined.len() > DIFF_TRUNCATE_AT {
                            combined.push_str("...(truncated)");
                            break;
                        }
                    }
                    if combined.len() > DIFF_TRUNCATE_AT {
                        break;
                    }
                }
                if combined.len() > DIFF_TRUNCATE_AT {
                    break;
                }
            }
            if !combined.is_empty() {
                diff_text = Some(combined);
            }
        }
    }

    Ok(RepoContext {
        branch,
        staged_files,
        unstaged_files,
        diff_stats,
        diff: diff_text,
        conflict_files,
        conflict_diff: None,
        has_conflicts,
        merge_type,
        detached_head,
    })
}

// ─── Error Classification ──────────────────────────────────────

/// Classify a reqwest transport-level error into a user-friendly message.
fn classify_request_error(e: reqwest::Error) -> anyhow::Error {
    if e.is_timeout() {
        anyhow::anyhow!(
            "Request timed out — the AI service may be under load. Try again in a moment."
        )
    } else if e.is_connect() {
        anyhow::anyhow!(
            "Cannot connect to AI service — check your internet connection and endpoint URL"
        )
    } else if e.is_decode() {
        anyhow::anyhow!("Received invalid response from AI service")
    } else {
        anyhow::anyhow!("Network error: {}", e)
    }
}

/// Map HTTP status codes to user-friendly error messages.
fn classify_http_error(status: u16) -> String {
    match status {
        401 => {
            "Invalid API key — check [ai] api_key in ~/.config/zit/config.toml or ZIT_AI_API_KEY"
                .to_string()
        }
        403 => "Access denied — your API key may have expired or hit its quota".to_string(),
        429 => "Rate limited — too many requests. Wait a moment and try again.".to_string(),
        500 => "AI service internal error — try again in a moment".to_string(),
        502..=504 => "AI service temporarily unavailable — try again shortly".to_string(),
        _ => format!("API error (HTTP {})", status),
    }
}

/// Parse `git diff --stat` summary line like " 3 files changed, 10 insertions(+), 2 deletions(-)"
fn parse_stat_line(stat: &str) -> (usize, usize, usize) {
    let mut files = 0;
    let mut ins = 0;
    let mut del = 0;

    for part in stat.split(',') {
        let part = part.trim();
        if part.contains("file") {
            files = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        } else if part.contains("insertion") {
            ins = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        } else if part.contains("deletion") {
            del = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
    }

    (files, ins, del)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stat_line_full() {
        let stat = " 3 files changed, 45 insertions(+), 12 deletions(-)";
        let (f, i, d) = parse_stat_line(stat);
        assert_eq!(f, 3);
        assert_eq!(i, 45);
        assert_eq!(d, 12);
    }

    #[test]
    fn test_parse_stat_line_insertions_only() {
        let stat = " 1 file changed, 10 insertions(+)";
        let (f, i, d) = parse_stat_line(stat);
        assert_eq!(f, 1);
        assert_eq!(i, 10);
        assert_eq!(d, 0);
    }

    #[test]
    fn test_parse_stat_line_empty() {
        let (f, i, d) = parse_stat_line("");
        assert_eq!(f, 0);
        assert_eq!(i, 0);
        assert_eq!(d, 0);
    }

    #[test]
    fn test_mentor_request_serialization() {
        let req = MentorRequest {
            request_type: "commit_suggestion".to_string(),
            context: Some(RepoContext {
                branch: Some("main".to_string()),
                staged_files: vec!["src/main.rs".to_string()],
                unstaged_files: vec![],
                diff_stats: Some(DiffStats {
                    files_changed: 1,
                    insertions: 10,
                    deletions: 2,
                }),
                diff: Some("+fn hello() {}".to_string()),
                conflict_files: vec![],
                conflict_diff: None,
                has_conflicts: false,
                merge_type: None,
                detached_head: false,
            }),
            query: None,
            error: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("commit_suggestion"));
        assert!(json.contains("main"));
        assert!(json.contains("src/main.rs"));
        assert!(!json.contains("query")); // None fields should be skipped
        assert!(!json.contains("error"));
    }

    // ── cache_key tests ──────────────────────────────────────────

    #[test]
    fn test_cache_key_deterministic() {
        let req = MentorRequest {
            request_type: "explain".to_string(),
            context: None,
            query: Some("hello".to_string()),
            error: None,
        };
        let k1 = cache_key(&req);
        let k2 = cache_key(&req);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_differs_on_type() {
        let r1 = MentorRequest {
            request_type: "explain".to_string(),
            context: None,
            query: None,
            error: None,
        };
        let r2 = MentorRequest {
            request_type: "commit_suggestion".to_string(),
            context: None,
            query: None,
            error: None,
        };
        assert_ne!(cache_key(&r1), cache_key(&r2));
    }

    #[test]
    fn test_cache_key_differs_on_context_branch() {
        let r1 = MentorRequest {
            request_type: "explain".to_string(),
            context: Some(RepoContext {
                branch: Some("main".to_string()),
                staged_files: vec![],
                unstaged_files: vec![],
                diff_stats: None,
                diff: None,
                conflict_files: vec![],
                conflict_diff: None,
                has_conflicts: false,
                merge_type: None,
                detached_head: false,
            }),
            query: None,
            error: None,
        };
        let r2 = MentorRequest {
            request_type: "explain".to_string(),
            context: Some(RepoContext {
                branch: Some("feature".to_string()),
                staged_files: vec![],
                unstaged_files: vec![],
                diff_stats: None,
                diff: None,
                conflict_files: vec![],
                conflict_diff: None,
                has_conflicts: false,
                merge_type: None,
                detached_head: false,
            }),
            query: None,
            error: None,
        };
        assert_ne!(cache_key(&r1), cache_key(&r2));
    }

    // ── Cache get/set tests ──────────────────────────────────────

    fn make_test_client() -> AiClient {
        // Create a dummy bedrock-style client for cache tests
        let config = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("https://example.com/mentor".to_string()),
            api_key: Some("test-key-12345".to_string()),
            timeout_secs: Some(30),
        };
        AiClient::from_config(&config).expect("test client should build")
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let client = make_test_client();
        assert!(client.get_cached("nonexistent").is_none());
    }

    #[test]
    fn test_cache_hit_returns_value() {
        let client = make_test_client();
        client.set_cached("k1".to_string(), "response text".to_string());
        assert_eq!(client.get_cached("k1"), Some("response text".to_string()));
    }

    #[test]
    fn test_cache_evicts_at_capacity() {
        let client = make_test_client();
        // Fill cache to capacity
        for i in 0..CACHE_MAX_ENTRIES {
            client.set_cached(format!("k{}", i), format!("v{}", i));
        }
        // Adding one more should still work (oldest evicted)
        client.set_cached("overflow".to_string(), "data".to_string());
        let cache = client.cache.lock().unwrap();
        assert!(cache.len() <= CACHE_MAX_ENTRIES);
        assert!(cache.contains_key("overflow"));
    }

    // ── classify_http_error tests ────────────────────────────────

    #[test]
    fn test_classify_http_401() {
        let msg = classify_http_error(401);
        assert!(msg.contains("Invalid API key"));
    }

    #[test]
    fn test_classify_http_403() {
        let msg = classify_http_error(403);
        assert!(msg.contains("Access denied"));
    }

    #[test]
    fn test_classify_http_429() {
        let msg = classify_http_error(429);
        assert!(msg.contains("Rate limited"));
    }

    #[test]
    fn test_classify_http_500() {
        let msg = classify_http_error(500);
        assert!(msg.contains("internal error"));
    }

    #[test]
    fn test_classify_http_502() {
        let msg = classify_http_error(502);
        assert!(msg.contains("temporarily unavailable"));
    }

    #[test]
    fn test_classify_http_unknown() {
        let msg = classify_http_error(418);
        assert!(msg.contains("418"));
    }

    // ── parse_success_response tests ─────────────────────────────

    #[test]
    fn test_response_deserialization_success() {
        let json = r#"{"success": true, "response": {"type": "commit_suggestion", "content": "fix: typo"}}"#;
        let resp: MentorApiResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.response.unwrap().content.unwrap(), "fix: typo");
    }

    #[test]
    fn test_response_deserialization_error() {
        let json = r#"{"success": false, "error": "quota exceeded"}"#;
        let resp: MentorApiResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.error.unwrap(), "quota exceeded");
    }

    #[test]
    fn test_response_deserialization_empty_content() {
        let json = r#"{"success": true, "response": {"type": "explain"}}"#;
        let resp: MentorApiResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert!(resp.response.unwrap().content.is_none());
    }

    // ── from_config validation tests ─────────────────────────────

    #[test]
    fn test_from_config_disabled() {
        let cfg = AiConfig {
            enabled: false,
            endpoint: Some("https://example.com/mentor".to_string()),
            api_key: Some("key123".to_string()),
            ..AiConfig::default()
        };
        assert!(AiClient::from_config(&cfg).is_none());
    }

    #[test]
    fn test_from_config_bad_scheme_caught_by_validate() {
        let cfg = AiConfig {
            enabled: true,
            endpoint: Some("ftp://example.com/mentor".to_string()),
            api_key: Some("key123456789".to_string()),
            ..AiConfig::default()
        };
        // from_config now creates the client (scheme is checked by validate())
        assert!(AiClient::from_config(&cfg).is_some());
        // But validate() should flag the bad scheme
        let issues = cfg.validate();
        assert!(issues
            .iter()
            .any(|i| i.contains("must start with https://")));
    }

    // ── is_configured tests ──────────────────────────────────────

    #[test]
    fn test_is_configured_true() {
        let client = make_test_client();
        assert!(client.is_configured());
    }

    // ── stat line edge cases ─────────────────────────────────────

    #[test]
    fn test_parse_stat_line_deletions_only() {
        let stat = " 2 files changed, 5 deletions(-)";
        let (f, i, d) = parse_stat_line(stat);
        assert_eq!(f, 2);
        assert_eq!(i, 0);
        assert_eq!(d, 5);
    }
}
