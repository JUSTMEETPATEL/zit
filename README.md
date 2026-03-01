# zit

> **zit** is an AI-powered, terminal-based Git and GitHub assistant built in Rust. It combines a rich TUI with intelligent AI mentorship to make Git accessible, safe, and educational.

[![License](https://img.shields.io/github/license/JUSTMEETPATEL/zit)](LICENSE)
[![Rust](https://img.shields.io/badge/built_with-Rust-orange)](https://www.rust-lang.org/)
[![CI](https://img.shields.io/github/actions/workflow/status/JUSTMEETPATEL/zit/ci.yml?label=CI)](https://github.com/JUSTMEETPATEL/zit/actions)
[![Release](https://img.shields.io/github/v/release/JUSTMEETPATEL/zit)](https://github.com/JUSTMEETPATEL/zit/releases)

## Features

- **Repository Dashboard**: At-a-glance status of your repo — branch, dirty state, recent commits (`d`)
- **Smart Staging**: Interactive file staging with diff previews and search (`s`)
- **Guided Commits**: Commit editor with validation, history lookup, and AI suggestions (`c`)
- **Visual Branching**: Create, switch, delete, rename branches visually (`b`)
- **Commit Timeline**: Browse git log with a visual commit graph (`l`)
- **Time Travel**: Safe reset/restore with confirmation dialogs (`t`)
- **Reflog Recovery**: Browse and recover "lost" commits from the reflog (`r`)
- **GitHub Integration**: OAuth device flow, repo creation, push/pull/sync, collaborators (`g`)
- **🤖 AI Mentor**: AI-powered assistant for explanations, recommendations, and error help (`a`)

## Installation

### macOS (Homebrew)

```bash
brew tap JUSTMEETPATEL/zit
brew install zit
```

### From source (Linux / macOS / Windows)

```bash
cargo install --git https://github.com/JUSTMEETPATEL/zit
```

**Prerequisites**: [Rust](https://rustup.rs), `git`, a modern terminal with TrueColor support.
Windows users also need [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++" workload).

## Usage

```bash
cd my-repo
zit
```

### Keybindings

| Key | Action |
|-----|--------|
| `s` | **Staging** — interactive file staging with diffs |
| `c` | **Commit** — write and submit commits |
| `b` | **Branches** — create, switch, delete, rename |
| `l` | **Log** — visual commit timeline / graph |
| `t` | **Time Travel** — reset / restore safely |
| `r` | **Reflog** — recover lost commits |
| `g` | **GitHub** — sync, push/pull, collaborators |
| `a` | **AI Mentor** — explain repo, ask questions, get recommendations |
| `?` | **Help** — context-sensitive keybinding reference |
| `q` | **Quit** |

### AI Mentor

The AI Mentor panel (`a` from the dashboard) provides four capabilities:

| Feature | Description |
|---------|-------------|
| 🔍 Explain Repo | AI explains your current repository state |
| 💬 Ask a Question | Ask anything about git — get a plain-English answer |
| 🛡️ Recommend | Get safe recommendations for git operations |
| 🏥 Health Check | Test connectivity to the AI backend |

Additional AI features work automatically:
- **Ctrl+G** in the Commit view generates an AI commit message from your staged diff
- **Auto Error Explainer** — when a git command fails (stage, unstage, reset, branch delete), the AI automatically explains the error and suggests fixes

### AI Setup

The AI features require an AWS Lambda backend. See [aws/README.md](aws/README.md) for deployment instructions.

Once deployed, configure zit:

**Option A — Config file** (`~/.config/zit/config.toml`):

```toml
[ai]
enabled = true
endpoint = "https://your-api.execute-api.region.amazonaws.com/dev/mentor"
api_key = "your-api-key"
timeout_secs = 30
```

**Option B — Environment variables**:

```bash
export ZIT_AI_ENDPOINT="https://your-api.execute-api.region.amazonaws.com/dev/mentor"
export ZIT_AI_API_KEY="your-api-key"
```

> AI is optional — all core features work without it. When AI is not configured, the Mentor panel shows setup instructions.

## Configuration

Zit reads config from `~/.config/zit/config.toml`:

```toml
[general]
tick_rate_ms = 2000          # UI refresh interval
confirm_destructive = true   # Require confirmation for risky operations

[ui]
color_scheme = "default"
show_help_hints = true

[github]
# pat = "ghp_..."           # Or use OAuth device flow from the GitHub view

[ai]
enabled = true
endpoint = "https://..."
api_key = "..."
timeout_secs = 30
```

## Architecture

```
zit (Rust TUI)
├── ratatui + crossterm      — Terminal UI rendering
├── Git CLI (shell)          — All git operations via native git
├── reqwest (blocking)       — HTTP for GitHub API + AI backend
└── AI Client                — Background thread + mpsc channel
    └── AWS Lambda (Python 3.12)
        └── Amazon Bedrock (Claude 3 Sonnet)
```

**Key design decisions**:
- **Shell-based Git**: Runs real `git` commands — never reimplements git internals
- **AI is optional**: Degrades gracefully to static help when AI is unavailable
- **Non-blocking AI**: All AI calls run in background threads to keep the TUI responsive
- **Retry with backoff**: AI client retries transient failures (2 retries, exponential backoff)

## Development

```bash
# Build
cargo build

# Run checks (format + clippy + test)
make check

# Run tests
cargo test --all-targets        # 13 Rust tests
cd aws && python3 -m pytest tests/ -v   # 27 Lambda tests

# Lint
cargo clippy --all-targets -- -D warnings

# Release build
cargo build --release
```

## Project Structure

```
src/
├── main.rs          # Entry point, terminal setup, render loop
├── app.rs           # App state, view routing, async AI dispatch
├── config.rs        # Config loading (~/.config/zit/config.toml)
├── event.rs         # Keyboard/tick event handling
├── ai/
│   └── client.rs    # AI client (retry, error classification, background threads)
├── git/
│   ├── runner.rs    # Core git command executor
│   ├── status.rs    # git status parser
│   ├── diff.rs      # git diff parser
│   ├── log.rs       # git log parser with graph support
│   ├── branch.rs    # Branch operations
│   ├── remote.rs    # Remote/push/pull operations
│   ├── reflog.rs    # Reflog parser
│   └── github_auth.rs  # GitHub OAuth device flow
└── ui/
    ├── dashboard.rs  # Repository dashboard view
    ├── staging.rs    # Interactive staging view
    ├── commit.rs     # Commit editor view
    ├── branches.rs   # Branch manager view
    ├── timeline.rs   # Commit log/graph view
    ├── time_travel.rs # Reset/restore view
    ├── reflog.rs     # Reflog viewer
    ├── github.rs     # GitHub integration view
    ├── ai_mentor.rs  # AI Mentor panel (menu, input, result)
    └── help.rs       # Context-sensitive help overlay
aws/
├── deploy.sh        # One-command deployment script
├── lambda/
│   ├── handler.py   # Lambda function (Bedrock integration)
│   └── prompts.py   # AI system prompts per request type
└── infrastructure/
    └── template.yaml # SAM/CloudFormation template
```

## Troubleshooting

### Windows: `linker link.exe not found`

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **"Desktop development with C++"** workload.

### AI not working

1. Check connectivity: use Health Check in the AI Mentor panel (`a` → select Health Check)
2. Verify config: `cat ~/.config/zit/config.toml` — ensure `[ai]` section is present
3. Check env vars: `echo $ZIT_AI_ENDPOINT $ZIT_AI_API_KEY`
4. Check Lambda logs: `aws logs tail /aws/lambda/zit-ai-mentor-dev --region ap-south-1`

## Contributing

Contributions are welcome! Please read the [contributing guidelines](CONTRIBUTING.md) before submitting a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
