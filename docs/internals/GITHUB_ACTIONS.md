# GitHub Actions (CI/CD)

## What It Is
GitHub Actions is a CI/CD platform that runs automated workflows triggered by events (push, PR, schedule).

## Why My Project Uses It
Every push and PR triggers formatting checks, linting, testing, and cross-platform builds. This ensures code quality and catches regressions before merging.

## Where It Appears in My Project
- `.github/workflows/ci.yml` (55 lines) — The CI pipeline
- `.github/dependabot.yml` (662 bytes) — Automated dependency updates
- `.github/FUNDING.yml` (183 bytes) — GitHub Sponsors configuration

## How It Works Internally
1. On every push to `main` or PR against `main`, two jobs run in parallel:
   - **Rust CI**: Matrix build across `ubuntu-latest`, `macos-latest`, `windows-latest`
     - Install stable Rust toolchain with rustfmt + clippy
     - Cache Cargo dependencies (Swatinem/rust-cache)
     - `cargo fmt --check` — enforce formatting
     - `cargo clippy -- -D warnings` — lint with warnings-as-errors
     - `cargo test --all-targets` — run all 178 tests
   - **Lambda Tests**: Ubuntu only, Python 3.12
     - Install pip dependencies
     - `pytest tests/ -v` — run 27 Lambda tests
2. Tests run on all 3 platforms simultaneously (~5 min total)

## Key Concepts I Must Know
- **Matrix strategy**: Tests run on 3 OSes in parallel to catch platform-specific bugs
- **Warnings as errors**: `clippy -- -D warnings` fails the build on any warning
- **Cargo caching**: `Swatinem/rust-cache@v2` caches compiled dependencies to speed up builds
- **178 Rust tests**: 143 unit tests (embedded in source) + 35 integration tests (tests/integration.rs)
- **27 Lambda tests**: Cover request validation, handler functions, health check, CORS, prompts

## How My Code Uses It (Annotated)
```yaml
# .github/workflows/ci.yml — The CI pipeline
name: CI
on:
  push: { branches: [main] }     # Run on push to main
  pull_request: { branches: [main] }  # Run on PRs to main
jobs:
  rust:
    runs-on: ${{ matrix.os }}    # Run on 3 OSes
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable  # Install Rust
      - uses: Swatinem/rust-cache@v2         # Cache dependencies
      - run: cargo fmt --all -- --check      # Format check
      - run: cargo clippy --all-targets -- -D warnings  # Lint
      - run: cargo test --all-targets        # Test
```

## What Could Go Wrong
- **CI passes but local fails**: Different git versions or OS-specific behavior. Matrix testing mitigates this.
- **Flaky tests**: Network-dependent tests (if any) could fail intermittently. Our tests are all unit/integration with no network calls.
- **Cache invalidation**: Dependency updates can cause cache misses, slowing builds temporarily.

## Judge-Ready One-Liner
"Every push triggers automated formatting, linting, and 205 tests across Linux, macOS, and Windows — we have zero manual QA steps."
