use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub github: GithubConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GithubConfig {
    #[serde(default)]
    pub pat: Option<String>,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

impl GithubConfig {
    /// Get the best available token: keychain first, then config file (OAuth preferred over PAT).
    pub fn get_token(&self) -> Option<String> {
        // Try OS keychain first
        if let Some(token) = crate::keychain::get_github_token() {
            return Some(token);
        }
        if let Some(token) = crate::keychain::get_github_pat() {
            return Some(token);
        }
        // Fallback to plaintext config
        self.oauth_token.clone().or_else(|| self.pat.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,
    #[serde(default = "default_true")]
    pub show_help_hints: bool,
}

fn default_tick_rate() -> u64 {
    2000
}

fn default_true() -> bool {
    true
}

fn default_color_scheme() -> String {
    "default".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: default_tick_rate(),
            confirm_destructive: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color_scheme: default_color_scheme(),
            show_help_hints: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    /// Enable AI mentor features.
    #[serde(default)]
    pub enabled: bool,
    /// AI provider: "bedrock" (default), "openai", "anthropic", "openrouter", "ollama".
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Model name (provider-specific). Each provider has a sensible default.
    #[serde(default)]
    pub model: Option<String>,
    /// API endpoint URL. Required for bedrock (Lambda URL). Optional for ollama
    /// (defaults to http://localhost:11434). Ignored for openai/anthropic/openrouter.
    #[serde(default)]
    pub endpoint: Option<String>,
    /// API key. Required for all providers except ollama.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Request timeout in seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

fn default_provider() -> String {
    "bedrock".to_string()
}

/// All supported AI provider names.
pub const VALID_PROVIDERS: &[&str] = &["bedrock", "openai", "anthropic", "openrouter", "ollama"];

impl AiConfig {
    /// Get the effective provider name, normalized to lowercase.
    pub fn effective_provider(&self) -> &str {
        if self.provider.is_empty() {
            "bedrock"
        } else {
            &self.provider
        }
    }

    /// Get the effective model name, falling back to a per-provider default.
    pub fn effective_model(&self) -> String {
        if let Some(ref m) = self.model {
            if !m.is_empty() {
                return m.clone();
            }
        }
        match self.effective_provider() {
            "openai" => "gpt-4o".to_string(),
            "anthropic" => "claude-sonnet-4-20250514".to_string(),
            "openrouter" => "anthropic/claude-sonnet-4".to_string(),
            "ollama" => "llama3.1".to_string(),
            _ => "claude-3-sonnet".to_string(), // bedrock default (display only)
        }
    }

    /// Get the effective endpoint, falling back to per-provider defaults.
    /// Ignores a saved endpoint if it appears to belong to a different provider.
    pub fn effective_endpoint(&self) -> Option<String> {
        let provider = self.effective_provider();

        // Provider-specific defaults
        let default_for_provider = match provider {
            "openai" => Some("https://api.openai.com/v1/chat/completions"),
            "anthropic" => Some("https://api.anthropic.com/v1/messages"),
            "openrouter" => Some("https://openrouter.ai/api/v1/chat/completions"),
            "ollama" => Some("http://localhost:11434"),
            _ => None, // bedrock requires explicit endpoint
        };

        // Config file, then env var
        let from_config = self.endpoint
            .clone()
            .or_else(|| std::env::var("ZIT_AI_ENDPOINT").ok());

        if let Some(ref ep) = from_config {
            // If a known provider has a default, check if the saved endpoint
            // belongs to a different provider (e.g. old Bedrock Lambda URL
            // still saved when switching to OpenRouter).
            if default_for_provider.is_some() {
                let is_stale = (provider != "bedrock" && ep.contains("amazonaws.com"))
                    || (provider != "openai" && ep.contains("api.openai.com"))
                    || (provider != "anthropic" && ep.contains("api.anthropic.com"))
                    || (provider != "openrouter" && ep.contains("openrouter.ai"))
                    || (provider != "ollama" && ep.contains("localhost:11434"));

                if is_stale {
                    log::debug!(
                        "[config] Ignoring stale endpoint '{}' for provider '{}', using default",
                        ep, provider
                    );
                    return default_for_provider.map(|s| s.to_string());
                }
            }
            return from_config;
        }

        default_for_provider.map(|s| s.to_string())
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_provider(),
            model: None,
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
        }
    }
}

impl AiConfig {
    /// Resolve API key: keychain first, then config file, then ZIT_AI_API_KEY env var.
    pub fn resolved_api_key(&self) -> Option<String> {
        // Try OS keychain first
        if let Some(key) = crate::keychain::get_ai_api_key() {
            return Some(key);
        }
        // Config file, then env var
        self.api_key
            .clone()
            .or_else(|| std::env::var("ZIT_AI_API_KEY").ok())
    }

    /// Resolve endpoint: config file first, then ZIT_AI_ENDPOINT env var.
    /// (Legacy helper — prefer effective_endpoint() for provider-aware resolution.)
    pub fn resolved_endpoint(&self) -> Option<String> {
        self.effective_endpoint()
    }

    /// Validate the AI configuration and return a list of issues (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.enabled {
            return issues; // Not enabled, nothing to validate
        }

        let provider = self.effective_provider();

        // Check provider is valid
        if !VALID_PROVIDERS.contains(&provider) {
            issues.push(format!(
                "Unknown AI provider '{}'. Must be one of: {}",
                provider,
                VALID_PROVIDERS.join(", ")
            ));
            return issues;
        }

        // Check endpoint (required for bedrock, optional for ollama, ignored for others)
        match provider {
            "bedrock" => {
                match self.endpoint.as_ref().or(std::env::var("ZIT_AI_ENDPOINT").ok().as_ref()) {
                    None => issues.push(
                        "Bedrock requires an endpoint — set your Lambda API Gateway URL".to_string(),
                    ),
                    Some(ref url) => {
                        if !url.starts_with("https://") && !url.starts_with("http://") {
                            issues.push(format!(
                                "AI endpoint must start with https:// or http://, got: {}",
                                url
                            ));
                        }
                    }
                }
            }
            "ollama" => {
                // Endpoint is optional for ollama (defaults to localhost)
            }
            _ => {
                // openai, anthropic, openrouter have built-in endpoints
            }
        }

        // Check API key (required for all except ollama)
        if provider != "ollama" {
            let resolved_key = self.resolved_api_key();
            if resolved_key.is_none() {
                issues.push(format!(
                    "API key required for '{}' — add 'api_key' to [ai] config or set ZIT_AI_API_KEY",
                    provider
                ));
            } else if let Some(ref key) = resolved_key {
                if key.len() < 8 {
                    issues.push("AI API key seems too short (< 8 chars)".to_string());
                }
            }
        }

        // Check model (required for openrouter)
        if provider == "openrouter" && self.model.is_none() {
            issues.push(
                "OpenRouter requires a model — e.g. model = \"anthropic/claude-sonnet-4\"".to_string(),
            );
        }

        // Check timeout
        if let Some(timeout) = self.timeout_secs {
            if timeout == 0 {
                issues.push("AI timeout_secs must be > 0".to_string());
            } else if timeout > 120 {
                issues.push(
                    "AI timeout_secs > 120s is unusually high — consider lowering".to_string(),
                );
            }
        }

        issues
    }

    /// Check if AI is properly configured.
    pub fn is_ready(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let provider = self.effective_provider();
        match provider {
            "ollama" => self.effective_endpoint().is_some(),
            "bedrock" => self.effective_endpoint().is_some() && self.resolved_api_key().is_some(),
            _ => self.resolved_api_key().is_some(), // openai, anthropic, openrouter
        }
    }
}

impl Config {
    /// Get the config file path (~/.config/zit/config.toml).
    pub fn path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zit");
        config_dir.join("config.toml")
    }

    /// Load config from file, falling back to defaults if file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save config to file, creating directories if needed.
    /// On Unix, sets file permissions to 0o600 (owner-only read/write).
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        // Restrict permissions on Unix (config may contain tokens as fallback)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a bedrock-style AiConfig (the default provider).
    fn bedrock_config() -> AiConfig {
        AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("https://api.example.com/mentor".to_string()),
            api_key: Some("test-api-key-12345".to_string()),
            timeout_secs: Some(30),
        }
    }

    // ── GeneralConfig defaults ──────────────────────────────────────
    #[test]
    fn test_general_config_defaults() {
        let g = GeneralConfig::default();
        assert_eq!(g.tick_rate_ms, 2000);
        assert!(g.confirm_destructive);
    }

    // ── UiConfig defaults ───────────────────────────────────────────
    #[test]
    fn test_ui_config_defaults() {
        let u = UiConfig::default();
        assert_eq!(u.color_scheme, "default");
        assert!(u.show_help_hints);
    }

    // ── AiConfig defaults ───────────────────────────────────────────
    #[test]
    fn test_ai_config_defaults() {
        let a = AiConfig::default();
        assert!(!a.enabled);
        assert_eq!(a.provider, "bedrock");
        assert!(a.model.is_none());
        assert!(a.endpoint.is_none());
        assert!(a.api_key.is_none());
        assert_eq!(a.timeout_secs, Some(30));
    }

    #[test]
    fn test_ai_default_provider_is_bedrock() {
        let a = AiConfig::default();
        assert_eq!(a.effective_provider(), "bedrock");
    }

    #[test]
    fn test_ai_effective_model_defaults() {
        let mut a = AiConfig::default();
        a.provider = "openai".to_string();
        assert_eq!(a.effective_model(), "gpt-4o");
        a.provider = "ollama".to_string();
        assert_eq!(a.effective_model(), "llama3.1");
        a.provider = "anthropic".to_string();
        assert_eq!(a.effective_model(), "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_ai_effective_model_custom() {
        let a = AiConfig {
            model: Some("my-model".to_string()),
            ..AiConfig::default()
        };
        assert_eq!(a.effective_model(), "my-model");
    }

    // ── AiConfig::is_ready ──────────────────────────────────────────
    #[test]
    fn test_ai_not_ready_by_default() {
        let a = AiConfig::default();
        assert!(!a.is_ready());
    }

    #[test]
    fn test_ai_ready_when_fully_configured() {
        assert!(bedrock_config().is_ready());
    }

    #[test]
    fn test_ai_not_ready_if_disabled() {
        let mut a = bedrock_config();
        a.enabled = false;
        assert!(!a.is_ready());
    }

    #[test]
    fn test_ai_not_ready_missing_endpoint() {
        std::env::remove_var("ZIT_AI_ENDPOINT");
        let a = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: None,
            api_key: Some("key12345".to_string()),
            timeout_secs: Some(30),
        };
        assert!(!a.is_ready());
    }

    #[test]
    fn test_ai_not_ready_missing_key() {
        std::env::remove_var("ZIT_AI_API_KEY");
        let a = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("https://api.example.com".to_string()),
            api_key: None,
            timeout_secs: Some(30),
        };
        assert!(!a.is_ready());
    }

    #[test]
    fn test_ollama_ready_without_key() {
        std::env::remove_var("ZIT_AI_API_KEY");
        let a = AiConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: None,
            endpoint: None, // will default to localhost
            api_key: None,
            timeout_secs: Some(30),
        };
        assert!(a.is_ready());
    }

    #[test]
    fn test_openai_ready_with_key_only() {
        let a = AiConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: None,
            endpoint: None, // has built-in default
            api_key: Some("sk-test12345678".to_string()),
            timeout_secs: Some(30),
        };
        assert!(a.is_ready());
    }

    // ── AiConfig::validate ──────────────────────────────────────────
    #[test]
    fn test_validate_disabled_returns_no_issues() {
        let a = AiConfig::default();
        assert!(a.validate().is_empty());
    }

    #[test]
    fn test_validate_bedrock_no_endpoint() {
        std::env::remove_var("ZIT_AI_ENDPOINT");
        let a = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: None,
            api_key: Some("test-key-1234".to_string()),
            timeout_secs: Some(30),
        };
        let issues = a.validate();
        assert!(issues.iter().any(|i| i.contains("endpoint")));
    }

    #[test]
    fn test_validate_bad_endpoint_scheme() {
        let a = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("ftp://example.com".to_string()),
            api_key: Some("test-key-1234".to_string()),
            timeout_secs: Some(30),
        };
        let issues = a.validate();
        assert!(issues
            .iter()
            .any(|i| i.contains("must start with https://")));
    }

    #[test]
    fn test_validate_short_api_key() {
        let a = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("https://api.example.com".to_string()),
            api_key: Some("abc".to_string()),
            timeout_secs: Some(30),
        };
        let issues = a.validate();
        assert!(issues.iter().any(|i| i.contains("too short")));
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut a = bedrock_config();
        a.timeout_secs = Some(0);
        let issues = a.validate();
        assert!(issues.iter().any(|i| i.contains("must be > 0")));
    }

    #[test]
    fn test_validate_high_timeout() {
        let mut a = bedrock_config();
        a.timeout_secs = Some(300);
        let issues = a.validate();
        assert!(issues.iter().any(|i| i.contains("unusually high")));
    }

    #[test]
    fn test_validate_all_good() {
        assert!(bedrock_config().validate().is_empty());
    }

    #[test]
    fn test_validate_unknown_provider() {
        let a = AiConfig {
            enabled: true,
            provider: "unknown".to_string(),
            model: None,
            endpoint: None,
            api_key: None,
            timeout_secs: None,
        };
        let issues = a.validate();
        assert!(issues.iter().any(|i| i.contains("Unknown AI provider")));
    }

    #[test]
    fn test_validate_ollama_no_key_ok() {
        std::env::remove_var("ZIT_AI_API_KEY");
        let a = AiConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: None,
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
        };
        assert!(a.validate().is_empty());
    }

    // ── GithubConfig ────────────────────────────────────────────────
    #[test]
    fn test_github_config_no_token() {
        let g = GithubConfig::default();
        assert!(g.get_token().is_none());
    }

    #[test]
    fn test_github_config_prefers_oauth() {
        let g = GithubConfig {
            pat: Some("pat-token".to_string()),
            oauth_token: Some("oauth-token".to_string()),
            username: None,
        };
        assert_eq!(g.get_token(), Some("oauth-token".to_string()));
    }

    #[test]
    fn test_github_config_falls_back_to_pat() {
        let g = GithubConfig {
            pat: Some("pat-token".to_string()),
            oauth_token: None,
            username: None,
        };
        assert_eq!(g.get_token(), Some("pat-token".to_string()));
    }

    // ── Config serialization roundtrip ──────────────────────────────
    #[test]
    fn test_config_toml_roundtrip() {
        let config = Config {
            general: GeneralConfig {
                tick_rate_ms: 500,
                confirm_destructive: false,
            },
            github: GithubConfig {
                pat: Some("ghp_test".to_string()),
                oauth_token: None,
                username: Some("user".to_string()),
            },
            ui: UiConfig {
                color_scheme: "dark".to_string(),
                show_help_hints: false,
            },
            ai: AiConfig {
                enabled: true,
                provider: "openai".to_string(),
                model: Some("gpt-4o".to_string()),
                endpoint: None,
                api_key: Some("key123456".to_string()),
                timeout_secs: Some(60),
            },
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.general.tick_rate_ms, 500);
        assert!(!parsed.general.confirm_destructive);
        assert_eq!(parsed.github.pat, Some("ghp_test".to_string()));
        assert_eq!(parsed.ui.color_scheme, "dark");
        assert!(parsed.ai.enabled);
        assert_eq!(parsed.ai.provider, "openai");
        assert_eq!(parsed.ai.model, Some("gpt-4o".to_string()));
    }

    // ── Config::default has expected values ──────────────────────────
    #[test]
    fn test_config_default_has_expected_values() {
        let config = Config::default();
        assert_eq!(config.general.tick_rate_ms, 2000);
        assert!(config.general.confirm_destructive);
        assert!(!config.ai.enabled);
        assert_eq!(config.ai.provider, "bedrock");
        assert_eq!(config.ui.color_scheme, "default");
    }

    // ── AiConfig::resolved_endpoint env fallback ────────────────────
    #[test]
    fn test_resolved_endpoint_prefers_config() {
        std::env::set_var("ZIT_AI_ENDPOINT", "https://env.example.com");
        let a = AiConfig {
            endpoint: Some("https://config.example.com".to_string()),
            ..AiConfig::default()
        };
        assert_eq!(a.resolved_endpoint().unwrap(), "https://config.example.com");
        std::env::remove_var("ZIT_AI_ENDPOINT");
    }

    #[test]
    fn test_resolved_endpoint_env_fallback() {
        std::env::set_var("ZIT_AI_ENDPOINT", "https://env.example.com");
        let a = AiConfig {
            endpoint: None,
            ..AiConfig::default()
        };
        assert_eq!(a.resolved_endpoint().unwrap(), "https://env.example.com");
        std::env::remove_var("ZIT_AI_ENDPOINT");
    }

    #[test]
    fn test_effective_endpoint_provider_defaults() {
        std::env::remove_var("ZIT_AI_ENDPOINT");
        let mut a = AiConfig::default();
        a.provider = "openai".to_string();
        assert!(a.effective_endpoint().unwrap().contains("openai.com"));
        a.provider = "ollama".to_string();
        assert!(a.effective_endpoint().unwrap().contains("localhost:11434"));
        a.provider = "bedrock".to_string();
        assert!(a.effective_endpoint().is_none()); // bedrock requires explicit
    }
}

