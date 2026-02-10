# zit

> **zit** (Git TUI) is a terminal-based interface for managing git repositories with efficiency and style.

![License](https://img.shields.io/github/license/JUSTMEETPATEL/zit)
![Rust](https://img.shields.io/badge/built_with-Rust-orange)

## Features

- **Repository Dashboard**: At-a-glance status of your repo (branch, dirty state, recent commits).
- **Smart Staging**: Interactive file staging with diff previews (`s` key).
- **Guided Commits**: Commit editor with validation and history lookup (`c` key).
- **Visual Branching**: Manage branches visually (`b` key).
- **Time Travel**: Safe reset/restore with confirmation (`t` key).
- **GitHub Integration**: 
  - Authenticate securely via OAuth Device Flow.
  - Create repositories.
  - **Push/Pull/Sync** in background.
  - Manage collaborators directly from the terminal.

## Installation

### From Source (Rust/Cargo)

```bash
cargo install --path .
```

*Note: Homebrew support will be available soon once the formula is added to the official core or a custom tap.*

## Usage

Run `zit` in any git repository:

```bash
cd my-repo
zit
```

### Keybindings

| Key | Action |
| --- | --- |
| `s` | **Stage** files (interactive staging area) |
| `c` | **Commit** changes |
| `b` | Manage **Branches** |
| `l` | View Commit **Log** |
| `t` | **Time Travel** (Reset/Restore) |
| `r` | View **Reflog** |
| `g` | **GitHub** Integration (Sync, PRs, Collaborators) |
| `?` | Help |
| `q` | Quit |

## License

MIT
