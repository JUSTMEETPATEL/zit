# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-01

### Added
- **AI Mentor panel** (`a` from dashboard) with four capabilities: Explain Repo, Ask a Question, Recommend, Health Check
- **AI commit suggestions** via `Ctrl+G` in the Commit view — generates messages from staged diffs
- **Auto error explainer** — AI automatically explains git failures (stage/unstage/reset/branch delete) with fix suggestions
- **AI health check** endpoint and client method to verify backend connectivity
- **Non-blocking AI calls** — all AI requests run in background threads via `mpsc` channels
- **Retry with exponential backoff** — AI client retries transient failures (2 retries, 500ms/1s)
- **Error classification** — distinguishes transient (5xx, timeout, DNS) from permanent (4xx) errors
- **Diff truncation** — caps diff content at 4,000 chars to avoid token explosion
- **Request body limit** — Lambda rejects requests > 128 KB
- **Environment variable fallback** — `ZIT_AI_ENDPOINT` and `ZIT_AI_API_KEY` env vars as alternative to config file
- **AWS Lambda backend** (Python 3.12) with Amazon Bedrock (Claude 3 Sonnet) integration
- **SAM/CloudFormation template** with API Gateway, API key auth, usage plan (5,000 req/month)
- **Lambda unit tests** — 27 tests covering request validation, responses, health check, CORS, prompts
- **Lambda CI job** in GitHub Actions (`lambda-test` job, Python 3.12)
- CI/CD pipeline with GitHub Actions (lint, test, build across Linux/macOS/Windows)
- Automated release workflow with cross-compilation and GitHub Releases
- Weekly security audit workflow (`cargo audit`)
- Dependabot configuration for Cargo, GitHub Actions, and pip dependencies
- Makefile with common development commands (`make check`, `make test`, `make lint`)
- Pre-commit hook setup script (`make setup-hooks`)
- Shared `ui::utils::centered_rect()` utility to eliminate code duplication
- Lazy-compiled regex in log parser via `OnceLock` for better performance
- `dev-dependencies`: `tempfile`, `pretty_assertions` for testing infrastructure
- This CHANGELOG

### Fixed
- Removed `#![allow(dead_code)]` that was globally suppressing compiler warnings
- Git staging errors are now properly reported to the user instead of silently swallowed
- `git pull` no longer passes `--allow-unrelated-histories` by default (safety fix)
- Homebrew formula test now correctly checks for non-repo error message

### Removed
- Unused dependencies: `clap`, `tokio`, `chrono` (significant compile time reduction)

### Changed
- `Cargo.lock` is now committed (correct practice for binary crates)
- Regex in log parser is now compiled once and reused via `OnceLock`

## [0.1.1] - 2025-01-01

### Added
- Initial release
- Repository dashboard with at-a-glance status
- Interactive file staging with diff previews
- Commit editor with validation
- Branch management (create, delete, rename, switch)
- Timeline view with git log graph
- Time travel (reset/restore) with safety confirmations
- Reflog viewer with filtering
- GitHub integration (OAuth device flow, repo creation, push/pull/sync, collaborators)
- Homebrew formula for macOS installation
