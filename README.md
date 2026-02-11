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

### macOS (via Homebrew)

You can install `zit` easily using our official Homebrew tap:

```bash
Sbrew tap JUSTMEETPATEL/zit
brew install zit
```

*Note: This installs the stable release.*

### Linux & Windows

Currently, the best way to install `zit` on Linux and Windows is via Cargo (Rust's package manager).

```bash
cargo install --git https://github.com/JUSTMEETPATEL/zit
```

*Prerequisites:*

- **Rust/Cargo**: Install via [rustup.rs](https://rustup.rs).
- **Git**: Ensure `git` is installed and in your PATH.
- **Terminal**: Use a modern terminal with TrueColor support (e.g., Windows Terminal, Alacritty, GNOME Terminal).
- **Windows Only**: You must install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **"Desktop development with C++"** workload to compile Rust.

## Usage

Run `zit` in any git repository:

```bash
cd my-repo
zit
```

### Keybindings

| Key   | Action                                                  |
| ----- | ------------------------------------------------------- |
| `s` | **Stage** files (interactive staging area)        |
| `c` | **Commit** changes                                |
| `b` | Manage**Branches**                                |
| `l` | View Commit**Log**                                |
| `t` | **Time Travel** (Reset/Restore)                   |
| `r` | View**Reflog**                                    |
| `g` | **GitHub** Integration (Sync, PRs, Collaborators) |
| `?` | Help                                                    |
| `q` | Quit                                                    |

## License

MIT
