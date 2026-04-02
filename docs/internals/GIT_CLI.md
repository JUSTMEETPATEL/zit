# Git CLI (Shell-based Integration)

## What It Is
Git is the distributed version control system. Instead of reimplementing git internals, zit shells out to the real `git` binary on the user's system.

## Why My Project Uses It
By calling the actual `git` command, zit gets 100% compatibility with the user's git configuration, aliases, hooks, and credentials. No need to reimplement git's complex internals in Rust.

## Where It Appears in My Project
- `src/git/runner.rs` (146 lines) — Core `run_git()` function that spawns git processes
- `src/git/status.rs` (426 lines) — Parses `git status --porcelain=v2 --branch`
- `src/git/diff.rs` (10,776 bytes) — Parses `git diff` output (staged, unstaged, hunk-level)
- `src/git/log.rs` (6,740 bytes) — Parses `git log` with graph visualization
- `src/git/branch.rs` (3,080 bytes) — Branch operations (create, delete, rename, switch)
- `src/git/merge.rs` (17,046 bytes) — Merge operations and conflict detection
- `src/git/remote.rs` (3,897 bytes) — Push, pull, fetch operations
- `src/git/stash.rs` (3,572 bytes) — Stash operations
- `src/git/reflog.rs` (6,474 bytes) — Reflog parsing
- `src/git/bisect.rs` (6,755 bytes) — Git bisect operations
- `src/git/cherry_pick.rs` (3,676 bytes) — Cherry-pick operations

## How It Works Internally
1. `run_git(&["status", "--porcelain=v2"])` spawns a child process via `Command::new("git")`.
2. stdout and stderr are captured via pipes.
3. A polling loop (`try_wait()`) checks if the process has finished, with a 30-second timeout.
4. If the process exits non-zero, stderr is captured and returned as an error.
5. If it times out, the process is killed and an error is returned.
6. The output is parsed into Rust types (`RepoStatus`, `FileDiff`, `LogEntry`, etc.).

## Key Concepts I Must Know
- **Porcelain v2 format**: `git status --porcelain=v2 --branch` outputs machine-parseable status. The XY codes (e.g., `M.`, `.M`, `AM`) indicate index vs worktree changes.
- **Shell-based, not library-based**: We don't use libgit2 or gitoxide — we call the `git` binary. This means we depend on git being installed.
- **Minimum version**: Git ≥ 2.13.0 is required for porcelain v2 format (runner.rs:66).
- **Timeout**: All git commands have a 30-second timeout to prevent hangs (runner.rs:6).
- **Error propagation**: Git errors are propagated to the UI and optionally explained by AI.

## How My Code Uses It (Annotated)
```rust
// src/git/runner.rs:10-58 — The core git executor
pub fn run_git(args: &[&str]) -> Result<String> {
    log::debug!("git {}", args.join(" "));     // Log every git command for debugging
    let mut child = Command::new("git")        // Spawn a git process
        .args(args)                            // Pass arguments
        .stdout(Stdio::piped())               // Capture stdout
        .stderr(Stdio::piped())               // Capture stderr
        .spawn()?;
    let start = Instant::now();
    loop {
        match child.try_wait() {              // Non-blocking poll
            Ok(Some(status)) => {             // Process exited
                if !status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    bail!("git {} failed: {}", args.join(" "), stderr.trim());
                }
                return Ok(stdout);            // Return stdout on success
            }
            Ok(None) => {                     // Still running
                if start.elapsed() > timeout {
                    child.kill();             // Kill if timed out
                    bail!("git {} timed out", args.join(" "));
                }
                thread::sleep(50ms);          // Poll every 50ms
            }
        }
    }
}
```

## What Could Go Wrong
- **Git not installed**: If `git` isn't on PATH, every command fails. We check for this at startup (main.rs:93).
- **Old git version**: Git < 2.13.0 doesn't support porcelain v2. We warn but continue.
- **Large repos**: `git status` on huge repos with thousands of files can be slow.
- **Git hooks**: User-defined hooks can hang or produce unexpected output.
- **Locale issues**: Git output in non-English locales could break parsing (we use porcelain format to avoid this).

## Judge-Ready One-Liner
"Instead of reimplementing git in Rust, we shell out to the real git binary — this gives us 100% compatibility with the user's git config, hooks, and credentials while we focus on the intelligent UI layer."
