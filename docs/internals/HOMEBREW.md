# Homebrew (Package Distribution)

## What It Is
Homebrew is the most popular package manager for macOS (and Linux). A "Formula" is a Ruby script that tells Homebrew how to download and build a package.

## Why My Project Uses It
We want macOS users to install zit with a single command: `brew tap JUSTMEETPATEL/zit && brew install zit`. No manual Rust toolchain setup needed.

## Where It Appears in My Project
- `Formula/zit.rb` (21 lines) — The Homebrew formula

## How It Works Internally
1. `brew tap JUSTMEETPATEL/zit` adds our tap (a GitHub repo containing formulas)
2. `brew install zit` downloads the source tarball from GitHub Releases, compiles with `cargo install`, and symlinks the binary to `/usr/local/bin/zit`
3. The formula declares `depends_on "rust" => :build` (build-time only) and `depends_on "git"` (runtime)
4. The test block verifies the binary runs and detects a non-git-repo error

## How My Code Uses It (Annotated)
```ruby
# Formula/zit.rb
class Zit < Formula
  desc "A TUI-based Git dashboard for efficient repository management"
  homepage "https://github.com/JUSTMEETPATEL/zit"
  url "https://github.com/JUSTMEETPATEL/zit/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "8a7ac6a5cbdda396..."  # Integrity check
  license "MIT"
  depends_on "rust" => :build   # Need Rust to compile, not at runtime
  depends_on "git"              # Runtime dependency
  def install
    system "cargo", "install", *std_cargo_args  # Build + install
  end
  test do
    assert_match "Not a git repository", shell_output("#{bin}/zit 2>&1", 1)
  end
end
```

## What Could Go Wrong
- **Stale SHA256**: When releasing a new version, the tarball URL and SHA256 must be updated
- **Compilation time**: Building from source takes 1-2 minutes on first install
- **Rust version**: If the user's Homebrew Rust is outdated, compilation may fail with syntax errors

## Judge-Ready One-Liner
"Users install zit with one command — `brew install zit` — just like any major CLI tool, giving us proper distribution that rivals established tools."
