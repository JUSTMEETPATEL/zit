# Keyring (OS Keychain Integration)

## What It Is
The `keyring` crate provides cross-platform access to the operating system's secret storage: macOS Keychain, Windows Credential Manager, and Linux Secret Service (GNOME Keyring/KDE Wallet).

## Why My Project Uses It
Zit stores sensitive tokens (GitHub OAuth tokens, GitHub PATs, AI API keys) securely in the OS keychain instead of plaintext config files. This is a security best practice.

## Where It Appears in My Project
- `Cargo.toml` line 31: `keyring = { version = "3", features = ["apple-native", "windows-native", "linux-native"] }`
- `src/keychain.rs` (156 lines) — Complete keychain abstraction layer
- `src/main.rs` lines 117–124: One-time migration of plaintext tokens to keychain on startup
- `src/config.rs` lines 37–48: `GithubConfig::get_token()` tries keychain first

## How It Works Internally
1. `keyring::Entry::new("zit-cli", "github-oauth-token")` creates a reference to a keychain entry.
2. `entry.set_password(token)` stores the token securely (encrypted by the OS).
3. `entry.get_password()` retrieves the token. Returns `Err(NoEntry)` if not found.
4. On macOS, this uses the Keychain Access API. On Windows, the Credential Manager. On Linux, D-Bus Secret Service.
5. **Migration flow** (main.rs:117–124): On first run, if plaintext tokens exist in `config.toml`, they're moved to the keychain and removed from the file.

## Key Concepts I Must Know
- **Service name**: `"zit-cli"` — identifies our app in the keychain
- **Key names**: `"github-oauth-token"`, `"github-pat"`, `"ai-api-key"` — three secret types
- **Fallback chain**: Keychain → config file → environment variable
- **Graceful degradation**: If keychain is unavailable (e.g., headless Linux server), tokens stay in the config file
- **Test isolation**: In `#[cfg(test)]`, `get_secret()` always returns `None` to avoid touching the real keychain during tests

## How My Code Uses It (Annotated)
```rust
// src/keychain.rs:111-142 — One-time migration from config to keychain
pub fn migrate_from_config(config: &mut Config) -> u32 {
    let mut count = 0;
    if let Some(ref token) = config.github.oauth_token {
        if store_github_token(token).is_ok() {  // Store in keychain
            config.github.oauth_token = None;   // Remove from config file
            count += 1;
        }
    }
    // ... same for PAT and AI API key
    count  // Returns number of secrets migrated
}
```

## What Could Go Wrong
- **Locked keychain**: On macOS, the keychain can be locked. The user may see a system prompt to unlock it.
- **Headless servers**: Linux servers without a graphical login may not have Secret Service running.
- **Permissions**: On some Linux distros, the binary needs D-Bus access to reach the Secret Service.
- **Config file permissions**: After migration, `config.rs:357–362` sets file permissions to `0o600` (owner-only) on Unix.

## Judge-Ready One-Liner
"We store all sensitive tokens — GitHub OAuth, PATs, and AI API keys — in the OS keychain (macOS Keychain, Windows Credential Manager, or Linux Secret Service) instead of plaintext config files, following security best practices."
