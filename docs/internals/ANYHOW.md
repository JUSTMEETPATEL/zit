# Anyhow (Error Handling)

## What It Is
Anyhow is a Rust library for idiomatic error handling in applications. It provides `anyhow::Result<T>` (which is `Result<T, anyhow::Error>`) and the `bail!()` / `context()` macros.

## Why My Project Uses It
Zit needs rich error messages with context (e.g., "Failed to execute git command" wrapping an IO error). Anyhow makes this ergonomic without defining custom error types for every module.

## Where It Appears in My Project
- `Cargo.toml` line 25: `anyhow = "1"`
- Every `.rs` file that returns `Result` uses `anyhow::Result`
- `src/git/runner.rs` lines 1, 35, 45–49: `bail!()` for git failures and timeouts
- `src/ai/client.rs` throughout: `context()` for wrapping HTTP errors
- `src/main.rs` line 56: `fn main() -> Result<()>`

## Key Concepts I Must Know
- **`Result<T>` shorthand**: `anyhow::Result<T>` = `Result<T, anyhow::Error>` — no need to define custom error types
- **`bail!("message")`**: Immediately returns an `Err` — like `return Err(anyhow!("message"))`
- **`.context("msg")?`**: Adds context to an error — produces a chain like "Failed to parse: invalid JSON"
- **`?` operator**: Propagates errors up the call stack, automatically converting to `anyhow::Error`
- **Not for libraries**: Anyhow is for applications, not libraries. Libraries should use `thiserror` for typed errors.

## How My Code Uses It (Annotated)
```rust
// src/git/runner.rs:10,32-36
pub fn run_git(args: &[&str]) -> Result<String> {  // Returns anyhow::Result
    let mut child = Command::new("git")
        .spawn()
        .context("Failed to execute git command")?;  // Adds context to IO error
    if !status.success() {
        bail!("git {} failed: {}", args.join(" "), stderr.trim()); // Early return with error
    }
}
```

## What Could Go Wrong
- **Error message quality depends on the developer**: Anyhow doesn't force structured errors, so messages may be inconsistent.

## Judge-Ready One-Liner
"Anyhow gives us Go-style error handling in Rust — every error carries full context about what went wrong and where, making debugging trivial."
