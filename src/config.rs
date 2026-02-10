use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub github: GithubConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubConfig {
    #[serde(default)]
    pub pat: Option<String>,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

impl GithubConfig {
    /// Get the best available token (OAuth preferred over PAT).
    pub fn get_token(&self) -> Option<&str> {
        self.oauth_token
            .as_deref()
            .or(self.pat.as_deref())
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

impl Default for GithubConfig {
    fn default() -> Self {
        Self {
            pat: None,
            oauth_token: None,
            username: None,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            github: GithubConfig::default(),
            ui: UiConfig::default(),
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
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
