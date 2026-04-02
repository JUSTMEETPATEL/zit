# Reqwest

## What It Is
Reqwest is a high-level HTTP client library for Rust. It supports both async and blocking modes, JSON serialization, TLS, timeouts, and more.

## Why My Project Uses It
Zit makes HTTP calls to three services: (1) GitHub API for OAuth, repos, PRs, and Actions; (2) AWS Lambda for the Bedrock AI backend; (3) direct AI providers (OpenAI, Anthropic, OpenRouter, Ollama). Reqwest gives us a clean, type-safe HTTP client with automatic JSON parsing.

## Where It Appears in My Project
- `Cargo.toml` line 16: `reqwest = { version = "0.12", features = ["json", "blocking", "rustls-tls"], default-features = false }`
- `src/ai/client.rs` lines 146–152: Creates a blocking client with timeouts
- `src/ai/client.rs` lines 271–278: Sends POST requests to the AI backend
- `src/ai/provider.rs` lines 49–58, 64–75, 81–91, 94–103: Each provider creates its own client
- `src/git/github_auth.rs` (entire file): All GitHub API calls use reqwest

## How It Works Internally
1. `reqwest::blocking::Client::builder()` creates a client with connection pooling (reuses TCP connections).
2. `.timeout(Duration::from_secs(30))` sets a per-request deadline.
3. `.json(&body)` serializes a Rust struct to JSON using serde and sets `Content-Type: application/json`.
4. `.send()` performs the HTTP request and returns a `Response`.
5. `resp.json::<T>()` deserializes the response body into a Rust type using serde.
6. **Feature flags**: `json` enables JSON support, `blocking` enables synchronous API, `rustls-tls` uses Rust-native TLS (no OpenSSL dependency).

## Key Concepts I Must Know
- **Blocking vs Async**: We use `blocking` mode because ratatui's render loop is synchronous. AI calls run in background `std::thread`s to avoid blocking the UI.
- **rustls-tls**: Uses a pure-Rust TLS implementation — no system OpenSSL needed, which simplifies cross-compilation
- **default-features = false**: Disables the default `native-tls` feature to avoid linking OpenSSL
- **Connection pooling**: The `Client` reuses connections — that's why we create one client and share it
- **Error types**: `reqwest::Error` has methods like `.is_timeout()`, `.is_connect()`, `.is_decode()` for error classification (used in client.rs:714–728)

## How My Code Uses It (Annotated)
```rust
// src/ai/client.rs:271-278 — Sending a request to the AI backend
let send_result = self
    .client                           // Reusable HTTP client with pooled connections
    .post(&self.endpoint)             // POST to the Lambda/AI endpoint
    .header("Content-Type", "application/json")
    .header("x-api-key", &self.api_key) // API key authentication
    .header("x-request-id", Self::request_id()) // Tracing ID
    .json(&body)                      // Serialize MentorRequest to JSON
    .send();                          // Execute the request (blocks this thread)
```

## What Could Go Wrong
- **Timeouts**: AI requests can take 10–30 seconds. The 30s timeout may not be enough for complex prompts on slow connections.
- **TLS certificate issues**: If the system clock is wrong, TLS validation fails. rustls doesn't use system CAs by default (uses webpki-roots).
- **Connection refused**: If the AI endpoint is unreachable, reqwest returns immediately. The retry loop (2 retries with exponential backoff) handles transient failures.

## Judge-Ready One-Liner
"Reqwest is our HTTP client — it handles all API calls to GitHub and AI providers with automatic JSON parsing, connection pooling, and built-in TLS, so we have zero system dependencies."
