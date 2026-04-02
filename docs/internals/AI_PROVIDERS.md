# Multi-Provider AI Architecture

## What It Is
Zit supports 5 AI providers through a trait-based abstraction: Amazon Bedrock (via Lambda), OpenAI, Anthropic, OpenRouter, and Ollama (local). Users choose their provider in an interactive setup wizard.

## Why My Project Uses It
Not every user has AWS access. By supporting multiple providers, zit works with any popular AI service — including fully local inference via Ollama. This makes the AI features accessible to everyone.

## Where It Appears in My Project
- `src/ai/provider.rs` (686 lines) — `AiProvider` trait + 4 implementations
- `src/ai/client.rs` (1065 lines) — `AiClient` with caching, retry, error classification
- `src/ai/prompts.rs` (486 lines) — 9 system prompts for different request types
- `src/ai/mod.rs` (7 lines) — Module definition with `DIFF_TRUNCATE_AT` constant
- `src/config.rs` lines 88–323: `AiConfig` with provider-aware endpoint/model resolution
- `src/app.rs` lines 696–843: Interactive AI setup wizard (5 providers)

## How It Works Internally
1. **Provider trait** (`AiProvider`): Defines `chat(system_prompt, user_message) → String` and `health_check()`.
2. **Factory pattern** (`create_provider(config)`): Reads config to instantiate the correct provider.
3. **Two code paths**: Bedrock sends raw JSON to Lambda (Lambda constructs prompts). All other providers build prompts client-side via `prompts.rs`.
4. **Response caching**: `AiClient` caches responses for 5 minutes (keyed by request type + query + branch). Max 50 entries.
5. **Retry with backoff**: 2 retries with exponential delay (500ms, 1s). Only retries transient errors (5xx, timeout, DNS).
6. **Error classification**: Distinguishes `timeout`, `connect`, `decode`, `401`, `403`, `429`, `5xx` with user-friendly messages.
7. **Non-blocking**: All AI calls spawn a `std::thread`, send results through `mpsc::channel`, and get polled in the main loop.

## Key Concepts I Must Know
- **Provider defaults**: OpenAI→`gpt-4o`, Anthropic→`claude-sonnet-4-20250514`, OpenRouter→`anthropic/claude-sonnet-4`, Ollama→`llama3.1`, Bedrock→`claude-3-sonnet`
- **Endpoint resolution**: Each provider has a default endpoint; Ollama defaults to `localhost:11434`; Bedrock requires explicit Lambda URL
- **Stale endpoint detection** (config.rs:169–182): If you switch from Bedrock to OpenRouter, the old Lambda URL is automatically ignored
- **Setup wizard** (app.rs:868–875): 3-step interactive prompt (provider → endpoint → API key)
- **9 request types**: explain, error, recommend, commit_suggestion, learn, review, merge_resolve, merge_strategy, generate_gitignore

## How My Code Uses It (Annotated)
```rust
// src/ai/provider.rs:14-27 — The trait all providers implement
pub trait AiProvider: Send + Sync {
    fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String>;
    fn health_check(&self) -> Result<String>;
    fn name(&self) -> &str;       // e.g., "Amazon Bedrock", "Ollama"
    fn model_name(&self) -> &str; // e.g., "gpt-4o"
}

// src/ai/client.rs:224-254 — Dispatch based on provider
fn call(&self, request: &MentorRequest) -> Result<String> {
    let ckey = cache_key(request);
    if let Some(cached) = self.get_cached(&ckey) { return Ok(cached); } // Cache hit
    let result = if self.provider_kind == "bedrock" {
        self.call_bedrock(request)  // Lambda handles prompt construction
    } else {
        self.call_direct(request)   // Client-side prompt construction
    };
    if let Ok(ref response) = result { self.set_cached(ckey, response.clone()); }
    result
}
```

## What Could Go Wrong
- **Provider-specific quirks**: Anthropic uses `x-api-key` header, OpenAI uses `Authorization: Bearer`, OpenRouter needs extra `HTTP-Referer` and `X-Title` headers
- **Model availability**: If a model is deprecated or renamed, requests fail. Users must update their config.
- **Ollama not running**: If `ollama serve` isn't running, connection fails immediately. The error message guides the user.
- **Cache staleness**: Cached responses persist for 5 minutes — if repo state changes, the cache may return outdated info

## Judge-Ready One-Liner
"Our AI system supports 5 providers — AWS Bedrock, OpenAI, Anthropic, OpenRouter, and local Ollama — through a pluggable trait-based architecture, so users can choose based on their preferences, budget, or privacy requirements."
