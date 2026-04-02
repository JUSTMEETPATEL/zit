# Serde + serde_json + TOML

## What It Is
Serde is Rust's serialization framework. `serde_json` handles JSON encoding/decoding; `toml` handles TOML config file parsing. Together they convert between Rust structs and data formats.

## Why My Project Uses It
Zit needs to: (1) serialize AI requests to JSON for HTTP APIs; (2) deserialize JSON responses from GitHub and AI providers; (3) read/write the `~/.config/zit/config.toml` configuration file.

## Where It Appears in My Project
- `Cargo.toml` lines 19–21: `serde = { features = ["derive"] }`, `serde_json = "1"`, `toml = "1"`
- `src/config.rs` lines 5–15: `Config`, `GeneralConfig`, `GithubConfig`, `UiConfig`, `AiConfig` — all derive `Serialize, Deserialize`
- `src/ai/client.rs` lines 31–89: `MentorRequest`, `RepoContext`, `MentorApiResponse` — JSON request/response types
- `src/ai/provider.rs`: Provider-specific request/response structs (OpenAI, Anthropic, Ollama)
- `src/git/github_auth.rs` lines 11–55: `DeviceCodeResponse`, `TokenResponse`, `PullRequest`, etc.

## How It Works Internally
1. `#[derive(Serialize, Deserialize)]` generates serialization code at compile time — zero runtime reflection.
2. `#[serde(rename = "type")]` maps Rust field `request_type` to JSON key `"type"` (since `type` is a reserved word in Rust).
3. `#[serde(skip_serializing_if = "Option::is_none")]` omits `null` fields from JSON output.
4. `#[serde(default)]` uses `Default::default()` when a field is missing during deserialization.
5. `toml::from_str(&content)` parses TOML into a `Config` struct; `toml::to_string_pretty(&config)` writes it back.

## Key Concepts I Must Know
- **Zero-cost abstractions**: Serde generates optimal code at compile time — no runtime overhead
- **Derive macros**: `#[derive(Serialize, Deserialize)]` auto-implements the traits
- **Field attributes**: `rename`, `skip_serializing_if`, `default`, `skip` control serialization behavior
- **Deserialization is fallible**: `serde_json::from_str::<T>()` returns `Result` — invalid JSON produces a descriptive error
- **TOML for config**: Human-readable, minimal syntax — better than JSON for config files

## How My Code Uses It (Annotated)
```rust
// src/ai/client.rs:31-41 — A request sent to the AI backend
#[derive(Debug, Serialize)]
pub struct MentorRequest {
    #[serde(rename = "type")]                    // JSON: {"type": "explain"}
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Omit if None
    pub context: Option<RepoContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// src/config.rs:335-344 — Loading the config file
pub fn load() -> Result<Self> {
    let path = Self::path();             // ~/.config/zit/config.toml
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?; // TOML → Rust struct
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
```

## What Could Go Wrong
- **Config file syntax errors**: If the user manually edits `config.toml` and introduces a syntax error, `toml::from_str` will fail. We handle this with `unwrap_or_default()`.
- **Missing fields**: If new config fields are added in a newer version, old config files won't have them. `#[serde(default)]` handles this gracefully.
- **Type mismatches in API responses**: If GitHub changes their API response format, deserialization will fail with an error.

## Judge-Ready One-Liner
"Serde is Rust's universal serialization framework — it turns our Rust structs into JSON for API calls and parses TOML config files, all with zero runtime overhead thanks to compile-time code generation."
