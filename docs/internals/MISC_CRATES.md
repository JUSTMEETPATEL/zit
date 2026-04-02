# Additional Crates (dirs, regex, unicode-width, cli-clipboard, libc, log, env_logger)

## dirs
**What**: Cross-platform directory paths (config, home, data). **Where**: `config.rs:328` — `dirs::config_dir()` returns `~/.config` on Linux, `~/Library/Application Support` on macOS. **One-liner**: "dirs gives us the correct config directory on every OS."

## regex
**What**: Regular expressions for pattern matching. **Where**: `src/git/log.rs` — parses git log output with custom format strings. Uses `OnceLock` for lazy compilation. **One-liner**: "We use regex to parse git log's graph output into structured commit entries."

## unicode-width
**What**: Calculates display width of Unicode characters (CJK characters are 2 columns wide). **Where**: `src/ui/utils.rs` — ensures TUI layout doesn't break with non-ASCII text. **One-liner**: "unicode-width ensures our terminal UI renders correctly for non-English characters."

## cli-clipboard
**What**: Cross-platform clipboard access (copy/paste). **Where**: Used in GitHub UI to copy verification codes and URLs to clipboard. **One-liner**: "cli-clipboard lets users copy GitHub OAuth codes to the clipboard directly from the terminal."

## libc
**What**: Raw C library bindings for Unix systems. **Where**: Low-level signal handling and terminal operations. **One-liner**: "libc provides the low-level Unix syscall access needed for terminal control."

## log + env_logger
**What**: Rust's standard logging facade (`log`) with a configurable backend (`env_logger`). **Where**: `main.rs:85–88` — initializes with `ZIT_LOG` env var filter. `log::debug!()`, `log::info!()`, `log::warn!()` throughout the codebase. **One-liner**: "log + env_logger give us structured debug logging controlled by the `ZIT_LOG=debug` environment variable."

## tempfile + pretty_assertions (dev-dependencies)
**What**: `tempfile` creates temporary directories for integration tests. `pretty_assertions` provides colorful diff output for test failures. **Where**: `tests/integration.rs` — creates temp git repos for testing. **One-liner**: "These dev-only dependencies make our 178 tests more reliable and their failures easier to debug."
