# TOML Configuration System

## What It Is
TOML (Tom's Obvious Minimal Language) is a human-readable config file format. Zit uses `~/.config/zit/config.toml` for persistent user settings.

## Why My Project Uses It
Users need to persist settings (AI config, GitHub tokens, UI preferences) between sessions. TOML is more readable than JSON and more structured than .env files.

## Where It Appears in My Project
- `Cargo.toml` line 21: `toml = "1"`
- `src/config.rs` (739 lines) — Complete config system with 4 sections: `[general]`, `[github]`, `[ui]`, `[ai]`
- `src/main.rs` line 107: `config::Config::load().unwrap_or_default()`

## How It Works Internally
1. On startup, `Config::load()` reads `~/.config/zit/config.toml`
2. Serde deserializes the TOML into nested Rust structs: `Config → GeneralConfig + GithubConfig + UiConfig + AiConfig`
3. Every field has a `#[serde(default)]` attribute so missing fields use sensible defaults
4. On save, `config.save()` serializes back to `toml::to_string_pretty()` and writes with `0o600` permissions (Unix)
5. Config path uses `dirs::config_dir()` — XDG-compliant on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows

## Key Concepts I Must Know
- **Default tick rate**: 2000ms — how often the UI auto-refreshes
- **`confirm_destructive`**: When true, hard reset/branch delete require y/n confirmation
- **AI config sections**: `enabled`, `provider`, `model`, `endpoint`, `api_key`, `timeout_secs`
- **Token migration**: On first run, plaintext tokens are moved from config to OS keychain (keychain.rs:111)
- **Environment variable fallback**: `ZIT_AI_ENDPOINT` and `ZIT_AI_API_KEY` override config file values

## How My Code Uses It (Annotated)
```rust
// src/config.rs:5-15 — The top-level config struct
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,   // tick_rate_ms, confirm_destructive
    #[serde(default)]
    pub github: GithubConfig,     // pat, oauth_token, username
    #[serde(default)]
    pub ui: UiConfig,             // color_scheme, show_help_hints
    #[serde(default)]
    pub ai: AiConfig,             // enabled, provider, model, endpoint, api_key, timeout_secs
}
```

## What Could Go Wrong
- **File permissions**: If another process reads `config.toml`, tokens could be exposed (mitigated by 0o600 permissions + keychain migration)
- **Manual edit errors**: If the user introduces a TOML syntax error, the config fails to load. We fall back to defaults with `unwrap_or_default()`.

## Judge-Ready One-Liner
"All user preferences — AI provider, GitHub auth, UI settings — persist in a single human-readable TOML file at `~/.config/zit/config.toml`, with automatic OS keychain migration for sensitive tokens."
